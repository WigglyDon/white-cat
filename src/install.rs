use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;

use crate::contract::{MANIFEST_FILE, PET_ID, SHEET_FILE};
use crate::error::{Result, WhiteCatError, fail};
use crate::validate::validate_project;

#[derive(Debug)]
pub struct InstallOutcome {
    pub target: PathBuf,
    pub backup: Option<PathBuf>,
}

fn codex_home() -> Result<PathBuf> {
    if let Some(path) = env::var_os("CODEX_HOME") {
        return Ok(PathBuf::from(path));
    }
    let home = env::var_os("HOME")
        .ok_or_else(|| WhiteCatError::new("HOME and CODEX_HOME are both unset"))?;
    Ok(PathBuf::from(home).join(".codex"))
}

pub fn installed_pet_path() -> Result<PathBuf> {
    Ok(codex_home()?.join("pets").join(PET_ID))
}

fn unused_backup_path(directory: &Path, stem: &str) -> PathBuf {
    let first = directory.join(stem);
    if !first.exists() {
        return first;
    }
    for suffix in 1u32.. {
        let candidate = directory.join(format!("{stem}-{suffix}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    unreachable!("finite filesystem exhausted every backup suffix")
}

fn copy_runtime_assets(project: &Path, staging: &Path) -> Result<()> {
    fs::create_dir(staging)?;
    fs::copy(project.join(MANIFEST_FILE), staging.join(MANIFEST_FILE))?;
    fs::copy(project.join(SHEET_FILE), staging.join(SHEET_FILE))?;
    Ok(())
}

fn install_pet_at<F>(
    project: &Path,
    root: &Path,
    force: bool,
    check_sources: bool,
    before_activation: F,
) -> Result<InstallOutcome>
where
    F: FnOnce(&Path) -> Result<()>,
{
    validate_project(project, check_sources)?;
    let pets = root.join("pets");
    let target = pets.join(PET_ID);
    if target.exists() && !force {
        return fail(format!(
            "{} already exists; use install-force for validated atomic replacement",
            target.display()
        ));
    }

    fs::create_dir_all(&pets)?;
    let staging = pets.join(format!(".{PET_ID}.staging-{}", std::process::id()));
    if staging.exists() {
        return fail(format!("staging path {} already exists", staging.display()));
    }
    copy_runtime_assets(project, &staging)?;
    if let Err(error) = validate_project(&staging, false) {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }

    let mut backup = None;
    if target.exists() {
        let backup_directory = root.join("pet-backups");
        fs::create_dir_all(&backup_directory)?;
        let timestamp = Utc::now().format("%Y%m%dT%H%M%S%.3fZ");
        let destination =
            unused_backup_path(&backup_directory, &format!("{PET_ID}.backup-{timestamp}"));
        fs::rename(&target, &destination)?;
        backup = Some(destination);
    }

    if let Err(error) = before_activation(&staging) {
        let _ = fs::remove_dir_all(&staging);
        if let Some(previous) = backup.as_ref() {
            let _ = fs::rename(previous, &target);
        }
        return Err(error);
    }

    if let Err(error) = fs::rename(&staging, &target) {
        if let Some(previous) = backup.as_ref() {
            let _ = fs::rename(previous, &target);
        }
        return Err(error.into());
    }

    if let Err(error) = validate_project(&target, false) {
        let failed = pets.join(format!(".{PET_ID}.failed-{}", std::process::id()));
        let _ = fs::rename(&target, &failed);
        if let Some(previous) = backup.as_ref() {
            let _ = fs::rename(previous, &target);
        }
        let _ = fs::remove_dir_all(&failed);
        return Err(error);
    }

    Ok(InstallOutcome { target, backup })
}

pub fn install_pet_to_root(project: &Path, root: &Path, force: bool) -> Result<InstallOutcome> {
    install_pet_at(project, root, force, true, |_| Ok(()))
}

pub fn install_pet(project: &Path, force: bool) -> Result<InstallOutcome> {
    let root = codex_home()?;
    install_pet_to_root(project, &root, force)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary_directory(label: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("white-cat-{label}-{}-{nonce}", std::process::id()))
    }

    #[test]
    fn backup_names_never_overwrite() {
        let directory = temporary_directory("backup-name-test");
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("create test directory");
        fs::create_dir(directory.join("white-cat.backup-fixed")).expect("create collision");
        assert_eq!(
            unused_backup_path(&directory, "white-cat.backup-fixed"),
            directory.join("white-cat.backup-fixed-1")
        );
        fs::remove_dir_all(directory).expect("remove test directory");
    }

    #[test]
    fn activation_failure_rolls_back_the_previous_runtime() {
        let directory = temporary_directory("rollback-test");
        let project = directory.join("project");
        let root = directory.join("codex-home");
        fs::create_dir_all(&project).expect("create generated project");
        crate::artwork::generate_project(&project).expect("generate exact runtime payload");

        let first = install_pet_at(&project, &root, false, false, |_| Ok(()))
            .expect("install initial runtime");
        let marker = first.target.join("rollback-marker");
        fs::write(&marker, b"previous runtime").expect("write rollback marker");

        let failure = install_pet_at(&project, &root, true, false, |staging| {
            fs::remove_dir_all(staging)?;
            Ok(())
        });
        assert!(failure.is_err(), "activation unexpectedly succeeded");
        assert_eq!(
            fs::read(marker).expect("previous runtime restored"),
            b"previous runtime"
        );
        crate::validate::validate_project(&first.target, false)
            .expect("restored runtime remains valid");
        fs::remove_dir_all(directory).expect("remove rollback test directory");
    }
}

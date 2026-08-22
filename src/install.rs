use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Local;

use crate::validate::validate_project;
use crate::{Result, WhiteCatError};

#[derive(Debug, Eq, PartialEq)]
pub struct InstallOutcome {
    pub target: PathBuf,
    pub backup: Option<PathBuf>,
}

pub fn resolve_codex_home() -> Result<PathBuf> {
    if let Some(configured) = env::var_os("CODEX_HOME") {
        return Ok(PathBuf::from(configured));
    }
    let home = env::var_os("HOME")
        .ok_or_else(|| WhiteCatError::new("neither CODEX_HOME nor HOME is set"))?;
    Ok(PathBuf::from(home).join(".codex"))
}

fn unused_backup_path(backup_root: &Path, target_name: &str, timestamp: &str) -> PathBuf {
    let base = format!("{target_name}.backup-{timestamp}");
    let first = backup_root.join(&base);
    if !first.exists() {
        return first;
    }
    for suffix in 1.. {
        let candidate = backup_root.join(format!("{base}-{suffix}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    unreachable!()
}

fn create_staging(pets_dir: &Path) -> Result<PathBuf> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| WhiteCatError::new(error.to_string()))?
        .as_nanos();
    for suffix in 0..1000_u32 {
        let staging = pets_dir.join(format!(
            ".white-cat-install-{}-{nanos}-{suffix}",
            std::process::id()
        ));
        match fs::create_dir(&staging) {
            Ok(()) => return Ok(staging),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }
    Err(WhiteCatError::new(
        "could not allocate a unique installer staging directory",
    ))
}

#[cfg(unix)]
fn ensure_private_directory(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::create_dir_all(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

#[cfg(not(unix))]
fn ensure_private_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    Ok(())
}

#[cfg(unix)]
fn same_filesystem(left: &Path, right: &Path) -> Result<bool> {
    use std::os::unix::fs::MetadataExt;
    Ok(fs::metadata(left)?.dev() == fs::metadata(right)?.dev())
}

#[cfg(not(unix))]
fn same_filesystem(_left: &Path, _right: &Path) -> Result<bool> {
    Ok(true)
}

fn install_with_hooks<V, A>(
    project: &Path,
    codex_home: &Path,
    force: bool,
    timestamp: &str,
    mut validate: V,
    mut activate: A,
) -> Result<InstallOutcome>
where
    V: FnMut(&Path, bool) -> Result<()>,
    A: FnMut(&Path, &Path) -> std::io::Result<()>,
{
    validate(project, true)?;
    let pets_dir = codex_home.join("pets");
    let backup_root = codex_home.join("pet-backups");
    let target = pets_dir.join("white-cat");

    if target.exists() && !force {
        return Err(WhiteCatError::new(format!(
            "installation exists at {}; use --force to back it up before replacement",
            target.display()
        )));
    }
    if target.exists() && !target.is_dir() {
        return Err(WhiteCatError::new(format!(
            "refusing to replace non-directory path: {}",
            target.display()
        )));
    }

    ensure_private_directory(&pets_dir)?;
    ensure_private_directory(&backup_root)?;
    if target.exists() && !same_filesystem(&target, &backup_root)? {
        return Err(WhiteCatError::new(format!(
            "active pet and backup root must share a filesystem for atomic rollback: {}",
            backup_root.display()
        )));
    }

    let staging = create_staging(&pets_dir)?;
    let mut backup = None;
    let result = (|| {
        fs::copy(
            project.join("spritesheet.webp"),
            staging.join("spritesheet.webp"),
        )?;
        fs::copy(project.join("pet.json"), staging.join("pet.json"))?;
        validate(&staging, false)?;

        if target.exists() {
            let path = unused_backup_path(&backup_root, "white-cat", timestamp);
            fs::rename(&target, &path)?;
            backup = Some(path);
        }

        if let Err(activation_error) = activate(&staging, &target) {
            if let Some(path) = backup.as_ref()
                && path.exists()
                && !target.exists()
                && let Err(rollback_error) = fs::rename(path, &target)
            {
                return Err(WhiteCatError::new(format!(
                    "activation failed: {activation_error}; rollback failed: {rollback_error}"
                )));
            }
            return Err(activation_error.into());
        }
        Ok(InstallOutcome { target, backup })
    })();

    if staging.exists() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

pub fn install_pet(project: &Path, force: bool) -> Result<InstallOutcome> {
    let codex_home = resolve_codex_home()?;
    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    install_with_hooks(
        project,
        &codex_home,
        force,
        &timestamp,
        validate_project,
        |source, target| fs::rename(source, target),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(directory: &Path) -> PathBuf {
        let project = directory.join("project");
        fs::create_dir(&project).unwrap();
        fs::write(project.join("pet.json"), b"new-manifest").unwrap();
        fs::write(project.join("spritesheet.webp"), b"new-sheet").unwrap();
        project
    }

    fn active_pet(codex_home: &Path) -> PathBuf {
        let target = codex_home.join("pets/white-cat");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("pet.json"), b"old-manifest").unwrap();
        fs::write(target.join("spritesheet.webp"), b"old-sheet").unwrap();
        target
    }

    fn accept_fixture(_path: &Path, _check_sources: bool) -> Result<()> {
        Ok(())
    }

    #[test]
    fn non_force_refuses_replacement() {
        let root = tempfile::tempdir().unwrap();
        let source = project(root.path());
        let codex_home = root.path().join("codex");
        let target = active_pet(&codex_home);
        let error = install_with_hooks(
            &source,
            &codex_home,
            false,
            "20260718-010203",
            accept_fixture,
            |source, target| fs::rename(source, target),
        )
        .unwrap_err();
        assert!(error.to_string().contains("use --force"));
        assert_eq!(fs::read(target.join("pet.json")).unwrap(), b"old-manifest");
    }

    #[test]
    fn force_moves_old_runtime_outside_discovery() {
        let root = tempfile::tempdir().unwrap();
        let source = project(root.path());
        let codex_home = root.path().join("codex");
        active_pet(&codex_home);
        let outcome = install_with_hooks(
            &source,
            &codex_home,
            true,
            "20260718-010203",
            accept_fixture,
            |source, target| fs::rename(source, target),
        )
        .unwrap();
        let backup = outcome.backup.unwrap();
        assert_eq!(
            backup,
            codex_home.join("pet-backups/white-cat.backup-20260718-010203")
        );
        assert_eq!(fs::read(backup.join("pet.json")).unwrap(), b"old-manifest");
        assert_eq!(
            fs::read(outcome.target.join("pet.json")).unwrap(),
            b"new-manifest"
        );
        assert!(fs::read_dir(codex_home.join("pets")).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("white-cat.backup-")
        }));
    }

    #[test]
    fn occupied_backup_name_is_never_overwritten() {
        let root = tempfile::tempdir().unwrap();
        let source = project(root.path());
        let codex_home = root.path().join("codex");
        active_pet(&codex_home);
        let occupied = codex_home.join("pet-backups/white-cat.backup-20260718-010203");
        fs::create_dir_all(&occupied).unwrap();
        fs::write(occupied.join("sentinel"), b"preserve").unwrap();
        let outcome = install_with_hooks(
            &source,
            &codex_home,
            true,
            "20260718-010203",
            accept_fixture,
            |source, target| fs::rename(source, target),
        )
        .unwrap();
        assert_eq!(
            outcome.backup.unwrap(),
            codex_home.join("pet-backups/white-cat.backup-20260718-010203-1")
        );
        assert_eq!(fs::read(occupied.join("sentinel")).unwrap(), b"preserve");
    }

    #[test]
    fn failed_activation_rolls_back_original_runtime() {
        let root = tempfile::tempdir().unwrap();
        let source = project(root.path());
        let codex_home = root.path().join("codex");
        let target = active_pet(&codex_home);
        let error = install_with_hooks(
            &source,
            &codex_home,
            true,
            "20260718-010203",
            accept_fixture,
            |_source, _target| Err(std::io::Error::other("injected activation failure")),
        )
        .unwrap_err();
        assert!(error.to_string().contains("injected activation failure"));
        assert_eq!(fs::read(target.join("pet.json")).unwrap(), b"old-manifest");
        assert!(
            !codex_home
                .join("pet-backups/white-cat.backup-20260718-010203")
                .exists()
        );
        assert!(fs::read_dir(codex_home.join("pets")).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".white-cat-install-")
        }));
    }
}

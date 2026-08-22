use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::artwork::build_frames;
use crate::contract::{
    ALIASES, ANIMATION_RANGES, COLUMNS, FRAME_HEIGHT, FRAME_WIDTH, ROWS, animation_fps,
    animation_loops, animation_timeline,
};
use crate::sheet::{pack_fixed_frames, save_lossless_sheet};
use crate::{Result, WhiteCatError};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub spritesheet_path: String,
    pub frame: FrameGeometry,
    pub frame_allocation: BTreeMap<String, FrameAllocation>,
    pub animations: BTreeMap<String, AnimationSpec>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FrameGeometry {
    pub width: usize,
    pub height: usize,
    pub columns: usize,
    pub rows: usize,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FrameAllocation {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AnimationSpec {
    pub frames: Vec<usize>,
    pub fps: u64,
    #[serde(rename = "loop")]
    pub loops: bool,
    pub fallback: String,
}

pub fn build_manifest(description: &str) -> Manifest {
    let frame_allocation = ANIMATION_RANGES
        .iter()
        .map(|range| {
            (
                range.name.to_owned(),
                FrameAllocation {
                    start: range.start,
                    end: range.end,
                },
            )
        })
        .collect();

    let mut names: Vec<&str> = ANIMATION_RANGES.iter().map(|range| range.name).collect();
    names.extend(ALIASES.iter().map(|(alias, _)| *alias));
    names.sort_unstable();
    let animations = names
        .into_iter()
        .map(|name| {
            (
                name.to_owned(),
                AnimationSpec {
                    frames: animation_timeline(name).expect("known animation").to_vec(),
                    fps: animation_fps(name).expect("known animation"),
                    loops: animation_loops(name),
                    fallback: "idle".to_owned(),
                },
            )
        })
        .collect();

    Manifest {
        id: "white-cat".to_owned(),
        display_name: "White Cat".to_owned(),
        description: description.to_owned(),
        spritesheet_path: "spritesheet.webp".to_owned(),
        frame: FrameGeometry {
            width: FRAME_WIDTH,
            height: FRAME_HEIGHT,
            columns: COLUMNS,
            rows: ROWS,
        },
        frame_allocation,
        animations,
    }
}

pub fn load_manifest(path: &Path) -> Result<Manifest> {
    let bytes = fs::read(path).map_err(|error| {
        WhiteCatError::new(format!("cannot read manifest {}: {error}", path.display()))
    })?;
    serde_json::from_slice(&bytes).map_err(|error| {
        WhiteCatError::new(format!("cannot read manifest {}: {error}", path.display()))
    })
}

pub fn write_manifest(manifest: &Manifest, path: &Path) -> Result<()> {
    let mut encoded = serde_json::to_vec_pretty(manifest)?;
    encoded.push(b'\n');
    fs::write(path, encoded)?;
    Ok(())
}

pub fn generate(project: &Path) -> Result<(PathBuf, PathBuf)> {
    fs::create_dir_all(project)?;
    let manifest_path = project.join("pet.json");
    let sheet_path = project.join("spritesheet.webp");
    let manifest_temp = project.join(".pet.json.tmp");
    let sheet_temp = project.join(".spritesheet.webp.tmp");

    let result = (|| {
        let frames = build_frames()?;
        let sheet = pack_fixed_frames(&frames)?;
        write_manifest(
            &build_manifest("A quiet, watchful white terminal cat."),
            &manifest_temp,
        )?;
        save_lossless_sheet(&sheet, &sheet_temp)?;
        fs::rename(&sheet_temp, &sheet_path)?;
        fs::rename(&manifest_temp, &manifest_path)?;
        Ok((manifest_path, sheet_path))
    })();

    let _ = fs::remove_file(&manifest_temp);
    let _ = fs::remove_file(&sheet_temp);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artwork::build_frames;
    use crate::contract::{FRAME_COUNT, SHEET_HEIGHT, SHEET_WIDTH};
    use crate::sheet::{extract_fixed_frames, load_sheet};

    #[test]
    fn manifest_is_explicit_and_complete() {
        let manifest = build_manifest("fixture");
        assert_eq!(
            manifest.frame,
            FrameGeometry {
                width: 192,
                height: 208,
                columns: 8,
                rows: 9
            }
        );
        assert_eq!(manifest.frame_allocation.len(), ROWS);
        assert_eq!(
            manifest.animations.len(),
            ANIMATION_RANGES.len() + ALIASES.len()
        );
        let covered: usize = manifest
            .frame_allocation
            .values()
            .map(|allocation| allocation.end - allocation.start + 1)
            .sum();
        assert_eq!(covered, FRAME_COUNT);
        assert_eq!((SHEET_WIDTH, SHEET_HEIGHT), (1536, 1872));
        assert_eq!(manifest.animations["idle"].fallback, "idle");
    }

    #[test]
    fn generated_lossless_sheet_round_trips_authored_pixels() {
        let directory = tempfile::tempdir().unwrap();
        generate(directory.path()).unwrap();
        let sheet = load_sheet(&directory.path().join("spritesheet.webp")).unwrap();
        assert_eq!(
            extract_fixed_frames(&sheet).unwrap(),
            build_frames().unwrap()
        );
    }
}

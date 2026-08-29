use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::contract::{
    ALIASES, FRAME_HEIGHT, FRAME_WIDTH, GRID_COLUMNS, GRID_ROWS, MANIFEST_FILE, PET_ID, SHEET_FILE,
    STATES, animation_fps, animation_timeline, state_named,
};
use crate::error::Result;
use crate::sheet::write_atomic;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PetManifest {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub spritesheet_path: String,
    pub frame: FrameGeometry,
    pub frame_allocation: BTreeMap<String, FrameRange>,
    pub animations: BTreeMap<String, Animation>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FrameGeometry {
    pub width: u32,
    pub height: u32,
    pub columns: u32,
    pub rows: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FrameRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Animation {
    pub frames: Vec<usize>,
    pub fps: u32,
    #[serde(rename = "loop")]
    pub loops: bool,
    pub fallback: String,
}

fn animated_state(name: &str, loops: bool) -> Animation {
    Animation {
        frames: animation_timeline(name)
            .expect("declared state has an animation timeline")
            .to_vec(),
        fps: animation_fps(name).expect("declared state has animation timing"),
        loops,
        fallback: "idle".to_owned(),
    }
}

pub fn build_manifest() -> PetManifest {
    let mut frame_allocation = BTreeMap::new();
    let mut animations = BTreeMap::new();
    for state in STATES {
        frame_allocation.insert(
            state.name.to_owned(),
            FrameRange {
                start: state.start,
                end: state.end,
            },
        );
        animations.insert(
            state.name.to_owned(),
            animated_state(state.name, state.loops),
        );
    }
    for (alias, target) in ALIASES {
        let target = state_named(target).expect("alias target is a declared state");
        animations.insert(alias.to_owned(), animated_state(target.name, target.loops));
    }

    PetManifest {
        id: PET_ID.to_owned(),
        display_name: "White Cat".to_owned(),
        description: "A quiet, watchful white terminal cat with responsive pixel animation."
            .to_owned(),
        spritesheet_path: SHEET_FILE.to_owned(),
        frame: FrameGeometry {
            width: FRAME_WIDTH,
            height: FRAME_HEIGHT,
            columns: GRID_COLUMNS,
            rows: GRID_ROWS,
        },
        frame_allocation,
        animations,
    }
}

pub fn write_manifest(project: &Path) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(&build_manifest())?;
    bytes.push(b'\n');
    write_atomic(&project.join(MANIFEST_FILE), &bytes)
}

pub fn read_manifest(project: &Path) -> Result<PetManifest> {
    Ok(serde_json::from_slice(&fs::read(
        project.join(MANIFEST_FILE),
    )?)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_runtime_states_have_explicit_timed_sequences() {
        let manifest = build_manifest();
        assert_eq!(manifest.frame_allocation.len(), STATES.len());
        assert_eq!(manifest.animations.len(), STATES.len() + ALIASES.len());
        assert!(
            manifest
                .animations
                .values()
                .all(|animation| animation.frames.len() >= 8 && animation.fps >= 8)
        );
    }
}

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use image::{Rgba, RgbaImage};

use crate::contract::{
    ANIMATION_RANGES, COLUMNS, FRAME_COUNT, FRAME_HEIGHT, FRAME_WIDTH, PIXEL_SCALE, SOURCE_HEIGHT,
    SOURCE_WIDTH, resolve_state,
};
use crate::maps::{POSES, PixelMap};
use crate::{Result, WhiteCatError};

pub const PALETTE: [(char, [u8; 4]); 5] = [
    ('.', [0, 0, 0, 0]),
    ('O', [42, 51, 64, 255]),
    ('W', [244, 242, 232, 255]),
    ('S', [205, 210, 216, 255]),
    ('E', [134, 215, 168, 255]),
];

pub const FRAME_POSES: &[(&str, [&str; COLUMNS])] = &[
    (
        "idle",
        [
            "neutral", "neutral", "blink", "neutral", "ear", "neutral", "tail", "neutral",
        ],
    ),
    (
        "running-right",
        [
            "run-right-a",
            "run-right-b",
            "run-right-a",
            "run-right-b",
            "run-right-a",
            "run-right-b",
            "run-right-a",
            "run-right-b",
        ],
    ),
    (
        "running-left",
        [
            "run-left-a",
            "run-left-b",
            "run-left-a",
            "run-left-b",
            "run-left-a",
            "run-left-b",
            "run-left-a",
            "run-left-b",
        ],
    ),
    (
        "waving",
        [
            "neutral",
            "wave-low",
            "wave-high",
            "wave-high",
            "wave-low",
            "wave-high",
            "wave-low",
            "neutral",
        ],
    ),
    (
        "jumping",
        [
            "run-right-a",
            "run-right-b",
            "jump-right-a",
            "jump-right-b",
            "jump-right-b",
            "jump-right-a",
            "run-right-b",
            "run-right-a",
        ],
    ),
    (
        "failed",
        [
            "failed",
            "failed",
            "failed-blink",
            "failed",
            "failed",
            "failed-blink",
            "failed",
            "failed",
        ],
    ),
    (
        "waiting",
        [
            "neutral", "waiting", "waiting", "neutral", "waiting", "waiting", "neutral", "neutral",
        ],
    ),
    (
        "running",
        [
            "working",
            "working",
            "working-press",
            "working-press",
            "working",
            "working",
            "working-press",
            "working",
        ],
    ),
    (
        "review",
        [
            "neutral", "review", "review", "blink", "review", "review", "neutral", "neutral",
        ],
    ),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedArtwork {
    poses: BTreeMap<String, Vec<String>>,
}

impl ParsedArtwork {
    pub fn pose(&self, name: &str) -> Result<&[String]> {
        self.poses
            .get(name)
            .map(Vec::as_slice)
            .ok_or_else(|| WhiteCatError::new(format!("unknown pose {name:?}")))
    }

    pub fn len(&self) -> usize {
        self.poses.len()
    }

    pub fn is_empty(&self) -> bool {
        self.poses.is_empty()
    }
}

pub fn palette_color(pixel: char) -> Option<Rgba<u8>> {
    PALETTE
        .iter()
        .find_map(|(symbol, color)| (*symbol == pixel).then_some(Rgba(*color)))
}

pub fn pose(name: &str) -> Option<&'static PixelMap> {
    POSES
        .iter()
        .find_map(|(pose_name, map)| (*pose_name == name).then_some(map))
}

pub fn frame_pose_names(state: &str) -> Result<&'static [&'static str; COLUMNS]> {
    let resolved = resolve_state(state)
        .ok_or_else(|| WhiteCatError::new(format!("unknown state {state:?}")))?;
    FRAME_POSES
        .iter()
        .find_map(|(name, poses)| (*name == resolved).then_some(poses))
        .ok_or_else(|| WhiteCatError::new(format!("state {resolved:?} has no frame plan")))
}

fn validate_rows<S: AsRef<str>>(name: &str, rows: &[S]) -> Result<()> {
    if rows.len() != SOURCE_HEIGHT {
        return Err(WhiteCatError::new(format!(
            "pose {name} has {} rows, expected {SOURCE_HEIGHT}",
            rows.len()
        )));
    }
    for (row_index, row) in rows.iter().enumerate() {
        let row = row.as_ref();
        if row.len() != SOURCE_WIDTH {
            return Err(WhiteCatError::new(format!(
                "pose {name} row {row_index} has width {}, expected {SOURCE_WIDTH}",
                row.len()
            )));
        }
        if row.as_bytes().first() != Some(&b'.') || row.as_bytes().last() != Some(&b'.') {
            return Err(WhiteCatError::new(format!(
                "pose {name} row {row_index} violates horizontal padding"
            )));
        }
        if let Some(pixel) = row.chars().find(|pixel| palette_color(*pixel).is_none()) {
            return Err(WhiteCatError::new(format!(
                "pose {name} row {row_index} uses unknown pixel {pixel:?}"
            )));
        }
    }
    Ok(())
}

pub fn validate_source_maps() -> Result<()> {
    if SOURCE_WIDTH * PIXEL_SCALE != FRAME_WIDTH || SOURCE_HEIGHT * PIXEL_SCALE != FRAME_HEIGHT {
        return Err(WhiteCatError::new(
            "source geometry does not scale exactly to the runtime frame",
        ));
    }
    for (name, rows) in POSES {
        validate_rows(name, rows)?;
    }
    if FRAME_POSES.len() != ANIMATION_RANGES.len() {
        return Err(WhiteCatError::new(
            "frame pose rows do not match the runtime allocation",
        ));
    }
    for range in ANIMATION_RANGES {
        let names = frame_pose_names(range.name)?;
        for name in names {
            if pose(name).is_none() {
                return Err(WhiteCatError::new(format!(
                    "animation {} references unknown pose {name}",
                    range.name
                )));
            }
        }
    }
    Ok(())
}

pub fn render_pose<S: AsRef<str>>(name: &str, rows: &[S]) -> Result<RgbaImage> {
    validate_rows(name, rows)?;
    let mut frame = RgbaImage::new(FRAME_WIDTH as u32, FRAME_HEIGHT as u32);
    for (source_y, row) in rows.iter().enumerate() {
        for (source_x, pixel) in row.as_ref().chars().enumerate() {
            let color = palette_color(pixel).expect("validated palette pixel");
            let origin_x = source_x * PIXEL_SCALE;
            let origin_y = source_y * PIXEL_SCALE;
            for y in origin_y..origin_y + PIXEL_SCALE {
                for x in origin_x..origin_x + PIXEL_SCALE {
                    frame.put_pixel(x as u32, y as u32, color);
                }
            }
        }
    }
    Ok(frame)
}

pub fn build_frames() -> Result<Vec<RgbaImage>> {
    validate_source_maps()?;
    let mut frames = Vec::with_capacity(FRAME_COUNT);
    for range in ANIMATION_RANGES {
        for pose_name in frame_pose_names(range.name)? {
            let map = pose(pose_name).expect("validated frame pose");
            frames.push(render_pose(pose_name, map)?);
        }
    }
    if frames.len() != FRAME_COUNT {
        return Err(WhiteCatError::new(format!(
            "built {} frames, expected {FRAME_COUNT}",
            frames.len()
        )));
    }
    Ok(frames)
}

pub fn parse_maps_source(source: &str) -> Result<ParsedArtwork> {
    let mut poses = BTreeMap::new();
    let mut current_name: Option<String> = None;
    let mut current_rows = Vec::new();
    let mut in_poses = false;
    let mut awaiting_name = false;
    let mut reading_rows = false;

    for line in source.lines() {
        let trimmed = line.trim();
        if !in_poses {
            in_poses = trimmed.starts_with("pub const POSES:");
            continue;
        }
        if trimmed == "];" && current_name.is_none() {
            break;
        }
        if current_name.is_none() && trimmed == "(" {
            awaiting_name = true;
            continue;
        }
        if awaiting_name {
            let name = trimmed
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix("\","))
                .ok_or_else(|| {
                    WhiteCatError::new(format!("invalid pose name in Rust source: {trimmed}"))
                })?;
            current_name = Some(name.to_owned());
            current_rows.clear();
            awaiting_name = false;
            continue;
        }
        if current_name.is_some() && !reading_rows && trimmed == "[" {
            reading_rows = true;
            continue;
        }
        if reading_rows && trimmed == "]," {
            let name = current_name.take().expect("pose parser state");
            validate_rows(&name, &current_rows)?;
            if poses.insert(name.clone(), current_rows.clone()).is_some() {
                return Err(WhiteCatError::new(format!("duplicate pose {name:?}")));
            }
            reading_rows = false;
            continue;
        }
        if reading_rows {
            let row = trimmed
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix("\","))
                .ok_or_else(|| {
                    WhiteCatError::new(format!("invalid row in pose source: {trimmed}"))
                })?;
            current_rows.push(row.to_owned());
        }
    }

    if let Some(name) = current_name {
        return Err(WhiteCatError::new(format!("unterminated pose {name:?}")));
    }
    if poses.is_empty() {
        return Err(WhiteCatError::new("pixel-map source defines no poses"));
    }
    for (_, names) in FRAME_POSES {
        for name in names {
            if !poses.contains_key(*name) {
                return Err(WhiteCatError::new(format!(
                    "live source is missing required pose {name:?}"
                )));
            }
        }
    }
    Ok(ParsedArtwork { poses })
}

pub fn load_maps_source(path: &Path) -> Result<ParsedArtwork> {
    let source = fs::read_to_string(path).map_err(|error| {
        WhiteCatError::new(format!(
            "cannot read pixel-map source {}: {error}",
            path.display()
        ))
    })?;
    parse_maps_source(&source)
}

pub fn parsed_source_pose<'a>(
    artwork: &'a ParsedArtwork,
    state: &str,
    frame_index: usize,
) -> Result<&'a [String]> {
    let names = frame_pose_names(state)?;
    artwork.pose(names[frame_index % names.len()])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pose_is_a_complete_literal_map() {
        validate_source_maps().unwrap();
        assert_eq!((SOURCE_WIDTH, SOURCE_HEIGHT, PIXEL_SCALE), (24, 26, 8));
    }

    #[test]
    fn palette_matches_the_approved_concept() {
        assert_eq!(
            PALETTE,
            [
                ('.', [0, 0, 0, 0]),
                ('O', [42, 51, 64, 255]),
                ('W', [244, 242, 232, 255]),
                ('S', [205, 210, 216, 255]),
                ('E', [134, 215, 168, 255]),
            ]
        );
    }

    #[test]
    fn idle_plan_is_restrained() {
        assert_eq!(
            frame_pose_names("idle").unwrap(),
            &[
                "neutral", "neutral", "blink", "neutral", "ear", "neutral", "tail", "neutral"
            ]
        );
    }

    #[test]
    fn idle_variants_change_at_most_six_pixels() {
        let neutral = pose("neutral").unwrap();
        for name in ["blink", "ear", "tail"] {
            let variant = pose(name).unwrap();
            let changed = neutral
                .iter()
                .zip(variant)
                .flat_map(|(before, after)| before.bytes().zip(after.bytes()))
                .filter(|(before, after)| before != after)
                .count();
            assert!(changed <= 6, "{name} changed {changed} pixels");
            assert_eq!(&variant[8..18], &neutral[8..18]);
            assert_eq!(&variant[20..24], &neutral[20..24]);
        }
    }

    #[test]
    fn live_parser_reads_the_same_compiled_maps() {
        let parsed = parse_maps_source(include_str!("maps.rs")).unwrap();
        assert_eq!(parsed.len(), POSES.len());
        for (name, map) in POSES {
            let expected: Vec<String> = map.iter().map(|row| (*row).to_owned()).collect();
            assert_eq!(parsed.pose(name).unwrap(), expected);
        }
    }

    #[test]
    fn fixed_scale_produces_solid_source_blocks() {
        let frame = render_pose("neutral", pose("neutral").unwrap()).unwrap();
        for source_y in 0..SOURCE_HEIGHT {
            for source_x in 0..SOURCE_WIDTH {
                let first = frame.get_pixel(
                    (source_x * PIXEL_SCALE) as u32,
                    (source_y * PIXEL_SCALE) as u32,
                );
                for y in source_y * PIXEL_SCALE..(source_y + 1) * PIXEL_SCALE {
                    for x in source_x * PIXEL_SCALE..(source_x + 1) * PIXEL_SCALE {
                        assert_eq!(frame.get_pixel(x as u32, y as u32), first);
                    }
                }
            }
        }
    }
}

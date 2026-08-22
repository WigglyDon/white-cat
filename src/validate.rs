use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use image::RgbaImage;

use crate::artwork::{build_frames, load_maps_source};
use crate::contract::{
    ALIASES, ANIMATION_RANGES, COLUMNS, FRAME_COUNT, FRAME_HEIGHT, FRAME_WIDTH, GROUND_PIXEL_Y,
    GROUND_Y, GROUNDED_ANIMATIONS, JUMP_GROUNDED_FRAMES, ROWS, TRANSPARENT_MARGIN, animation_fps,
    animation_loops, animation_timeline, state_for_index,
};
use crate::manifest::{FrameAllocation, FrameGeometry, Manifest, load_manifest};
use crate::sheet::{extract_fixed_frames, load_sheet, webp_chunk_ids};
use crate::{Result, WhiteCatError};

const ALPHA_THRESHOLD: u8 = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameBounds {
    pub left: usize,
    pub top: usize,
    pub right: usize,
    pub bottom_exclusive: usize,
}

impl FrameBounds {
    pub fn bottom(self) -> usize {
        self.bottom_exclusive - 1
    }

    pub fn height(self) -> usize {
        self.bottom_exclusive - self.top
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameRecord {
    pub index: usize,
    pub animation: &'static str,
    pub bounds: FrameBounds,
}

fn require(condition: bool, message: impl Into<String>) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(WhiteCatError::new(message))
    }
}

pub fn opaque_bounds(frame: &RgbaImage) -> Result<FrameBounds> {
    let mut left = FRAME_WIDTH;
    let mut top = FRAME_HEIGHT;
    let mut right = 0;
    let mut bottom = 0;
    let mut found = false;
    for y in 0..FRAME_HEIGHT {
        for x in 0..FRAME_WIDTH {
            if frame.get_pixel(x as u32, y as u32).0[3] >= ALPHA_THRESHOLD {
                found = true;
                left = left.min(x);
                top = top.min(y);
                right = right.max(x + 1);
                bottom = bottom.max(y + 1);
            }
        }
    }
    require(found, "runtime frame is blank")?;
    Ok(FrameBounds {
        left,
        top,
        right,
        bottom_exclusive: bottom,
    })
}

pub fn frame_records(frames: &[RgbaImage]) -> Result<Vec<FrameRecord>> {
    frames
        .iter()
        .enumerate()
        .map(|(index, frame)| {
            Ok(FrameRecord {
                index,
                animation: state_for_index(index).ok_or_else(|| {
                    WhiteCatError::new(format!("frame {index} has no allocation"))
                })?,
                bounds: opaque_bounds(frame)?,
            })
        })
        .collect()
}

pub fn validate_manifest(manifest: &Manifest) -> Result<()> {
    require(manifest.id == "white-cat", "manifest id must be white-cat")?;
    require(
        manifest.display_name == "White Cat",
        "displayName must be White Cat",
    )?;
    require(
        manifest.spritesheet_path == "spritesheet.webp",
        "invalid spritesheetPath",
    )?;
    require(
        manifest.frame
            == (FrameGeometry {
                width: FRAME_WIDTH,
                height: FRAME_HEIGHT,
                columns: COLUMNS,
                rows: ROWS,
            }),
        "manifest frame geometry is invalid",
    )?;

    require(
        manifest.frame_allocation.len() == ANIMATION_RANGES.len(),
        "frameAllocation must contain exactly nine rows",
    )?;
    let mut documented = BTreeSet::new();
    for range in ANIMATION_RANGES {
        require(
            range.start / COLUMNS == range.end / COLUMNS,
            format!("frameAllocation {} crosses a row", range.name),
        )?;
        require(
            manifest.frame_allocation.get(range.name)
                == Some(&FrameAllocation {
                    start: range.start,
                    end: range.end,
                }),
            format!(
                "frameAllocation for {} must be {}..{}",
                range.name, range.start, range.end
            ),
        )?;
        documented.extend(range.start..=range.end);
    }
    require(
        documented == (0..FRAME_COUNT).collect(),
        "frameAllocation must cover every cell once",
    )?;

    let required: Vec<&str> = ANIMATION_RANGES
        .iter()
        .map(|range| range.name)
        .chain(ALIASES.iter().map(|(alias, _)| *alias))
        .collect();
    for name in required {
        let spec = manifest.animations.get(name).ok_or_else(|| {
            WhiteCatError::new(format!("required animation or alias is missing: {name}"))
        })?;
        require(
            spec.frames == animation_timeline(name).expect("known timeline"),
            format!("animation {name} has the wrong timeline"),
        )?;
        require(
            spec.fps == animation_fps(name).expect("known FPS"),
            format!("animation {name} has the wrong FPS"),
        )?;
        require(
            spec.loops == animation_loops(name),
            format!("animation {name} has wrong loop behavior"),
        )?;
        require(
            spec.fallback == "idle",
            format!("animation {name} fallback must be idle"),
        )?;
    }
    Ok(())
}

fn validate_webp_container(path: &Path) -> Result<()> {
    let chunks = webp_chunk_ids(path)?;
    require(
        chunks
            .iter()
            .filter(|chunk| chunk.as_slice() == b"VP8L")
            .count()
            == 1,
        "spritesheet must contain one lossless VP8L image",
    )?;
    require(
        !chunks.iter().any(|chunk| chunk.as_slice() == b"VP8 "),
        "spritesheet contains a lossy VP8 image",
    )?;
    require(
        !chunks
            .iter()
            .any(|chunk| chunk.as_slice() == b"ANIM" || chunk.as_slice() == b"ANMF"),
        "spritesheet must be static",
    )
}

fn validate_frames(frames: &[RgbaImage]) -> Result<()> {
    require(
        frames.len() == FRAME_COUNT,
        "packed sheet has the wrong cell count",
    )?;
    for (index, frame) in frames.iter().enumerate() {
        let mut transparent = false;
        let mut content = false;
        for pixel in frame.pixels() {
            transparent |= pixel.0[3] == 0;
            content |= pixel.0[3] >= ALPHA_THRESHOLD;
        }
        require(content, format!("frame {index} is blank"))?;
        require(
            transparent && content,
            format!("frame {index} lost transparency or content"),
        )?;

        for y in 0..FRAME_HEIGHT {
            for x in 0..FRAME_WIDTH {
                if !(TRANSPARENT_MARGIN..FRAME_WIDTH - TRANSPARENT_MARGIN).contains(&x)
                    || !(TRANSPARENT_MARGIN..FRAME_HEIGHT - TRANSPARENT_MARGIN).contains(&y)
                {
                    require(
                        frame.get_pixel(x as u32, y as u32).0[3] == 0,
                        format!("frame {index} violates transparent margin"),
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn baseline_bytes(frame: &RgbaImage) -> &[u8] {
    let start = GROUND_PIXEL_Y * FRAME_WIDTH * 4;
    &frame.as_raw()[start..start + FRAME_WIDTH * 4]
}

pub fn validate_vertical_stability(frames: &[RgbaImage]) -> Result<Vec<FrameRecord>> {
    require(
        (FRAME_WIDTH, FRAME_HEIGHT) == (192, 208),
        "fixed frame dimensions changed",
    )?;
    require(
        (GROUND_Y, GROUND_PIXEL_Y) == (192, 191),
        "canonical ground geometry changed",
    )?;
    let records = frame_records(frames)?;

    for animation in GROUNDED_ANIMATIONS {
        let range = ANIMATION_RANGES
            .iter()
            .find(|range| range.name == animation)
            .expect("grounded animation allocation");
        for record in &records[range.start..=range.end] {
            require(
                record.bounds.bottom() == GROUND_PIXEL_Y,
                format!(
                    "grounded frame {} ({animation}) ends at {}, expected {GROUND_PIXEL_Y}",
                    record.index,
                    record.bounds.bottom()
                ),
            )?;
        }
    }

    for index in JUMP_GROUNDED_FRAMES {
        require(
            records[index].bounds.bottom() == GROUND_PIXEL_Y,
            format!("jump frame {index} does not return to baseline"),
        )?;
    }
    let jumping = ANIMATION_RANGES
        .iter()
        .find(|range| range.name == "jumping")
        .expect("jump allocation");
    for (index, record) in records
        .iter()
        .enumerate()
        .take(jumping.end - 1)
        .skip(jumping.start + 2)
    {
        require(
            record.bounds.bottom() < GROUND_PIXEL_Y,
            format!("jump frame {index} does not leave baseline"),
        )?;
    }

    let idle = ANIMATION_RANGES[0];
    let reference_bounds = records[idle.start].bounds;
    let reference_baseline = baseline_bytes(&frames[idle.start]);
    for index in idle.start + 1..=idle.end {
        require(
            records[index].bounds == reference_bounds,
            format!("idle frame {index} changes scale or placement"),
        )?;
        require(
            baseline_bytes(&frames[index]) == reference_baseline,
            format!("idle frame {index} changes baseline pixels"),
        )?;
    }
    Ok(records)
}

pub fn validate_fixed_placement_sources(project: &Path) -> Result<()> {
    let sheet_source_path = project.join("src/sheet.rs");
    let artwork_source_path = project.join("src/artwork.rs");
    let maps_source_path = project.join("src/maps.rs");
    for path in [&sheet_source_path, &artwork_source_path, &maps_source_path] {
        require(
            path.is_file(),
            format!("missing pure-Rust infrastructure: {}", path.display()),
        )?;
    }

    let sheet_source = fs::read_to_string(&sheet_source_path)?;
    for forbidden in [
        "resize(",
        "thumbnail(",
        "crop_imm(",
        "scale_to_fit",
        "content_bounds",
        "visible_bounds",
    ] {
        require(
            !sheet_source.contains(forbidden),
            format!("fixed-cell packer contains content normalization: {forbidden}"),
        )?;
    }
    require(
        sheet_source.contains("copy_from_slice"),
        "fixed-cell packer must copy complete rows directly into fixed cells",
    )?;

    let artwork_source = fs::read_to_string(&artwork_source_path)?;
    for forbidden in [
        "polygon",
        "ellipse",
        "supersampl",
        "antialias",
        "scale_to_fit",
    ] {
        require(
            !artwork_source.to_ascii_lowercase().contains(forbidden),
            format!("artwork renderer contains forbidden construction: {forbidden}"),
        )?;
    }
    load_maps_source(&maps_source_path)?;
    Ok(())
}

pub fn validate_project(project: &Path, check_sources: bool) -> Result<()> {
    let manifest_path = project.join("pet.json");
    let sheet_path = project.join("spritesheet.webp");
    let manifest_exists = manifest_path.is_file();
    let sheet_exists = sheet_path.is_file();
    if !manifest_exists && !sheet_exists {
        return Err(WhiteCatError::new(
            "No White Cat runtime assets have been built",
        ));
    }
    if !manifest_exists || !sheet_exists {
        let missing = if manifest_exists {
            "spritesheet.webp"
        } else {
            "pet.json"
        };
        return Err(WhiteCatError::new(format!(
            "White Cat runtime build is incomplete: missing {missing}"
        )));
    }

    let manifest = load_manifest(&manifest_path)?;
    validate_manifest(&manifest)?;
    validate_webp_container(&sheet_path)?;
    let sheet = load_sheet(&sheet_path)?;
    let frames = extract_fixed_frames(&sheet)?;
    validate_frames(&frames)?;
    validate_vertical_stability(&frames)?;

    if check_sources {
        validate_fixed_placement_sources(project)?;
        let expected = build_frames()?;
        require(
            frames == expected,
            "checked-in spritesheet differs from the authored Rust pixel maps",
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::manifest::{build_manifest, write_manifest};
    use crate::sheet::{pack_fixed_frames, save_lossless_sheet};

    fn fixture_frames() -> Vec<RgbaImage> {
        let jumping = ANIMATION_RANGES[4];
        (0..FRAME_COUNT)
            .map(|index| {
                let mut frame = RgbaImage::new(FRAME_WIDTH as u32, FRAME_HEIGHT as u32);
                let mut bottom = GROUND_PIXEL_Y;
                if jumping.start + 2 <= index && index < jumping.end - 1 {
                    bottom -= 30;
                }
                for y in bottom - 23..=bottom {
                    for x in 80..=111 {
                        frame.put_pixel(x as u32, y as u32, image::Rgba([255, 255, 255, 255]));
                    }
                }
                frame
            })
            .collect()
    }

    fn fixture_project() -> TempDir {
        let directory = tempfile::tempdir().unwrap();
        write_manifest(
            &build_manifest("Geometry-only test fixture"),
            &directory.path().join("pet.json"),
        )
        .unwrap();
        let sheet = pack_fixed_frames(&fixture_frames()).unwrap();
        save_lossless_sheet(&sheet, &directory.path().join("spritesheet.webp")).unwrap();
        directory
    }

    #[test]
    fn temporary_geometry_fixture_validates() {
        let fixture = fixture_project();
        validate_project(fixture.path(), false).unwrap();
    }

    #[test]
    fn missing_runtime_has_a_clear_error() {
        let fixture = tempfile::tempdir().unwrap();
        assert_eq!(
            validate_project(fixture.path(), false)
                .unwrap_err()
                .to_string(),
            "No White Cat runtime assets have been built"
        );
    }

    #[test]
    fn geometry_mismatch_is_rejected() {
        let fixture = fixture_project();
        let path = fixture.path().join("pet.json");
        let mut manifest = load_manifest(&path).unwrap();
        manifest.frame.height -= 1;
        write_manifest(&manifest, &path).unwrap();
        assert!(
            validate_project(fixture.path(), false)
                .unwrap_err()
                .to_string()
                .contains("frame geometry")
        );
    }

    #[test]
    fn every_grounded_frame_uses_the_canonical_baseline() {
        let frames = fixture_frames();
        let records = validate_vertical_stability(&frames).unwrap();
        for animation in GROUNDED_ANIMATIONS {
            let range = ANIMATION_RANGES
                .iter()
                .find(|range| range.name == animation)
                .unwrap();
            assert!(
                records[range.start..=range.end]
                    .iter()
                    .all(|record| record.bounds.bottom() == GROUND_PIXEL_Y)
            );
        }
    }

    #[test]
    fn source_audit_rejects_content_resizing() {
        let project = tempfile::tempdir().unwrap();
        let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let target_root = project.path().join("src");
        fs::create_dir(&target_root).unwrap();
        for name in ["sheet.rs", "artwork.rs", "maps.rs"] {
            fs::copy(source_root.join(name), target_root.join(name)).unwrap();
        }
        let sheet_path = target_root.join("sheet.rs");
        let mut source = fs::read_to_string(&sheet_path).unwrap();
        source.push_str("\nfn forbidden() { resize(); }\n");
        fs::write(&sheet_path, source).unwrap();
        assert!(
            validate_fixed_placement_sources(project.path())
                .unwrap_err()
                .to_string()
                .contains("content normalization")
        );
    }
}

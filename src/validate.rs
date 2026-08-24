use std::collections::{HashSet, VecDeque};
use std::fs;
use std::path::Path;

use image::RgbaImage;

use crate::artwork::{
    self, DARK_REVIEW_FILE, EXACT_REVIEW_FILE, LARGE_REVIEW_HEIGHT, LARGE_REVIEW_WIDTH,
    LIGHT_REVIEW_FILE, REVIEW_DIRECTORY, SILHOUETTE_REVIEW_FILE, SOURCE_REVIEW_FILE,
};
use crate::contract::{
    ALIASES, EXACT_REVIEW_HEIGHT, EXACT_REVIEW_WIDTH, FRAME_COUNT, FRAME_HEIGHT, FRAME_MARGIN,
    FRAME_WIDTH, GRID_COLUMNS, GRID_ROWS, LAST_PLANTED_Y, MANIFEST_FILE, PET_ID, SHEET_FILE,
    SHEET_HEIGHT, SHEET_WIDTH, STATES, state_named,
};
use crate::error::{Result, fail};
use crate::kitten::{self, EYE, FUR, OUTLINE, SHADOW};
use crate::manifest::{self, PetManifest};
use crate::sheet;

fn ensure(condition: bool, message: impl Into<String>) -> Result<()> {
    if condition { Ok(()) } else { fail(message) }
}

fn alpha_bounds(image: &RgbaImage) -> Option<(u32, u32, u32, u32)> {
    let mut min_x = image.width();
    let mut min_y = image.height();
    let mut max_x = 0;
    let mut max_y = 0;
    let mut found = false;
    for (x, y, pixel) in image.enumerate_pixels() {
        if pixel[3] > 8 {
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
            found = true;
        }
    }
    found.then_some((min_x, min_y, max_x, max_y))
}

fn color_distance(pixel: &[u8; 4], color: [u8; 4]) -> u32 {
    (0..3)
        .map(|channel| {
            let difference = i32::from(pixel[channel]) - i32::from(color[channel]);
            (difference * difference) as u32
        })
        .sum()
}

fn validate_concept_cell_centers(frame: &RgbaImage) -> Result<()> {
    for logical_y in 0..kitten::LOGICAL_HEIGHT {
        for logical_x in 0..kitten::LOGICAL_WIDTH {
            let symbol = kitten::canonical_symbol(logical_x, logical_y)
                .ok_or_else(|| crate::error::WhiteCatError::new("canonical map is incomplete"))?;
            let expected = kitten::palette_color(symbol).ok_or_else(|| {
                crate::error::WhiteCatError::new(format!(
                    "canonical map uses unknown symbol {symbol:?}"
                ))
            })?;
            let x = logical_x as u32 * kitten::RUNTIME_PIXEL_SIZE + kitten::RUNTIME_PIXEL_SIZE / 2;
            let y = logical_y as u32 * kitten::RUNTIME_PIXEL_SIZE + kitten::RUNTIME_PIXEL_SIZE / 2;
            let actual = frame.get_pixel(x, y).0;
            ensure(
                actual == expected,
                format!(
                    "concept cell ({logical_x},{logical_y}) {symbol:?} rendered as {actual:?}, expected {expected:?}"
                ),
            )?;
        }
    }
    Ok(())
}

fn validate_connected_silhouette(frame: &RgbaImage) -> Result<()> {
    let occupied: HashSet<(u32, u32)> = frame
        .enumerate_pixels()
        .filter(|(_, _, pixel)| pixel[3] >= 32)
        .map(|(x, y, _)| (x, y))
        .collect();
    let start = *occupied
        .iter()
        .next()
        .ok_or_else(|| crate::error::WhiteCatError::new("canonical frame is empty"))?;
    let mut reached = HashSet::new();
    let mut queue = VecDeque::from([start]);
    while let Some((x, y)) = queue.pop_front() {
        if !occupied.contains(&(x, y)) || !reached.insert((x, y)) {
            continue;
        }
        if x > 0 {
            queue.push_back((x - 1, y));
        }
        if x + 1 < frame.width() {
            queue.push_back((x + 1, y));
        }
        if y > 0 {
            queue.push_back((x, y - 1));
        }
        if y + 1 < frame.height() {
            queue.push_back((x, y + 1));
        }
    }
    ensure(
        reached.len() == occupied.len(),
        format!(
            "silhouette has disconnected occupancy: reached {} of {} pixels",
            reached.len(),
            occupied.len()
        ),
    )
}

pub fn validate_frame(frame: &RgbaImage) -> Result<()> {
    ensure(
        frame.dimensions() == (FRAME_WIDTH, FRAME_HEIGHT),
        format!(
            "canonical frame is {}x{}, expected {FRAME_WIDTH}x{FRAME_HEIGHT}",
            frame.width(),
            frame.height()
        ),
    )?;
    let (min_x, min_y, max_x, max_y) = alpha_bounds(frame)
        .ok_or_else(|| crate::error::WhiteCatError::new("canonical frame is empty"))?;
    ensure(
        min_x >= FRAME_MARGIN && min_y >= FRAME_MARGIN,
        format!("frame begins at ({min_x},{min_y}), inside the {FRAME_MARGIN}px guard"),
    )?;
    ensure(
        max_x + FRAME_MARGIN < FRAME_WIDTH,
        format!("frame ends at x={max_x}, outside the production guard"),
    )?;
    ensure(
        max_y == LAST_PLANTED_Y,
        format!("last planted pixel is y={max_y}, expected {LAST_PLANTED_Y}"),
    )?;
    ensure(
        (176..=182).contains(&(max_x - min_x + 1)),
        format!(
            "concept silhouette width {} is outside the faithful pixel range",
            max_x - min_x + 1
        ),
    )?;
    ensure(
        (176..=182).contains(&(max_y - min_y + 1)),
        format!(
            "concept silhouette height {} is outside the faithful pixel range",
            max_y - min_y + 1
        ),
    )?;
    ensure(
        frame
            .enumerate_pixels()
            .all(|(_, y, pixel)| y <= LAST_PLANTED_Y || pixel[3] == 0),
        "pixels extend below the grounded boundary",
    )?;
    ensure(
        frame
            .enumerate_pixels()
            .filter(|(_, _, pixel)| pixel[3] > 0 && pixel[3] < 255)
            .count()
            > 100,
        "canonical frame has no meaningful antialiased edge coverage",
    )?;
    let eye_pixels = frame
        .pixels()
        .filter(|pixel| pixel[3] > 180 && color_distance(&pixel.0, EYE) < 500)
        .count();
    ensure(
        (40..=90).contains(&eye_pixels),
        format!("single concept eye area is {eye_pixels} pixels"),
    )?;
    for (name, color, minimum) in [
        ("outline", OUTLINE, 500usize),
        ("fur", FUR, 4_000usize),
        ("shadow", SHADOW, 100usize),
    ] {
        let count = frame
            .pixels()
            .filter(|pixel| pixel[3] > 180 && color_distance(&pixel.0, color) < 500)
            .count();
        ensure(
            count >= minimum,
            format!("concept {name} color has only {count} anchored pixels"),
        )?;
    }
    ensure(
        (0..FRAME_WIDTH)
            .filter(|x| frame.get_pixel(*x, LAST_PLANTED_Y)[3] > 8)
            .count()
            >= 100,
        "ground contact is too narrow",
    )?;
    validate_concept_cell_centers(frame)?;
    validate_connected_silhouette(frame)
}

pub fn validate_manifest(manifest: &PetManifest) -> Result<()> {
    ensure(
        manifest.id == PET_ID,
        format!("manifest id is {}", manifest.id),
    )?;
    ensure(
        manifest.spritesheet_path == SHEET_FILE,
        "manifest spritesheetPath is not spritesheet.webp",
    )?;
    ensure(
        manifest.frame.width == FRAME_WIDTH
            && manifest.frame.height == FRAME_HEIGHT
            && manifest.frame.columns == GRID_COLUMNS
            && manifest.frame.rows == GRID_ROWS,
        "manifest frame geometry does not match the 192x208, 8x9 contract",
    )?;
    ensure(
        manifest.frame_allocation.len() == STATES.len(),
        "manifest frame allocation count is unstable",
    )?;
    for state in STATES {
        let allocation = manifest.frame_allocation.get(state.name).ok_or_else(|| {
            crate::error::WhiteCatError::new(format!("missing state {}", state.name))
        })?;
        ensure(
            allocation.start == state.start && allocation.end == state.end,
            format!("state {} has incorrect allocation", state.name),
        )?;
        let animation = manifest.animations.get(state.name).ok_or_else(|| {
            crate::error::WhiteCatError::new(format!("missing animation {}", state.name))
        })?;
        ensure(
            animation.frames == [state.start]
                && animation.fps == 1
                && animation.loops == state.loops
                && animation.fallback == "idle",
            format!("state {} is not an explicit static held pose", state.name),
        )?;
    }
    for (alias, target) in ALIASES {
        let expected = state_named(target).expect("known alias target");
        let animation = manifest.animations.get(alias).ok_or_else(|| {
            crate::error::WhiteCatError::new(format!("missing alias animation {alias}"))
        })?;
        ensure(
            animation.frames == [expected.start]
                && animation.fps == 1
                && animation.fallback == "idle",
            format!("alias {alias} does not hold the {target} pose"),
        )?;
    }
    ensure(
        manifest.animations.len() == STATES.len() + ALIASES.len(),
        "manifest contains an unexpected animation state",
    )
}

fn validate_reviews(project: &Path, frame: &RgbaImage) -> Result<()> {
    let review = project.join(REVIEW_DIRECTORY);
    let dark = image::open(review.join(DARK_REVIEW_FILE))?.to_rgba8();
    let light = image::open(review.join(LIGHT_REVIEW_FILE))?.to_rgba8();
    let exact = image::open(review.join(EXACT_REVIEW_FILE))?.to_rgba8();
    let source = image::open(review.join(SOURCE_REVIEW_FILE))?.to_rgba8();
    let silhouette = image::open(review.join(SILHOUETTE_REVIEW_FILE))?.to_rgba8();

    ensure(
        dark.dimensions() == (LARGE_REVIEW_WIDTH, LARGE_REVIEW_HEIGHT)
            && light.dimensions() == (LARGE_REVIEW_WIDTH, LARGE_REVIEW_HEIGHT),
        "dark/light prompt reviews have incorrect dimensions",
    )?;
    ensure(
        exact.dimensions() == (EXACT_REVIEW_WIDTH, EXACT_REVIEW_HEIGHT),
        "70x15 review does not have exact terminal-cell geometry",
    )?;
    ensure(
        source.dimensions() == (kitten::SOURCE_WIDTH, kitten::SOURCE_HEIGHT),
        "source review is not the canonical 4x render",
    )?;
    ensure(
        silhouette.dimensions() == source.dimensions(),
        "silhouette review does not match source dimensions",
    )?;
    ensure(
        source.as_raw() == kitten::render_source().as_raw(),
        "source review differs from the canonical authored render",
    )?;
    ensure(
        silhouette.as_raw() == kitten::render_silhouette_source().as_raw(),
        "silhouette review differs from the canonical occupancy mask",
    )?;
    ensure(
        dark.as_raw() == artwork::dark_review(frame).as_raw()
            && light.as_raw() == artwork::light_review(frame).as_raw()
            && exact.as_raw() == artwork::exact_70x15_review(frame).as_raw(),
        "prompt review artifacts are stale or nondeterministic",
    )?;
    Ok(())
}

fn validate_sources(project: &Path) -> Result<()> {
    ensure(
        project.join("src/kitten.rs").is_file(),
        "canonical art source src/kitten.rs is missing",
    )?;
    ensure(
        !project.join("src/maps.rs").exists(),
        "rejected map audition remains connected to production",
    )?;
    let reference_path = project.join(kitten::CONCEPT_REFERENCE_FILE);
    ensure(
        reference_path.is_file(),
        format!(
            "approved concept reference is missing: {}",
            reference_path.display()
        ),
    )?;
    let reference = image::open(&reference_path)?;
    ensure(
        reference.width() == 1448 && reference.height() == 1086,
        format!(
            "approved concept reference is {}x{}, expected 1448x1086",
            reference.width(),
            reference.height()
        ),
    )?;
    let source = fs::read_to_string(project.join("src/kitten.rs"))?;
    ensure(
        source.contains("SOURCE_SCALE: u32 = 4")
            && source.contains("Lanczos3")
            && source.contains("pub const CANONICAL_MAP")
            && source.contains(kitten::CONCEPT_REFERENCE_SHA256)
            && !source.contains("smooth_shape")
            && !source.contains("CubicCurve"),
        "canonical source does not declare the concept-faithful pixel pipeline",
    )
}

pub fn validate_project(project: &Path, check_sources: bool) -> Result<()> {
    ensure(
        project.join(MANIFEST_FILE).is_file(),
        format!("{} is missing", project.join(MANIFEST_FILE).display()),
    )?;
    ensure(
        project.join(SHEET_FILE).is_file(),
        format!("{} is missing", project.join(SHEET_FILE).display()),
    )?;
    let manifest = manifest::read_manifest(project)?;
    validate_manifest(&manifest)?;
    sheet::validate_lossless_static_webp(&project.join(SHEET_FILE))?;
    let packed = sheet::load_rgba(&project.join(SHEET_FILE))?;
    ensure(
        packed.dimensions() == (SHEET_WIDTH, SHEET_HEIGHT),
        format!(
            "sheet is {}x{}, expected {SHEET_WIDTH}x{SHEET_HEIGHT}",
            packed.width(),
            packed.height()
        ),
    )?;
    let canonical = kitten::render_frame();
    validate_frame(&canonical)?;
    for index in 0..FRAME_COUNT {
        let frame = sheet::extract_frame(&packed, index)?;
        ensure(
            frame.as_raw() == canonical.as_raw(),
            format!("runtime frame {index} is not the canonical held pose"),
        )?;
    }
    if check_sources {
        validate_sources(project)?;
        validate_reviews(project, &canonical)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_frame_meets_production_geometry() {
        validate_frame(&kitten::render_frame()).expect("canonical frame is production-valid");
    }

    #[test]
    fn canonical_manifest_is_explicit_and_static() {
        validate_manifest(&manifest::build_manifest()).expect("manifest is valid");
    }

    #[test]
    fn full_sheet_is_one_honest_held_pose() {
        let frame = kitten::render_frame();
        let packed = sheet::pack_fixed_frames(&vec![frame.clone(); FRAME_COUNT]).expect("pack");
        for index in 0..FRAME_COUNT {
            assert_eq!(
                sheet::extract_frame(&packed, index)
                    .expect("extract")
                    .as_raw(),
                frame.as_raw()
            );
        }
    }
}

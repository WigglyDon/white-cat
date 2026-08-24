use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use image::RgbaImage;

use crate::artwork::{
    self, DARK_REVIEW_FILE, EXACT_REVIEW_FILE, LARGE_REVIEW_HEIGHT, LARGE_REVIEW_WIDTH,
    LIGHT_REVIEW_FILE, REVIEW_DIRECTORY, SILHOUETTE_REVIEW_FILE, SOURCE_REVIEW_FILE,
};
use crate::contract::{
    ALIASES, EXACT_REVIEW_HEIGHT, EXACT_REVIEW_WIDTH, FRAME_COUNT, FRAME_HEIGHT, FRAME_WIDTH,
    GRID_COLUMNS, GRID_ROWS, GROUND_Y, LAST_PLANTED_Y, MANIFEST_FILE, PET_ID, SHEET_FILE,
    SHEET_HEIGHT, SHEET_WIDTH, STATES, state_named,
};
use crate::digest::sha256_hex;
use crate::error::{Result, WhiteCatError, fail};
use crate::evidence;
use crate::kitten;
use crate::manifest::{self, PetManifest};
use crate::sheet;

pub const SHEET_RGBA_SHA256: &str =
    "fa797cfe00ce23809d1b22daf68b0c9faa5b2062a599af4d5a83728bbf0ee754";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ValidationReport {
    pub source_mismatches: usize,
    pub runtime_mismatches: usize,
    pub reconstructed_source_mismatches: usize,
    pub nonuniform_logical_blocks: usize,
    pub illegal_source_symbols: usize,
    pub illegal_runtime_colors: usize,
    pub alpha_violations: usize,
    pub frame_to_frame_mismatches: usize,
    pub logical_blocks_checked: usize,
    pub frames_checked: usize,
    pub normalized_matrix_sha256: String,
    pub contiguous_symbol_sha256: String,
    pub logical_rgba_sha256: String,
    pub runtime_rgba_sha256: String,
    pub sheet_rgba_sha256: String,
}

impl ValidationReport {
    pub fn summary_tsv(&self) -> String {
        format!(
            concat!(
                "check\tvalue\n",
                "source_mismatches\t{}\n",
                "runtime_mismatches\t{}\n",
                "reconstructed_source_mismatches\t{}\n",
                "nonuniform_logical_blocks\t{}\n",
                "illegal_source_symbols\t{}\n",
                "illegal_runtime_colors\t{}\n",
                "alpha_violations\t{}\n",
                "frame_to_frame_mismatches\t{}\n",
                "logical_blocks_checked\t{}\n",
                "frames_checked\t{}\n",
                "normalized_matrix_sha256\t{}\n",
                "contiguous_symbol_sha256\t{}\n",
                "logical_rgba_sha256\t{}\n",
                "runtime_rgba_sha256\t{}\n",
                "sheet_rgba_sha256\t{}\n"
            ),
            self.source_mismatches,
            self.runtime_mismatches,
            self.reconstructed_source_mismatches,
            self.nonuniform_logical_blocks,
            self.illegal_source_symbols,
            self.illegal_runtime_colors,
            self.alpha_violations,
            self.frame_to_frame_mismatches,
            self.logical_blocks_checked,
            self.frames_checked,
            self.normalized_matrix_sha256,
            self.contiguous_symbol_sha256,
            self.logical_rgba_sha256,
            self.runtime_rgba_sha256,
            self.sheet_rgba_sha256,
        )
    }
}

fn ensure(condition: bool, message: impl Into<String>) -> Result<()> {
    if condition { Ok(()) } else { fail(message) }
}

pub fn rgba_hex(color: [u8; 4]) -> String {
    format!(
        "#{:02X}{:02X}{:02X}{:02X}",
        color[0], color[1], color[2], color[3]
    )
}

fn occupied_bounds(image: &RgbaImage) -> Option<(u32, u32, u32, u32)> {
    let mut bounds: Option<(u32, u32, u32, u32)> = None;
    for (x, y, pixel) in image.enumerate_pixels() {
        if pixel[3] == 0 {
            continue;
        }
        bounds = Some(match bounds {
            None => (x, y, x, y),
            Some((min_x, min_y, max_x, max_y)) => {
                (min_x.min(x), min_y.min(y), max_x.max(x), max_y.max(y))
            }
        });
    }
    bounds
}

fn fail_with_details(label: &str, count: usize, details: &[String]) -> Result<()> {
    if count == 0 {
        return Ok(());
    }
    let mut message = format!("{label}: mismatch count {count}");
    for detail in details {
        message.push('\n');
        message.push_str(detail);
    }
    fail(message)
}

fn validate_source_contract(report: &mut ValidationReport) -> Result<()> {
    ensure(
        kitten::CANONICAL_MAP.len() == kitten::LOGICAL_HEIGHT,
        format!(
            "source row count is {}, expected {}",
            kitten::CANONICAL_MAP.len(),
            kitten::LOGICAL_HEIGHT
        ),
    )?;

    let mut structural_details = Vec::new();
    let mut counts: BTreeMap<char, usize> = BTreeMap::new();
    for (y, row) in kitten::CANONICAL_MAP.iter().enumerate() {
        if row.len() != kitten::LOGICAL_WIDTH {
            structural_details.push(format!(
                "logical row {y} width is {}, expected {}",
                row.len(),
                kitten::LOGICAL_WIDTH
            ));
        }
        for (x, symbol) in row.chars().enumerate() {
            if kitten::palette_color(symbol).is_none() {
                report.illegal_source_symbols += 1;
                structural_details.push(format!(
                    "illegal source symbol {symbol:?} at logical ({x},{y})"
                ));
            } else {
                *counts.entry(symbol).or_default() += 1;
            }
        }
    }
    fail_with_details(
        "canonical source structure",
        structural_details.len(),
        &structural_details,
    )?;

    for (symbol, expected) in kitten::EXPECTED_SYMBOL_COUNTS {
        let actual = counts.get(&symbol).copied().unwrap_or(0);
        ensure(
            actual == expected,
            format!("source symbol {symbol:?} count is {actual}, expected {expected}"),
        )?;
    }
    ensure(
        counts.values().sum::<usize>() == kitten::LOGICAL_WIDTH * kitten::LOGICAL_HEIGHT,
        "source symbol total is not 624",
    )?;

    let normalized = kitten::normalized_matrix_text();
    let contiguous = kitten::contiguous_symbols();
    ensure(
        normalized.len() == kitten::NORMALIZED_MATRIX_BYTES,
        format!(
            "normalized matrix is {} bytes, expected {}",
            normalized.len(),
            kitten::NORMALIZED_MATRIX_BYTES
        ),
    )?;
    ensure(
        contiguous.len() == kitten::LOGICAL_WIDTH * kitten::LOGICAL_HEIGHT,
        "contiguous symbol payload is not 624 bytes",
    )?;
    report.normalized_matrix_sha256 = sha256_hex(normalized.as_bytes());
    report.contiguous_symbol_sha256 = sha256_hex(&contiguous);
    ensure(
        report.normalized_matrix_sha256 == kitten::NORMALIZED_MATRIX_SHA256,
        format!(
            "normalized matrix SHA-256 is {}, expected {}",
            report.normalized_matrix_sha256,
            kitten::NORMALIZED_MATRIX_SHA256
        ),
    )?;
    ensure(
        report.contiguous_symbol_sha256 == kitten::CONTIGUOUS_SYMBOL_SHA256,
        format!(
            "contiguous symbol SHA-256 is {}, expected {}",
            report.contiguous_symbol_sha256,
            kitten::CONTIGUOUS_SYMBOL_SHA256
        ),
    )?;

    let logical = kitten::render_logical();
    ensure(
        logical.dimensions() == (kitten::LOGICAL_WIDTH as u32, kitten::LOGICAL_HEIGHT as u32),
        "logical RGBA image is not 24x26",
    )?;
    ensure(
        logical.as_raw().len() == kitten::LOGICAL_RGBA_BYTES,
        "logical RGBA payload is not 2496 bytes",
    )?;
    let mut mismatch_details = Vec::new();
    for (x, y, actual) in logical.enumerate_pixels() {
        let symbol = kitten::canonical_symbol(x as usize, y as usize)
            .ok_or_else(|| WhiteCatError::new(format!("missing source at logical ({x},{y})")))?;
        let expected = kitten::palette_color(symbol).expect("validated source symbol");
        if actual.0 != expected {
            report.source_mismatches += 1;
            mismatch_details.push(format!(
                "logical ({x},{y}) expected {} actual {}",
                rgba_hex(expected),
                rgba_hex(actual.0)
            ));
        }
        if !matches!(actual[3], 0 | 255) || (actual[3] == 0 && actual.0[..3] != [0, 0, 0]) {
            report.alpha_violations += 1;
            mismatch_details.push(format!(
                "logical alpha violation at ({x},{y}) actual {}",
                rgba_hex(actual.0)
            ));
        }
    }
    fail_with_details(
        "logical source identity",
        report.source_mismatches + report.alpha_violations,
        &mismatch_details,
    )?;
    report.logical_rgba_sha256 = sha256_hex(logical.as_raw());
    ensure(
        report.logical_rgba_sha256 == kitten::LOGICAL_RGBA_SHA256,
        format!(
            "logical RGBA SHA-256 is {}, expected {}",
            report.logical_rgba_sha256,
            kitten::LOGICAL_RGBA_SHA256
        ),
    )?;
    ensure(
        occupied_bounds(&logical) == Some((0, 1, 23, 24)),
        format!(
            "logical occupied bounds are {:?}, expected (0,1)-(23,24)",
            occupied_bounds(&logical)
        ),
    )?;
    ensure(
        kitten::CANONICAL_MAP[25] == "........................",
        "logical row 25 is not entirely transparent",
    )
}

pub fn validate_frame(frame: &RgbaImage) -> Result<ValidationReport> {
    let mut report = ValidationReport::default();
    validate_source_contract(&mut report)?;
    ensure(
        frame.dimensions() == (FRAME_WIDTH, FRAME_HEIGHT),
        format!(
            "runtime frame is {}x{}, expected {FRAME_WIDTH}x{FRAME_HEIGHT}",
            frame.width(),
            frame.height()
        ),
    )?;

    let mut details = Vec::new();
    let mut runtime_counts: BTreeMap<[u8; 4], usize> = BTreeMap::new();
    for (x, y, actual) in frame.enumerate_pixels() {
        *runtime_counts.entry(actual.0).or_default() += 1;
        if !kitten::PALETTE.iter().any(|(_, color)| color == &actual.0) {
            report.illegal_runtime_colors += 1;
            details.push(format!(
                "illegal runtime color at ({x},{y}) actual {}",
                rgba_hex(actual.0)
            ));
        }
        if !matches!(actual[3], 0 | 255) || (actual[3] == 0 && actual.0[..3] != [0, 0, 0]) {
            report.alpha_violations += 1;
            details.push(format!(
                "runtime alpha violation at ({x},{y}) actual {}",
                rgba_hex(actual.0)
            ));
        }
        let logical_x = (x / kitten::RUNTIME_PIXEL_SIZE) as usize;
        let logical_y = (y / kitten::RUNTIME_PIXEL_SIZE) as usize;
        let symbol = kitten::canonical_symbol(logical_x, logical_y)
            .expect("runtime coordinate maps inside canonical matrix");
        let expected = kitten::palette_color(symbol).expect("validated source symbol");
        if actual.0 != expected {
            report.runtime_mismatches += 1;
            details.push(format!(
                "runtime ({x},{y}) logical ({logical_x},{logical_y}) expected {} actual {}",
                rgba_hex(expected),
                rgba_hex(actual.0)
            ));
        }
    }

    for logical_y in 0..kitten::LOGICAL_HEIGHT {
        for logical_x in 0..kitten::LOGICAL_WIDTH {
            report.logical_blocks_checked += 1;
            let origin_x = logical_x as u32 * kitten::RUNTIME_PIXEL_SIZE;
            let origin_y = logical_y as u32 * kitten::RUNTIME_PIXEL_SIZE;
            let first = frame.get_pixel(origin_x, origin_y).0;
            let symbol =
                kitten::canonical_symbol(logical_x, logical_y).expect("complete canonical matrix");
            let expected = kitten::palette_color(symbol).expect("validated source symbol");
            let mut block_mismatches = 0usize;
            for dy in 0..kitten::RUNTIME_PIXEL_SIZE {
                for dx in 0..kitten::RUNTIME_PIXEL_SIZE {
                    if frame.get_pixel(origin_x + dx, origin_y + dy).0 != first {
                        block_mismatches += 1;
                    }
                }
            }
            if block_mismatches != 0 {
                report.nonuniform_logical_blocks += 1;
                details.push(format!(
                    "nonuniform logical block ({logical_x},{logical_y}) runtime rectangle ({origin_x},{origin_y})-({},{}) mismatch count {block_mismatches}",
                    origin_x + kitten::RUNTIME_PIXEL_SIZE - 1,
                    origin_y + kitten::RUNTIME_PIXEL_SIZE - 1,
                ));
            }
            if first != expected {
                report.reconstructed_source_mismatches += 1;
                details.push(format!(
                    "reconstructed logical ({logical_x},{logical_y}) runtime rectangle ({origin_x},{origin_y})-({},{}) expected {} actual {}",
                    origin_x + kitten::RUNTIME_PIXEL_SIZE - 1,
                    origin_y + kitten::RUNTIME_PIXEL_SIZE - 1,
                    rgba_hex(expected),
                    rgba_hex(first),
                ));
            }
        }
    }
    fail_with_details(
        "runtime identity",
        report.runtime_mismatches
            + report.reconstructed_source_mismatches
            + report.nonuniform_logical_blocks
            + report.illegal_runtime_colors
            + report.alpha_violations,
        &details,
    )?;

    for (symbol, logical_count) in kitten::EXPECTED_SYMBOL_COUNTS {
        let color = kitten::palette_color(symbol).expect("known palette symbol");
        let expected = logical_count * (kitten::RUNTIME_PIXEL_SIZE as usize).pow(2);
        let actual = runtime_counts.get(&color).copied().unwrap_or(0);
        ensure(
            actual == expected,
            format!(
                "runtime color {} count is {actual}, expected {expected}",
                rgba_hex(color)
            ),
        )?;
    }
    ensure(
        runtime_counts.values().sum::<usize>() == (FRAME_WIDTH * FRAME_HEIGHT) as usize,
        "runtime pixel total is not 39936",
    )?;
    ensure(
        occupied_bounds(frame) == Some((0, 8, 191, 199)),
        format!(
            "runtime occupied bounds are {:?}, expected (0,8)-(191,199)",
            occupied_bounds(frame)
        ),
    )?;
    ensure(
        frame
            .enumerate_pixels()
            .all(|(_, y, pixel)| y < GROUND_Y || pixel.0 == kitten::TRANSPARENT),
        format!("runtime pixels extend to or below GROUND_Y={GROUND_Y}"),
    )?;
    ensure(
        LAST_PLANTED_Y == 199,
        format!("last planted y is {LAST_PLANTED_Y}, expected 199"),
    )?;
    report.runtime_rgba_sha256 = sha256_hex(frame.as_raw());
    ensure(
        report.runtime_rgba_sha256 == kitten::RUNTIME_RGBA_SHA256,
        format!(
            "runtime RGBA SHA-256 is {}, expected {}",
            report.runtime_rgba_sha256,
            kitten::RUNTIME_RGBA_SHA256
        ),
    )?;
    Ok(report)
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
        let allocation = manifest
            .frame_allocation
            .get(state.name)
            .ok_or_else(|| WhiteCatError::new(format!("missing state {}", state.name)))?;
        ensure(
            allocation.start == state.start && allocation.end == state.end,
            format!("state {} has incorrect allocation", state.name),
        )?;
        let animation = manifest
            .animations
            .get(state.name)
            .ok_or_else(|| WhiteCatError::new(format!("missing animation {}", state.name)))?;
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
        let animation = manifest
            .animations
            .get(alias)
            .ok_or_else(|| WhiteCatError::new(format!("missing alias animation {alias}")))?;
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
        source.dimensions() == (kitten::LOGICAL_WIDTH as u32, kitten::LOGICAL_HEIGHT as u32),
        "source review is not the exact 24x26 canonical matrix",
    )?;
    ensure(
        silhouette.dimensions() == source.dimensions(),
        "silhouette review does not match logical dimensions",
    )?;
    ensure(
        source.as_raw() == kitten::render_logical().as_raw(),
        "source review differs from the canonical matrix",
    )?;
    ensure(
        silhouette.as_raw() == kitten::render_silhouette_logical().as_raw(),
        "silhouette review differs from canonical occupancy",
    )?;
    ensure(
        dark.as_raw() == artwork::dark_review(frame).as_raw()
            && light.as_raw() == artwork::light_review(frame).as_raw()
            && exact.as_raw() == artwork::exact_70x15_review(frame).as_raw(),
        "prompt review artifacts are stale or nondeterministic",
    )
}

fn validate_sources(project: &Path) -> Result<()> {
    let kitten_source = project.join("src/kitten.rs");
    ensure(
        kitten_source.is_file(),
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
            "design-provenance file is missing: {}",
            reference_path.display()
        ),
    )?;
    ensure(
        sha256_hex(&fs::read(&reference_path)?) == kitten::CONCEPT_REFERENCE_SHA256,
        "design-provenance file hash changed",
    )?;
    let source = fs::read_to_string(&kitten_source)?;
    for forbidden in [
        "Lanczos",
        "SOURCE_SCALE",
        "SOURCE_PIXEL_SIZE",
        "premultiply",
        "unpremultiply",
        "resize_rgba",
        "FilterType",
    ] {
        ensure(
            !source.contains(forbidden),
            format!("production kitten renderer still contains forbidden path {forbidden:?}"),
        )?;
    }
    ensure(
        source.contains("pub const CANONICAL_MAP")
            && source.contains("runtime_x / RUNTIME_PIXEL_SIZE")
            && source.contains("runtime_y / RUNTIME_PIXEL_SIZE"),
        "canonical source does not declare the direct matrix-to-runtime path",
    )
}

pub fn inspect_project(project: &Path, check_sources: bool) -> Result<ValidationReport> {
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

    let report = validate_packed(&packed)?;
    if check_sources {
        validate_sources(project)?;
        validate_reviews(project, &kitten::render_frame())?;
        evidence::validate_generation_evidence(project, &packed, &report)?;
    }
    Ok(report)
}

pub fn validate_packed(packed: &RgbaImage) -> Result<ValidationReport> {
    ensure(
        packed.dimensions() == (SHEET_WIDTH, SHEET_HEIGHT),
        format!(
            "sheet is {}x{}, expected {SHEET_WIDTH}x{SHEET_HEIGHT}",
            packed.width(),
            packed.height()
        ),
    )?;
    let canonical = kitten::render_frame();
    let mut report = validate_frame(&canonical)?;
    report.sheet_rgba_sha256 = sha256_hex(packed.as_raw());
    ensure(
        report.sheet_rgba_sha256 == SHEET_RGBA_SHA256,
        format!(
            "decoded sheet RGBA SHA-256 is {}, expected {SHEET_RGBA_SHA256}",
            report.sheet_rgba_sha256
        ),
    )?;
    let mut details = Vec::new();
    for index in 0..FRAME_COUNT {
        report.frames_checked += 1;
        let frame = sheet::extract_frame(packed, index)?;
        for (x, y, actual) in frame.enumerate_pixels() {
            let expected = canonical.get_pixel(x, y);
            if actual != expected {
                report.frame_to_frame_mismatches += 1;
                details.push(format!(
                    "frame {index} runtime ({x},{y}) expected {} actual {}",
                    rgba_hex(expected.0),
                    rgba_hex(actual.0)
                ));
            }
        }
    }
    fail_with_details(
        "packed frame identity",
        report.frame_to_frame_mismatches,
        &details,
    )?;
    Ok(report)
}

pub fn validate_project(project: &Path, check_sources: bool) -> Result<()> {
    inspect_project(project, check_sources).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_frame_meets_exact_takeover_contract() {
        let report = validate_frame(&kitten::render_frame()).expect("canonical frame is exact");
        assert_eq!(report.source_mismatches, 0);
        assert_eq!(report.runtime_mismatches, 0);
        assert_eq!(report.reconstructed_source_mismatches, 0);
        assert_eq!(report.nonuniform_logical_blocks, 0);
        assert_eq!(report.illegal_source_symbols, 0);
        assert_eq!(report.illegal_runtime_colors, 0);
        assert_eq!(report.alpha_violations, 0);
        assert_eq!(report.logical_blocks_checked, 624);
    }

    #[test]
    fn canonical_manifest_is_explicit_and_static() {
        validate_manifest(&manifest::build_manifest()).expect("manifest is valid");
    }

    #[test]
    fn full_sheet_is_one_exact_held_pose() {
        let frame = kitten::render_frame();
        let packed = sheet::pack_fixed_frames(&vec![frame.clone(); FRAME_COUNT]).expect("pack");
        assert_eq!(sha256_hex(packed.as_raw()), SHEET_RGBA_SHA256);
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

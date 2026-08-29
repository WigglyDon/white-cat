use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use image::RgbaImage;

use crate::artwork::{
    self, ANIMATION_STORYBOARD_FILE, DARK_REVIEW_FILE, EXACT_REVIEW_FILE, IDLE_STRIP_FILE,
    LARGE_REVIEW_HEIGHT, LARGE_REVIEW_WIDTH, LIGHT_REVIEW_FILE, REVIEW_DIRECTORY,
    SILHOUETTE_REVIEW_FILE, SOURCE_REVIEW_FILE,
};
use crate::contract::{
    ALIASES, EXACT_REVIEW_HEIGHT, EXACT_REVIEW_WIDTH, FRAME_COUNT, FRAME_HEIGHT, FRAME_WIDTH,
    GRID_COLUMNS, GRID_ROWS, GROUND_Y, LAST_PLANTED_Y, MANIFEST_FILE, PET_ID, SHEET_FILE,
    SHEET_HEIGHT, SHEET_WIDTH, STATES, animation_fps, animation_timeline, state_named,
};
use crate::digest::sha256_hex;
use crate::error::{Result, WhiteCatError, fail};
use crate::evidence;
use crate::kitten::{self, PixelMap, PoseId};
use crate::manifest::{self, PetManifest};
use crate::sheet;

pub const SHEET_RGBA_SHA256: &str =
    "00359ee381c087a36f9100433b01ec9fbb2a52c567deb72c206418e0ae6e80dc";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ValidationReport {
    pub source_mismatches: usize,
    pub runtime_mismatches: usize,
    pub reconstructed_source_mismatches: usize,
    pub nonuniform_logical_blocks: usize,
    pub illegal_source_symbols: usize,
    pub illegal_runtime_colors: usize,
    pub alpha_violations: usize,
    pub frame_contract_mismatches: usize,
    pub logical_blocks_checked: usize,
    pub frames_checked: usize,
    pub distinct_frames: usize,
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
                "frame_contract_mismatches\t{}\n",
                "logical_blocks_checked\t{}\n",
                "frames_checked\t{}\n",
                "distinct_frames\t{}\n",
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
            self.frame_contract_mismatches,
            self.logical_blocks_checked,
            self.frames_checked,
            self.distinct_frames,
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
    for detail in details.iter().take(128) {
        message.push('\n');
        message.push_str(detail);
    }
    if details.len() > 128 {
        message.push_str(&format!("\n... {} more details", details.len() - 128));
    }
    fail(message)
}

fn validate_map_structure(name: &str, map: &PixelMap, report: &mut ValidationReport) -> Result<()> {
    let mut details = Vec::new();
    for (y, row) in map.iter().enumerate() {
        if row.len() != kitten::LOGICAL_WIDTH {
            details.push(format!(
                "pose {name} row {y} width is {}, expected {}",
                row.len(),
                kitten::LOGICAL_WIDTH
            ));
        }
        for (x, symbol) in row.chars().enumerate() {
            if kitten::palette_color(symbol).is_none() {
                report.illegal_source_symbols += 1;
                details.push(format!(
                    "pose {name} has illegal source symbol {symbol:?} at logical ({x},{y})"
                ));
            }
        }
    }
    fail_with_details("literal pose structure", details.len(), &details)?;
    ensure(
        map[25] == "........................",
        format!("pose {name} uses pixels below the grounded boundary"),
    )?;
    ensure(
        map.iter()
            .any(|row| row.chars().any(|symbol| symbol != '.')),
        format!("pose {name} is empty"),
    )
}

fn validate_source_contract(report: &mut ValidationReport) -> Result<()> {
    for pose in kitten::ALL_POSES {
        validate_map_structure(kitten::pose_name(pose), kitten::pose_map(pose), report)?;
    }

    let mut counts: BTreeMap<char, usize> = BTreeMap::new();
    for row in kitten::CANONICAL_MAP {
        for symbol in row.chars() {
            *counts.entry(symbol).or_default() += 1;
        }
    }
    for (symbol, expected) in kitten::EXPECTED_SYMBOL_COUNTS {
        let actual = counts.get(&symbol).copied().unwrap_or(0);
        ensure(
            actual == expected,
            format!("canonical symbol {symbol:?} count is {actual}, expected {expected}"),
        )?;
    }

    let normalized = kitten::normalized_matrix_text();
    let contiguous = kitten::contiguous_symbols();
    ensure(
        normalized.len() == kitten::NORMALIZED_MATRIX_BYTES,
        "canonical normalized matrix byte count changed",
    )?;
    ensure(
        contiguous.len() == kitten::LOGICAL_WIDTH * kitten::LOGICAL_HEIGHT,
        "canonical contiguous symbol payload is not 624 bytes",
    )?;
    report.normalized_matrix_sha256 = sha256_hex(normalized.as_bytes());
    report.contiguous_symbol_sha256 = sha256_hex(&contiguous);
    ensure(
        report.normalized_matrix_sha256 == kitten::NORMALIZED_MATRIX_SHA256,
        "canonical normalized matrix hash changed",
    )?;
    ensure(
        report.contiguous_symbol_sha256 == kitten::CONTIGUOUS_SYMBOL_SHA256,
        "canonical contiguous symbol hash changed",
    )?;

    let logical = kitten::render_logical();
    report.logical_rgba_sha256 = sha256_hex(logical.as_raw());
    ensure(
        report.logical_rgba_sha256 == kitten::LOGICAL_RGBA_SHA256,
        "canonical logical RGBA hash changed",
    )?;
    ensure(
        sha256_hex(kitten::frozen_animation_contract_text().as_bytes())
            == kitten::FROZEN_ANIMATION_CONTRACT_SHA256,
        "frozen animation pose or frame-plan hash changed",
    )?;
    ensure(
        occupied_bounds(&logical) == Some((0, 1, 23, 24)),
        format!(
            "canonical occupied bounds are {:?}, expected (0,1)-(23,24)",
            occupied_bounds(&logical)
        ),
    )?;

    ensure(
        kitten::FRAME_POSES.len() == STATES.len(),
        "pose-plan rows do not match declared runtime states",
    )?;
    ensure(
        kitten::FRAME_POSES.iter().flatten().count() == FRAME_COUNT,
        "pose plan does not fill all 72 sheet cells",
    )?;
    for (row, state) in STATES.iter().enumerate() {
        if state.name == "jumping" {
            continue;
        }
        for pose in kitten::FRAME_POSES[row] {
            ensure(
                kitten::pose_map(pose)[24]
                    .chars()
                    .any(|symbol| symbol != '.'),
                format!(
                    "grounded state {} pose {} is not planted on logical row 24",
                    state.name,
                    kitten::pose_name(pose)
                ),
            )?;
        }
    }
    ensure(
        kitten::pose_map(PoseId::JumpRise)[24] == "........................"
            && kitten::pose_map(PoseId::JumpApex)[24] == "........................",
        "airborne jump poses touch the grounded row",
    )?;
    ensure(
        LAST_PLANTED_Y == 199,
        format!("last planted y is {LAST_PLANTED_Y}, expected 199"),
    )
}

fn validate_rendered_frame(
    map: &PixelMap,
    frame: &RgbaImage,
    report: &mut ValidationReport,
) -> Result<()> {
    ensure(
        frame.dimensions() == (FRAME_WIDTH, FRAME_HEIGHT),
        format!(
            "runtime frame is {}x{}, expected {FRAME_WIDTH}x{FRAME_HEIGHT}",
            frame.width(),
            frame.height()
        ),
    )?;

    let before_runtime = report.runtime_mismatches;
    let before_reconstructed = report.reconstructed_source_mismatches;
    let before_blocks = report.nonuniform_logical_blocks;
    let before_colors = report.illegal_runtime_colors;
    let before_alpha = report.alpha_violations;
    let mut details = Vec::new();
    for logical_y in 0..kitten::LOGICAL_HEIGHT {
        for logical_x in 0..kitten::LOGICAL_WIDTH {
            report.logical_blocks_checked += 1;
            let x0 = logical_x as u32 * kitten::RUNTIME_PIXEL_SIZE;
            let y0 = logical_y as u32 * kitten::RUNTIME_PIXEL_SIZE;
            let symbol = kitten::map_symbol(map, logical_x, logical_y)
                .expect("complete validated literal pose");
            let expected = kitten::palette_color(symbol).expect("validated palette symbol");
            let origin = frame.get_pixel(x0, y0).0;
            let mut nonuniform = 0usize;
            for dy in 0..kitten::RUNTIME_PIXEL_SIZE {
                for dx in 0..kitten::RUNTIME_PIXEL_SIZE {
                    let actual = frame.get_pixel(x0 + dx, y0 + dy).0;
                    if actual != origin {
                        nonuniform += 1;
                    }
                    if actual != expected {
                        report.runtime_mismatches += 1;
                    }
                    if !kitten::PALETTE.iter().any(|(_, color)| *color == actual) {
                        report.illegal_runtime_colors += 1;
                    }
                    if !matches!(actual[3], 0 | 255) || (actual[3] == 0 && actual[..3] != [0, 0, 0])
                    {
                        report.alpha_violations += 1;
                    }
                }
            }
            if nonuniform != 0 {
                report.nonuniform_logical_blocks += 1;
                details.push(format!(
                    "logical block ({logical_x},{logical_y}) has {nonuniform} nonuniform runtime pixels"
                ));
            }
            if origin != expected {
                report.reconstructed_source_mismatches += 1;
                details.push(format!(
                    "logical ({logical_x},{logical_y}) expected {} actual {}",
                    rgba_hex(expected),
                    rgba_hex(origin)
                ));
            }
        }
    }
    ensure(
        frame
            .enumerate_pixels()
            .all(|(_, y, pixel)| y < GROUND_Y || pixel.0 == kitten::TRANSPARENT),
        format!("runtime pixels extend to or below GROUND_Y={GROUND_Y}"),
    )?;
    let new_mismatches = (report.runtime_mismatches - before_runtime)
        + (report.reconstructed_source_mismatches - before_reconstructed)
        + (report.nonuniform_logical_blocks - before_blocks)
        + (report.illegal_runtime_colors - before_colors)
        + (report.alpha_violations - before_alpha);
    fail_with_details("direct map-to-runtime expansion", new_mismatches, &details)
}

pub fn validate_frame(frame: &RgbaImage) -> Result<ValidationReport> {
    let mut report = ValidationReport::default();
    validate_source_contract(&mut report)?;
    validate_rendered_frame(&kitten::CANONICAL_MAP, frame, &mut report)?;
    report.frames_checked = 1;
    report.distinct_frames = 1;
    report.runtime_rgba_sha256 = sha256_hex(frame.as_raw());
    ensure(
        report.runtime_rgba_sha256 == kitten::RUNTIME_RGBA_SHA256,
        "canonical runtime RGBA hash changed",
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
            animation.frames == animation_timeline(state.name).expect("declared timeline")
                && animation.fps == animation_fps(state.name).expect("declared fps")
                && animation.loops == state.loops
                && animation.fallback == "idle",
            format!(
                "state {} does not match its frozen animation contract",
                state.name
            ),
        )?;
    }
    for (alias, target) in ALIASES {
        let target_state = state_named(target).expect("known alias target");
        let animation = manifest
            .animations
            .get(alias)
            .ok_or_else(|| WhiteCatError::new(format!("missing alias animation {alias}")))?;
        ensure(
            animation.frames == animation_timeline(target).expect("target timeline")
                && animation.fps == animation_fps(target).expect("target fps")
                && animation.loops == target_state.loops
                && animation.fallback == "idle",
            format!("alias {alias} does not share the {target} animation"),
        )?;
    }
    ensure(
        manifest.animations.len() == STATES.len() + ALIASES.len(),
        "manifest contains an unexpected animation state",
    )
}

fn validate_reviews(project: &Path, packed: &RgbaImage) -> Result<()> {
    let review = project.join(REVIEW_DIRECTORY);
    let dark = image::open(review.join(DARK_REVIEW_FILE))?.to_rgba8();
    let light = image::open(review.join(LIGHT_REVIEW_FILE))?.to_rgba8();
    let exact = image::open(review.join(EXACT_REVIEW_FILE))?.to_rgba8();
    let source = image::open(review.join(SOURCE_REVIEW_FILE))?.to_rgba8();
    let silhouette = image::open(review.join(SILHOUETTE_REVIEW_FILE))?.to_rgba8();
    let storyboard = image::open(review.join(ANIMATION_STORYBOARD_FILE))?.to_rgba8();
    let idle_strip = image::open(review.join(IDLE_STRIP_FILE))?.to_rgba8();
    let canonical = kitten::render_frame();
    let frames = kitten::build_frames();

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
        source.as_raw() == kitten::render_logical().as_raw(),
        "source review differs from the canonical base matrix",
    )?;
    ensure(
        silhouette.as_raw() == kitten::render_silhouette_logical().as_raw(),
        "silhouette review differs from canonical occupancy",
    )?;
    ensure(
        dark.as_raw() == artwork::dark_review(&canonical).as_raw()
            && light.as_raw() == artwork::light_review(&canonical).as_raw()
            && exact.as_raw() == artwork::exact_70x15_review(&canonical).as_raw(),
        "prompt review artifacts are stale or nondeterministic",
    )?;
    ensure(
        storyboard.as_raw() == artwork::solid_animation_storyboard(packed).as_raw(),
        "animation storyboard is stale or nondeterministic",
    )?;
    ensure(
        idle_strip.as_raw() == artwork::solid_idle_strip(&frames)?.as_raw(),
        "idle strip is stale or nondeterministic",
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
            format!("production kitten renderer contains forbidden path {forbidden:?}"),
        )?;
    }
    ensure(
        source.contains("pub const CANONICAL_MAP")
            && source.contains("pub const FRAME_POSES")
            && source.contains("runtime_x / RUNTIME_PIXEL_SIZE")
            && source.contains("runtime_y / RUNTIME_PIXEL_SIZE"),
        "canonical source does not declare the literal pose-to-runtime path",
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
    let report = validate_packed(&packed)?;
    if check_sources {
        validate_sources(project)?;
        validate_reviews(project, &packed)?;
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

    let mut report = ValidationReport::default();
    validate_source_contract(&mut report)?;
    let expected_frames = kitten::build_frames();
    ensure(
        expected_frames.len() == FRAME_COUNT,
        "animation renderer did not build 72 frames",
    )?;
    let mut distinct = BTreeSet::new();
    let mut details = Vec::new();
    for (index, expected) in expected_frames.iter().enumerate() {
        let actual = sheet::extract_frame(packed, index)?;
        let pose =
            kitten::FRAME_POSES[index / GRID_COLUMNS as usize][index % GRID_COLUMNS as usize];
        validate_rendered_frame(kitten::pose_map(pose), &actual, &mut report)?;
        let mismatches = expected
            .pixels()
            .zip(actual.pixels())
            .filter(|(left, right)| left != right)
            .count();
        if mismatches != 0 {
            report.frame_contract_mismatches += mismatches;
            details.push(format!(
                "frame {index} pose {} has {mismatches} pixel mismatches",
                kitten::pose_name(pose)
            ));
        }
        distinct.insert(sha256_hex(actual.as_raw()));
        report.frames_checked += 1;
    }
    fail_with_details(
        "packed animation-frame contract",
        report.frame_contract_mismatches,
        &details,
    )?;
    report.distinct_frames = distinct.len();
    ensure(
        report.distinct_frames == kitten::ALL_POSES.len(),
        format!(
            "sheet has {} distinct frames, expected {} frozen poses",
            report.distinct_frames,
            kitten::ALL_POSES.len()
        ),
    )?;
    report.runtime_rgba_sha256 = sha256_hex(expected_frames[0].as_raw());
    report.sheet_rgba_sha256 = sha256_hex(packed.as_raw());
    ensure(
        report.runtime_rgba_sha256 == kitten::RUNTIME_RGBA_SHA256,
        "sheet frame 0 no longer matches the canonical runtime hash",
    )?;
    ensure(
        report.sheet_rgba_sha256 == SHEET_RGBA_SHA256,
        format!(
            "decoded sheet RGBA SHA-256 is {}, expected {SHEET_RGBA_SHA256}",
            report.sheet_rgba_sha256
        ),
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
    fn canonical_frame_preserves_the_approved_base_contract() {
        let report = validate_frame(&kitten::render_frame()).expect("canonical frame is exact");
        assert_eq!(report.runtime_mismatches, 0);
        assert_eq!(report.reconstructed_source_mismatches, 0);
        assert_eq!(report.nonuniform_logical_blocks, 0);
        assert_eq!(report.illegal_source_symbols, 0);
        assert_eq!(report.illegal_runtime_colors, 0);
        assert_eq!(report.alpha_violations, 0);
        assert_eq!(report.logical_blocks_checked, 624);
    }

    #[test]
    fn animated_manifest_is_explicit_and_timed() {
        validate_manifest(&manifest::build_manifest()).expect("manifest is valid");
    }

    #[test]
    fn full_sheet_matches_all_literal_pose_cells() {
        let frames = kitten::build_frames();
        let packed = sheet::pack_fixed_frames(&frames).expect("pack");
        let report = validate_packed(&packed).expect("animated sheet is exact");
        assert_eq!(report.frames_checked, FRAME_COUNT);
        assert_eq!(report.logical_blocks_checked, FRAME_COUNT * 624);
        assert_eq!(report.distinct_frames, kitten::ALL_POSES.len());
        assert_eq!(report.frame_contract_mismatches, 0);
    }
}

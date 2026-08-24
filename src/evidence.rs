use std::fs;
use std::path::{Path, PathBuf};

use image::{Rgba, RgbaImage};

use crate::artwork;
use crate::contract::{FRAME_COUNT, FRAME_HEIGHT, FRAME_WIDTH, MANIFEST_FILE, SHEET_FILE};
use crate::digest::sha256_hex;
use crate::error::{Result, WhiteCatError, fail};
use crate::kitten;
use crate::sheet;
use crate::validate::{self, ValidationReport, rgba_hex};

pub const EVIDENCE_DIRECTORY: &str = "review/evidence";
pub const CANONICAL_MATRIX_FILE: &str = "canonical-matrix.txt";
pub const COORDINATE_MATRIX_FILE: &str = "coordinate-labelled-matrix.txt";
pub const CANONICAL_PNG_FILE: &str = "canonical-24x26.png";
pub const RUNTIME_PNG_FILE: &str = "runtime-192x208.png";
pub const DARK_INSPECTION_FILE: &str = "runtime-dark-192x208.png";
pub const LIGHT_INSPECTION_FILE: &str = "runtime-light-192x208.png";
pub const DECODED_FRAME_FILE: &str = "decoded-frame-0.png";
pub const DECODED_SHEET_FILE: &str = "decoded-sheet-1536x1872.png";
pub const PALETTE_FILE: &str = "palette-and-counts.tsv";
pub const HASH_FILE: &str = "source-runtime-sheet-hashes.tsv";
pub const BLOCK_REPORT_FILE: &str = "block-uniformity.tsv";
pub const MISMATCH_REPORT_FILE: &str = "mismatch-coordinates.tsv";
pub const VALIDATION_REPORT_FILE: &str = "validation-summary.tsv";
pub const DETERMINISM_REPORT_FILE: &str = "deterministic-generation.tsv";
pub const INSTALLED_REPORT_FILE: &str = "generated-versus-installed.tsv";
pub const OBSERVED_FRAME_FILE: &str = "fresh-codex-cache-frame.png";
pub const OBSERVED_REPORT_FILE: &str = "fresh-codex-cache-comparison.tsv";
pub const CACHE_FRAMES_REPORT_FILE: &str = "fresh-codex-cache-all-frames.tsv";
pub const COMPARISON_BOARD_FILE: &str = "canonical-runtime-capture-comparison.png";
pub const COMPARISON_BOARD_MANIFEST_FILE: &str = "comparison-board-panels.tsv";
pub const XOR_FILE: &str = "runtime-capture-xor.png";
pub const PROCESS_CAPTURE_FILE: &str = "fresh-codex-process-pty.bin";
pub const PROCESS_CAPTURE_REPORT_FILE: &str = "fresh-codex-process-pty.tsv";

const DARK_BACKGROUND: Rgba<u8> = Rgba([13, 17, 22, 255]);
const LIGHT_BACKGROUND: Rgba<u8> = Rgba([244, 241, 234, 255]);

fn write_png(path: &Path, image: &RgbaImage) -> Result<()> {
    sheet::write_atomic(path, &artwork::encode_png(image)?)
}

fn solid_inspection(frame: &RgbaImage, background: Rgba<u8>) -> RgbaImage {
    let mut image = RgbaImage::from_pixel(FRAME_WIDTH, FRAME_HEIGHT, background);
    for (target, source) in image.pixels_mut().zip(frame.pixels()) {
        if source[3] != 0 {
            *target = *source;
        }
    }
    image
}

fn coordinate_matrix() -> String {
    let mut text = String::from(
        "coordinate law: row 0 is top; column 0 is left\n    000000000011111111112222\n    012345678901234567890123\n",
    );
    for (y, row) in kitten::CANONICAL_MAP.iter().enumerate() {
        text.push_str(&format!("{y:02}  {row}\n"));
    }
    text
}

fn palette_manifest() -> String {
    let mut text = String::from("symbol\tmeaning\trgba\tlogical_pixels\truntime_pixels\n");
    let meanings = [
        ('.', "transparent"),
        ('O', "outline"),
        ('B', "body"),
        ('S', "shade"),
        ('E', "eye"),
    ];
    for (symbol, meaning) in meanings {
        let color = kitten::palette_color(symbol).expect("declared palette symbol");
        let logical = kitten::EXPECTED_SYMBOL_COUNTS
            .iter()
            .find_map(|(candidate, count)| (*candidate == symbol).then_some(*count))
            .expect("declared symbol count");
        let runtime = logical * (kitten::RUNTIME_PIXEL_SIZE as usize).pow(2);
        text.push_str(&format!(
            "{symbol}\t{meaning}\t{}\t{logical}\t{runtime}\n",
            rgba_hex(color)
        ));
    }
    text.push_str("TOTAL\tall\tNOT_APPLICABLE\t624\t39936\n");
    text
}

fn hash_manifest(report: &ValidationReport) -> String {
    format!(
        concat!(
            "payload\tbytes\tsha256\n",
            "normalized_matrix_text\t650\t{}\n",
            "contiguous_symbols\t624\t{}\n",
            "logical_rgba\t2496\t{}\n",
            "runtime_rgba\t159744\t{}\n",
            "decoded_sheet_rgba\t11501568\t{}\n"
        ),
        report.normalized_matrix_sha256,
        report.contiguous_symbol_sha256,
        report.logical_rgba_sha256,
        report.runtime_rgba_sha256,
        report.sheet_rgba_sha256,
    )
}

fn block_report(frame: &RgbaImage) -> String {
    let mut text = String::from(
        "logical_x\tlogical_y\truntime_x0\truntime_y0\truntime_x1\truntime_y1\texpected_rgba\tactual_origin_rgba\tuniform\tmismatch_pixels\n",
    );
    for logical_y in 0..kitten::LOGICAL_HEIGHT {
        for logical_x in 0..kitten::LOGICAL_WIDTH {
            let x0 = logical_x as u32 * kitten::RUNTIME_PIXEL_SIZE;
            let y0 = logical_y as u32 * kitten::RUNTIME_PIXEL_SIZE;
            let x1 = x0 + kitten::RUNTIME_PIXEL_SIZE - 1;
            let y1 = y0 + kitten::RUNTIME_PIXEL_SIZE - 1;
            let actual = frame.get_pixel(x0, y0).0;
            let expected = kitten::palette_color(
                kitten::canonical_symbol(logical_x, logical_y).expect("complete canonical matrix"),
            )
            .expect("canonical palette symbol");
            let mismatches = (y0..=y1)
                .flat_map(|y| (x0..=x1).map(move |x| (x, y)))
                .filter(|(x, y)| frame.get_pixel(*x, *y).0 != actual)
                .count();
            text.push_str(&format!(
                "{logical_x}\t{logical_y}\t{x0}\t{y0}\t{x1}\t{y1}\t{}\t{}\t{}\t{mismatches}\n",
                rgba_hex(expected),
                rgba_hex(actual),
                mismatches == 0,
            ));
        }
    }
    text
}

fn mismatch_report(report: &ValidationReport) -> String {
    format!(
        concat!(
            "kind\tlogical_coordinate\truntime_coordinate_or_rectangle\texpected_rgba\tactual_rgba\n",
            "SUMMARY_source_mismatches\t{}\tNOT_APPLICABLE\tNOT_APPLICABLE\tNOT_APPLICABLE\n",
            "SUMMARY_runtime_mismatches\t{}\tNOT_APPLICABLE\tNOT_APPLICABLE\tNOT_APPLICABLE\n",
            "SUMMARY_reconstructed_source_mismatches\t{}\tNOT_APPLICABLE\tNOT_APPLICABLE\tNOT_APPLICABLE\n",
            "SUMMARY_nonuniform_logical_blocks\t{}\tNOT_APPLICABLE\tNOT_APPLICABLE\tNOT_APPLICABLE\n",
            "SUMMARY_illegal_source_symbols\t{}\tNOT_APPLICABLE\tNOT_APPLICABLE\tNOT_APPLICABLE\n",
            "SUMMARY_illegal_runtime_colors\t{}\tNOT_APPLICABLE\tNOT_APPLICABLE\tNOT_APPLICABLE\n",
            "SUMMARY_alpha_violations\t{}\tNOT_APPLICABLE\tNOT_APPLICABLE\tNOT_APPLICABLE\n",
            "SUMMARY_frame_to_frame_mismatches\t{}\tNOT_APPLICABLE\tNOT_APPLICABLE\tNOT_APPLICABLE\n"
        ),
        report.source_mismatches,
        report.runtime_mismatches,
        report.reconstructed_source_mismatches,
        report.nonuniform_logical_blocks,
        report.illegal_source_symbols,
        report.illegal_runtime_colors,
        report.alpha_violations,
        report.frame_to_frame_mismatches,
    )
}

fn core_artifacts(
    packed: &RgbaImage,
    report: &ValidationReport,
) -> Result<Vec<(PathBuf, Vec<u8>)>> {
    let logical = kitten::render_logical();
    let runtime = kitten::render_frame();
    let frame_zero = sheet::extract_frame(packed, 0)?;
    Ok(vec![
        (
            PathBuf::from(CANONICAL_MATRIX_FILE),
            kitten::normalized_matrix_text().into_bytes(),
        ),
        (
            PathBuf::from(COORDINATE_MATRIX_FILE),
            coordinate_matrix().into_bytes(),
        ),
        (
            PathBuf::from(CANONICAL_PNG_FILE),
            artwork::encode_png(&logical)?,
        ),
        (
            PathBuf::from(RUNTIME_PNG_FILE),
            artwork::encode_png(&runtime)?,
        ),
        (
            PathBuf::from(DARK_INSPECTION_FILE),
            artwork::encode_png(&solid_inspection(&runtime, DARK_BACKGROUND))?,
        ),
        (
            PathBuf::from(LIGHT_INSPECTION_FILE),
            artwork::encode_png(&solid_inspection(&runtime, LIGHT_BACKGROUND))?,
        ),
        (
            PathBuf::from(DECODED_FRAME_FILE),
            artwork::encode_png(&frame_zero)?,
        ),
        (
            PathBuf::from(DECODED_SHEET_FILE),
            artwork::encode_png(packed)?,
        ),
        (PathBuf::from(PALETTE_FILE), palette_manifest().into_bytes()),
        (PathBuf::from(HASH_FILE), hash_manifest(report).into_bytes()),
        (
            PathBuf::from(BLOCK_REPORT_FILE),
            block_report(&runtime).into_bytes(),
        ),
        (
            PathBuf::from(MISMATCH_REPORT_FILE),
            mismatch_report(report).into_bytes(),
        ),
        (
            PathBuf::from(VALIDATION_REPORT_FILE),
            report.summary_tsv().into_bytes(),
        ),
    ])
}

pub fn generation_relative_paths() -> Vec<PathBuf> {
    let mut paths = vec![PathBuf::from(MANIFEST_FILE), PathBuf::from(SHEET_FILE)];
    paths.extend(artwork::review_relative_paths());
    paths.extend(
        [
            CANONICAL_MATRIX_FILE,
            COORDINATE_MATRIX_FILE,
            CANONICAL_PNG_FILE,
            RUNTIME_PNG_FILE,
            DARK_INSPECTION_FILE,
            LIGHT_INSPECTION_FILE,
            DECODED_FRAME_FILE,
            DECODED_SHEET_FILE,
            PALETTE_FILE,
            HASH_FILE,
            BLOCK_REPORT_FILE,
            MISMATCH_REPORT_FILE,
            VALIDATION_REPORT_FILE,
        ]
        .into_iter()
        .map(|name| PathBuf::from(EVIDENCE_DIRECTORY).join(name)),
    );
    paths
}

pub fn write_generation_evidence(
    project: &Path,
    packed: &RgbaImage,
    report: &ValidationReport,
) -> Result<Vec<PathBuf>> {
    let directory = project.join(EVIDENCE_DIRECTORY);
    fs::create_dir_all(&directory)?;
    let mut paths = Vec::new();
    for (relative, bytes) in core_artifacts(packed, report)? {
        let path = directory.join(relative);
        sheet::write_atomic(&path, &bytes)?;
        paths.push(path);
    }
    Ok(paths)
}

pub fn validate_generation_evidence(
    project: &Path,
    packed: &RgbaImage,
    report: &ValidationReport,
) -> Result<()> {
    let directory = project.join(EVIDENCE_DIRECTORY);
    let mut details = Vec::new();
    for (relative, expected) in core_artifacts(packed, report)? {
        let path = directory.join(relative);
        let actual = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) => {
                details.push(format!("{} missing or unreadable: {error}", path.display()));
                continue;
            }
        };
        let mismatches = byte_mismatch_count(&expected, &actual);
        if mismatches != 0 {
            details.push(format!(
                "{} byte mismatch count {mismatches}; expected SHA-256 {} actual SHA-256 {}",
                path.display(),
                sha256_hex(&expected),
                sha256_hex(&actual)
            ));
        }
    }
    if details.is_empty() {
        Ok(())
    } else {
        fail(format!(
            "generation evidence mismatch count {}\n{}",
            details.len(),
            details.join("\n")
        ))
    }
}

pub fn byte_mismatch_count(left: &[u8], right: &[u8]) -> usize {
    left.iter().zip(right).filter(|(a, b)| a != b).count() + left.len().abs_diff(right.len())
}

pub fn compare_generations(project: &Path, left: &Path, right: &Path) -> Result<()> {
    let mut report = String::from(
        "relative_path\tleft_sha256\tright_sha256\tleft_bytes\tright_bytes\tbyte_mismatches\n",
    );
    let mut total = 0usize;
    for relative in generation_relative_paths() {
        let left_bytes = fs::read(left.join(&relative))?;
        let right_bytes = fs::read(right.join(&relative))?;
        let mismatches = byte_mismatch_count(&left_bytes, &right_bytes);
        total += mismatches;
        report.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{mismatches}\n",
            relative.display(),
            sha256_hex(&left_bytes),
            sha256_hex(&right_bytes),
            left_bytes.len(),
            right_bytes.len(),
        ));
    }
    report.push_str(&format!(
        "TOTAL\tNOT_APPLICABLE\tNOT_APPLICABLE\tNOT_APPLICABLE\tNOT_APPLICABLE\t{total}\n"
    ));
    let path = project
        .join(EVIDENCE_DIRECTORY)
        .join(DETERMINISM_REPORT_FILE);
    sheet::write_atomic(&path, report.as_bytes())?;
    if total == 0 {
        Ok(())
    } else {
        fail(format!(
            "deterministic-generation byte mismatch count {total}; see {}",
            path.display()
        ))
    }
}

pub fn compare_installed(project: &Path, installed: &Path) -> Result<()> {
    let mut report = String::from(
        "asset\tgenerated_sha256\tinstalled_sha256\tgenerated_bytes\tinstalled_bytes\tbyte_mismatches\n",
    );
    let mut total = 0usize;
    for name in [MANIFEST_FILE, SHEET_FILE] {
        let generated = fs::read(project.join(name))?;
        let active = fs::read(installed.join(name))?;
        let mismatches = byte_mismatch_count(&generated, &active);
        total += mismatches;
        report.push_str(&format!(
            "{name}\t{}\t{}\t{}\t{}\t{mismatches}\n",
            sha256_hex(&generated),
            sha256_hex(&active),
            generated.len(),
            active.len(),
        ));
    }
    let generated_sheet = sheet::load_rgba(&project.join(SHEET_FILE))?;
    let installed_sheet = sheet::load_rgba(&installed.join(SHEET_FILE))?;
    let decoded_mismatches =
        byte_mismatch_count(generated_sheet.as_raw(), installed_sheet.as_raw());
    total += decoded_mismatches;
    report.push_str(&format!(
        "decoded_sheet_rgba\t{}\t{}\t{}\t{}\t{decoded_mismatches}\n",
        sha256_hex(generated_sheet.as_raw()),
        sha256_hex(installed_sheet.as_raw()),
        generated_sheet.as_raw().len(),
        installed_sheet.as_raw().len(),
    ));
    report.push_str(&format!(
        "TOTAL\tNOT_APPLICABLE\tNOT_APPLICABLE\tNOT_APPLICABLE\tNOT_APPLICABLE\t{total}\n"
    ));
    let path = project.join(EVIDENCE_DIRECTORY).join(INSTALLED_REPORT_FILE);
    sheet::write_atomic(&path, report.as_bytes())?;
    if total == 0 {
        Ok(())
    } else {
        fail(format!(
            "generated-versus-installed mismatch count {total}; see {}",
            path.display()
        ))
    }
}

fn expand_logical_independently() -> RgbaImage {
    let logical = kitten::render_logical();
    RgbaImage::from_fn(FRAME_WIDTH, FRAME_HEIGHT, |x, y| {
        *logical.get_pixel(
            x / kitten::RUNTIME_PIXEL_SIZE,
            y / kitten::RUNTIME_PIXEL_SIZE,
        )
    })
}

pub fn compare_observed_frame(project: &Path, observed_path: &Path) -> Result<()> {
    let observed = image::open(observed_path)?.to_rgba8();
    let canonical_expansion = expand_logical_independently();
    let runtime = kitten::render_frame();
    let dimensions_match = observed.dimensions() == (FRAME_WIDTH, FRAME_HEIGHT);
    let mut mismatch_rows =
        String::from("runtime_x\truntime_y\tlogical_x\tlogical_y\texpected_rgba\tactual_rgba\n");
    let mut mismatch_count = 0usize;
    let mut xor = RgbaImage::new(FRAME_WIDTH, FRAME_HEIGHT);
    if dimensions_match {
        for (x, y, expected) in runtime.enumerate_pixels() {
            let actual = observed.get_pixel(x, y);
            let difference = [
                expected[0] ^ actual[0],
                expected[1] ^ actual[1],
                expected[2] ^ actual[2],
                expected[3] ^ actual[3],
            ];
            xor.put_pixel(x, y, Rgba(difference));
            if expected != actual {
                mismatch_count += 1;
                mismatch_rows.push_str(&format!(
                    "{x}\t{y}\t{}\t{}\t{}\t{}\n",
                    x / kitten::RUNTIME_PIXEL_SIZE,
                    y / kitten::RUNTIME_PIXEL_SIZE,
                    rgba_hex(expected.0),
                    rgba_hex(actual.0),
                ));
            }
        }
    } else {
        mismatch_count = (FRAME_WIDTH * FRAME_HEIGHT) as usize;
    }

    let mut board = RgbaImage::new(FRAME_WIDTH * 4, FRAME_HEIGHT);
    for (panel, source) in [&canonical_expansion, &runtime, &observed, &xor]
        .into_iter()
        .enumerate()
    {
        if source.dimensions() != (FRAME_WIDTH, FRAME_HEIGHT) {
            continue;
        }
        for (x, y, pixel) in source.enumerate_pixels() {
            board.put_pixel(panel as u32 * FRAME_WIDTH + x, y, *pixel);
        }
    }

    let directory = project.join(EVIDENCE_DIRECTORY);
    fs::create_dir_all(&directory)?;
    write_png(&directory.join(OBSERVED_FRAME_FILE), &observed)?;
    write_png(&directory.join(XOR_FILE), &xor)?;
    write_png(&directory.join(COMPARISON_BOARD_FILE), &board)?;
    let source_bytes = fs::read(observed_path)?;
    let report = format!(
        concat!(
            "check\tvalue\n",
            "source_path\t{}\n",
            "source_file_sha256\t{}\n",
            "observed_width\t{}\n",
            "observed_height\t{}\n",
            "observed_rgba_sha256\t{}\n",
            "expected_rgba_sha256\t{}\n",
            "runtime_pixel_mismatches\t{}\n"
        ),
        observed_path.display(),
        sha256_hex(&source_bytes),
        observed.width(),
        observed.height(),
        sha256_hex(observed.as_raw()),
        kitten::RUNTIME_RGBA_SHA256,
        mismatch_count,
    );
    sheet::write_atomic(&directory.join(OBSERVED_REPORT_FILE), report.as_bytes())?;
    mismatch_rows.push_str(&format!(
        "SUMMARY\tNOT_APPLICABLE\tNOT_APPLICABLE\tNOT_APPLICABLE\tNOT_APPLICABLE\t{mismatch_count}\n"
    ));
    sheet::write_atomic(
        &directory.join("fresh-codex-cache-mismatches.tsv"),
        mismatch_rows.as_bytes(),
    )?;
    let manifest = concat!(
        "panel\tx0\ty0\twidth\theight\tcontent\n",
        "0\t0\t0\t192\t208\tcanonical 24x26 enlarged by independent direct 8x nearest neighbor\n",
        "1\t192\t0\t192\t208\texact production runtime frame\n",
        "2\t384\t0\t192\t208\tfresh Codex cache frame\n",
        "3\t576\t0\t192\t208\tRGBA XOR difference between runtime and fresh cache frame\n",
    );
    sheet::write_atomic(
        &directory.join(COMPARISON_BOARD_MANIFEST_FILE),
        manifest.as_bytes(),
    )?;

    if !dimensions_match {
        return fail(format!(
            "observed frame is {}x{}, expected {FRAME_WIDTH}x{FRAME_HEIGHT}",
            observed.width(),
            observed.height()
        ));
    }
    if mismatch_count == 0 {
        Ok(())
    } else {
        fail(format!(
            "observed runtime pixel mismatch count {mismatch_count}; see {}",
            directory.join("fresh-codex-cache-mismatches.tsv").display()
        ))
    }
}

pub fn compare_cache_directory(project: &Path, frames_directory: &Path) -> Result<()> {
    let canonical = kitten::render_frame();
    let mut report =
        String::from("frame\tfile\tfile_sha256\trgba_sha256\twidth\theight\tpixel_mismatches\n");
    let mut total = 0usize;
    for index in 0..FRAME_COUNT {
        let path = frames_directory.join(format!("frame_{index:03}.png"));
        let bytes = fs::read(&path)?;
        let frame = image::load_from_memory(&bytes)?.to_rgba8();
        let mismatches = if frame.dimensions() == (FRAME_WIDTH, FRAME_HEIGHT) {
            canonical
                .pixels()
                .zip(frame.pixels())
                .filter(|(expected, actual)| expected != actual)
                .count()
        } else {
            (FRAME_WIDTH * FRAME_HEIGHT) as usize
        };
        total += mismatches;
        report.push_str(&format!(
            "{index}\t{}\t{}\t{}\t{}\t{}\t{mismatches}\n",
            path.display(),
            sha256_hex(&bytes),
            sha256_hex(frame.as_raw()),
            frame.width(),
            frame.height(),
        ));
    }
    let png_count = fs::read_dir(frames_directory)?
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some("png"))
        .count();
    if png_count != FRAME_COUNT {
        total += png_count.abs_diff(FRAME_COUNT);
    }
    report.push_str(&format!(
        "TOTAL\tNOT_APPLICABLE\tNOT_APPLICABLE\tNOT_APPLICABLE\tNOT_APPLICABLE\tNOT_APPLICABLE\t{total}\n"
    ));
    let directory = project.join(EVIDENCE_DIRECTORY);
    sheet::write_atomic(&directory.join(CACHE_FRAMES_REPORT_FILE), report.as_bytes())?;
    compare_observed_frame(project, &frames_directory.join("frame_000.png"))?;
    if total == 0 {
        Ok(())
    } else {
        fail(format!(
            "fresh Codex cache mismatch count {total}; see {}",
            directory.join(CACHE_FRAMES_REPORT_FILE).display()
        ))
    }
}

pub fn record_runtime_capture(project: &Path, capture_path: &Path) -> Result<()> {
    let bytes = fs::read(capture_path)?;
    if bytes.is_empty() {
        return fail(format!(
            "runtime capture is empty: {}",
            capture_path.display()
        ));
    }
    let directory = project.join(EVIDENCE_DIRECTORY);
    fs::create_dir_all(&directory)?;
    sheet::write_atomic(&directory.join(PROCESS_CAPTURE_FILE), &bytes)?;
    let report = format!(
        concat!(
            "field\tvalue\n",
            "source_path\t{}\n",
            "captured_bytes\t{}\n",
            "capture_sha256\t{}\n"
        ),
        capture_path.display(),
        bytes.len(),
        sha256_hex(&bytes),
    );
    sheet::write_atomic(
        &directory.join(PROCESS_CAPTURE_REPORT_FILE),
        report.as_bytes(),
    )
}

pub fn validate_external_evidence(project: &Path) -> Result<()> {
    for name in [
        DETERMINISM_REPORT_FILE,
        INSTALLED_REPORT_FILE,
        OBSERVED_REPORT_FILE,
        CACHE_FRAMES_REPORT_FILE,
    ] {
        let path = project.join(EVIDENCE_DIRECTORY).join(name);
        if !path.is_file() {
            return fail(format!(
                "required external evidence is missing: {}",
                path.display()
            ));
        }
        let text = fs::read_to_string(&path)?;
        let final_line = text
            .lines()
            .last()
            .ok_or_else(|| WhiteCatError::new(format!("{} is empty", path.display())))?;
        if !final_line.ends_with("\t0") {
            return fail(format!(
                "{} does not end in a zero mismatch total",
                path.display()
            ));
        }
    }
    let directory = project.join(EVIDENCE_DIRECTORY);
    let observed = image::open(directory.join(OBSERVED_FRAME_FILE))?.to_rgba8();
    validate::validate_frame(&observed)?;
    let xor = image::open(directory.join(XOR_FILE))?.to_rgba8();
    if xor.dimensions() != (FRAME_WIDTH, FRAME_HEIGHT)
        || xor.as_raw().iter().any(|channel| *channel != 0)
    {
        return fail("runtime capture XOR surface is not an all-zero 192x208 RGBA image");
    }
    let board = image::open(directory.join(COMPARISON_BOARD_FILE))?.to_rgba8();
    if board.dimensions() != (FRAME_WIDTH * 4, FRAME_HEIGHT) {
        return fail("comparison board is not 768x208");
    }
    let runtime = kitten::render_frame();
    for panel in 0..3u32 {
        for (x, y, expected) in runtime.enumerate_pixels() {
            if board.get_pixel(panel * FRAME_WIDTH + x, y) != expected {
                return fail(format!(
                    "comparison board panel {panel} mismatch at runtime ({x},{y})"
                ));
            }
        }
    }
    let process_capture = directory.join(PROCESS_CAPTURE_FILE);
    if !process_capture.is_file() || fs::metadata(&process_capture)?.len() == 0 {
        return fail(format!(
            "fresh-process capture is missing or empty: {}",
            process_capture.display()
        ));
    }
    if !directory.join(PROCESS_CAPTURE_REPORT_FILE).is_file() {
        return fail("fresh-process capture hash report is missing");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{GRID_COLUMNS, GRID_ROWS};

    #[test]
    fn evidence_dimensions_and_coordinate_matrix_are_exact() {
        let frame = kitten::render_frame();
        assert_eq!(
            solid_inspection(&frame, DARK_BACKGROUND).dimensions(),
            (192, 208)
        );
        assert_eq!(coordinate_matrix().lines().count(), 29);
        assert!(coordinate_matrix().contains("25  ........................"));
        assert_eq!(
            GRID_COLUMNS * GRID_ROWS,
            crate::contract::FRAME_COUNT as u32
        );
    }

    #[test]
    fn independent_expansion_matches_production() {
        assert_eq!(
            expand_logical_independently().as_raw(),
            kitten::render_frame().as_raw()
        );
    }
}

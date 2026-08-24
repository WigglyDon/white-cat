use std::fs;
use std::path::{Path, PathBuf};

use image::codecs::webp::WebPEncoder;
use image::{ImageEncoder, RgbaImage};

use crate::contract::{
    FRAME_COUNT, FRAME_HEIGHT, FRAME_WIDTH, GRID_COLUMNS, SHEET_HEIGHT, SHEET_WIDTH,
};
use crate::error::{Result, WhiteCatError, fail};

pub fn pack_fixed_frames(frames: &[RgbaImage]) -> Result<RgbaImage> {
    if frames.len() != FRAME_COUNT {
        return fail(format!(
            "frame count is {}, expected {FRAME_COUNT}",
            frames.len()
        ));
    }

    let mut sheet = RgbaImage::new(SHEET_WIDTH, SHEET_HEIGHT);
    for (index, frame) in frames.iter().enumerate() {
        if frame.dimensions() != (FRAME_WIDTH, FRAME_HEIGHT) {
            return fail(format!(
                "frame {index} is {}x{}, expected {FRAME_WIDTH}x{FRAME_HEIGHT}",
                frame.width(),
                frame.height()
            ));
        }
        let cell_x = index as u32 % GRID_COLUMNS * FRAME_WIDTH;
        let cell_y = index as u32 / GRID_COLUMNS * FRAME_HEIGHT;
        for y in 0..FRAME_HEIGHT {
            let source_start = (y * FRAME_WIDTH * 4) as usize;
            let source_end = source_start + (FRAME_WIDTH * 4) as usize;
            let destination_start = (((cell_y + y) * SHEET_WIDTH + cell_x) * 4) as usize;
            let destination_end = destination_start + (FRAME_WIDTH * 4) as usize;
            sheet.as_mut()[destination_start..destination_end]
                .copy_from_slice(&frame.as_raw()[source_start..source_end]);
        }
    }
    Ok(sheet)
}

pub fn extract_frame(sheet: &RgbaImage, index: usize) -> Result<RgbaImage> {
    if sheet.dimensions() != (SHEET_WIDTH, SHEET_HEIGHT) {
        return fail(format!(
            "sheet is {}x{}, expected {SHEET_WIDTH}x{SHEET_HEIGHT}",
            sheet.width(),
            sheet.height()
        ));
    }
    if index >= FRAME_COUNT {
        return fail(format!("frame index {index} is outside 0..{FRAME_COUNT}"));
    }

    let cell_x = index as u32 % GRID_COLUMNS * FRAME_WIDTH;
    let cell_y = index as u32 / GRID_COLUMNS * FRAME_HEIGHT;
    let mut frame = RgbaImage::new(FRAME_WIDTH, FRAME_HEIGHT);
    for y in 0..FRAME_HEIGHT {
        let source_start = (((cell_y + y) * SHEET_WIDTH + cell_x) * 4) as usize;
        let source_end = source_start + (FRAME_WIDTH * 4) as usize;
        let destination_start = (y * FRAME_WIDTH * 4) as usize;
        let destination_end = destination_start + (FRAME_WIDTH * 4) as usize;
        frame.as_mut()[destination_start..destination_end]
            .copy_from_slice(&sheet.as_raw()[source_start..source_end]);
    }
    Ok(frame)
}

pub fn encode_lossless_webp(image: &RgbaImage) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    WebPEncoder::new_lossless(&mut bytes).write_image(
        image.as_raw(),
        image.width(),
        image.height(),
        image::ExtendedColorType::Rgba8,
    )?;
    Ok(bytes)
}

pub fn write_lossless_webp(path: &Path, image: &RgbaImage) -> Result<()> {
    write_atomic(path, &encode_lossless_webp(image)?)
}

pub fn load_rgba(path: &Path) -> Result<RgbaImage> {
    Ok(image::open(path)?.to_rgba8())
}

pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| WhiteCatError::new(format!("{} has no parent", path.display())))?;
    fs::create_dir_all(parent)?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| WhiteCatError::new(format!("{} has no file name", path.display())))?;
    let temporary: PathBuf = parent.join(format!(".{file_name}.tmp-{}", std::process::id()));
    fs::write(&temporary, bytes)?;
    fs::rename(&temporary, path)?;
    Ok(())
}

pub fn validate_lossless_static_webp(path: &Path) -> Result<()> {
    let bytes = fs::read(path)?;
    if bytes.len() < 20 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WEBP" {
        return fail(format!("{} is not a WebP RIFF container", path.display()));
    }
    let declared = u32::from_le_bytes(bytes[4..8].try_into().expect("four-byte RIFF size"));
    if declared as usize + 8 != bytes.len() {
        return fail(format!("{} has an invalid RIFF length", path.display()));
    }

    let mut offset = 12usize;
    let mut found_lossless = false;
    while offset + 8 <= bytes.len() {
        let kind = &bytes[offset..offset + 4];
        let length = u32::from_le_bytes(
            bytes[offset + 4..offset + 8]
                .try_into()
                .expect("four-byte chunk size"),
        ) as usize;
        if kind == b"VP8L" {
            found_lossless = true;
        }
        if kind == b"ANIM" || kind == b"ANMF" || kind == b"VP8 " {
            return fail(format!("{} is not a static lossless WebP", path.display()));
        }
        offset = offset
            .checked_add(8 + length + (length % 2))
            .ok_or_else(|| WhiteCatError::new("WebP chunk offset overflow"))?;
    }
    if offset != bytes.len() || !found_lossless {
        return fail(format!(
            "{} does not contain one complete VP8L payload",
            path.display()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn fixed_cells_are_not_trimmed_or_recentered() {
        let mut frames = vec![RgbaImage::new(FRAME_WIDTH, FRAME_HEIGHT); FRAME_COUNT];
        frames[17].put_pixel(3, 191, Rgba([1, 2, 3, 4]));
        let packed = pack_fixed_frames(&frames).expect("pack fixed cells");
        let extracted = extract_frame(&packed, 17).expect("extract fixed cell");
        assert_eq!(extracted.get_pixel(3, 191), &Rgba([1, 2, 3, 4]));
    }
}

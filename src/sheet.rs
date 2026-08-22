use std::fs::{self, File};
use std::io::BufWriter;
use std::path::Path;

use image::codecs::webp::WebPEncoder;
use image::{ExtendedColorType, ImageEncoder, ImageFormat, ImageReader, RgbaImage};

use crate::contract::{
    FRAME_COUNT, FRAME_HEIGHT, FRAME_WIDTH, SHEET_HEIGHT, SHEET_WIDTH, frame_position,
};
use crate::{Result, WhiteCatError};

fn require_frame(frame: &RgbaImage, index: usize) -> Result<()> {
    if frame.dimensions() != (FRAME_WIDTH as u32, FRAME_HEIGHT as u32) {
        return Err(WhiteCatError::new(format!(
            "frame {index} size is {:?}, expected ({FRAME_WIDTH}, {FRAME_HEIGHT})",
            frame.dimensions()
        )));
    }
    Ok(())
}

pub fn pack_fixed_frames(frames: &[RgbaImage]) -> Result<RgbaImage> {
    if frames.len() != FRAME_COUNT {
        return Err(WhiteCatError::new(format!(
            "received {} frames, expected {FRAME_COUNT}",
            frames.len()
        )));
    }

    let mut sheet = RgbaImage::new(SHEET_WIDTH as u32, SHEET_HEIGHT as u32);
    let destination = sheet.as_mut();
    for (index, frame) in frames.iter().enumerate() {
        require_frame(frame, index)?;
        let (row, column) = frame_position(index).expect("validated frame index");
        for y in 0..FRAME_HEIGHT {
            let source_start = y * FRAME_WIDTH * 4;
            let source_end = source_start + FRAME_WIDTH * 4;
            let destination_start =
                ((row * FRAME_HEIGHT + y) * SHEET_WIDTH + column * FRAME_WIDTH) * 4;
            let destination_end = destination_start + FRAME_WIDTH * 4;
            destination[destination_start..destination_end]
                .copy_from_slice(&frame.as_raw()[source_start..source_end]);
        }
    }
    Ok(sheet)
}

pub fn extract_fixed_frames(sheet: &RgbaImage) -> Result<Vec<RgbaImage>> {
    if sheet.dimensions() != (SHEET_WIDTH as u32, SHEET_HEIGHT as u32) {
        return Err(WhiteCatError::new(format!(
            "sheet size is {:?}, expected ({SHEET_WIDTH}, {SHEET_HEIGHT})",
            sheet.dimensions()
        )));
    }

    let mut frames = Vec::with_capacity(FRAME_COUNT);
    for index in 0..FRAME_COUNT {
        let (row, column) = frame_position(index).expect("validated frame index");
        let mut frame = RgbaImage::new(FRAME_WIDTH as u32, FRAME_HEIGHT as u32);
        for y in 0..FRAME_HEIGHT {
            let source_start = ((row * FRAME_HEIGHT + y) * SHEET_WIDTH + column * FRAME_WIDTH) * 4;
            let source_end = source_start + FRAME_WIDTH * 4;
            let destination_start = y * FRAME_WIDTH * 4;
            let destination_end = destination_start + FRAME_WIDTH * 4;
            frame.as_mut()[destination_start..destination_end]
                .copy_from_slice(&sheet.as_raw()[source_start..source_end]);
        }
        require_frame(&frame, index)?;
        frames.push(frame);
    }
    Ok(frames)
}

pub fn save_lossless_sheet(sheet: &RgbaImage, path: &Path) -> Result<()> {
    if sheet.dimensions() != (SHEET_WIDTH as u32, SHEET_HEIGHT as u32) {
        return Err(WhiteCatError::new(
            "refusing to encode a non-contract sheet",
        ));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let writer = BufWriter::new(File::create(path)?);
    WebPEncoder::new_lossless(writer).write_image(
        sheet.as_raw(),
        SHEET_WIDTH as u32,
        SHEET_HEIGHT as u32,
        ExtendedColorType::Rgba8,
    )?;
    Ok(())
}

pub fn load_sheet(path: &Path) -> Result<RgbaImage> {
    let reader = ImageReader::open(path)?.with_guessed_format()?;
    if reader.format() != Some(ImageFormat::WebP) {
        return Err(WhiteCatError::new("spritesheet must be WebP"));
    }
    let image = reader.decode()?.into_rgba8();
    if image.dimensions() != (SHEET_WIDTH as u32, SHEET_HEIGHT as u32) {
        return Err(WhiteCatError::new("spritesheet dimensions are invalid"));
    }
    Ok(image)
}

pub fn webp_chunk_ids(path: &Path) -> Result<Vec<[u8; 4]>> {
    let data = fs::read(path)?;
    if data.len() < 12 || &data[0..4] != b"RIFF" || &data[8..12] != b"WEBP" {
        return Err(WhiteCatError::new("invalid WebP header"));
    }
    let declared = u32::from_le_bytes(data[4..8].try_into().expect("four bytes")) as usize + 8;
    if declared != data.len() {
        return Err(WhiteCatError::new("WebP RIFF size mismatch"));
    }

    let mut chunks = Vec::new();
    let mut offset = 12;
    while offset < data.len() {
        if offset + 8 > data.len() {
            return Err(WhiteCatError::new("truncated WebP chunk header"));
        }
        let id: [u8; 4] = data[offset..offset + 4].try_into().expect("four bytes");
        let size = u32::from_le_bytes(data[offset + 4..offset + 8].try_into().expect("four bytes"))
            as usize;
        let end = offset + 8 + size;
        if end > data.len() {
            return Err(WhiteCatError::new(format!(
                "truncated WebP chunk: {:?}",
                String::from_utf8_lossy(&id)
            )));
        }
        chunks.push(id);
        offset = end + (size & 1);
    }
    if offset != data.len() {
        return Err(WhiteCatError::new("invalid WebP chunk padding"));
    }
    Ok(chunks)
}

#[cfg(test)]
mod tests {
    use image::Rgba;

    use super::*;

    fn blank_frames() -> Vec<RgbaImage> {
        (0..FRAME_COUNT)
            .map(|_| RgbaImage::new(FRAME_WIDTH as u32, FRAME_HEIGHT as u32))
            .collect()
    }

    #[test]
    fn packer_rejects_wrong_frame_size() {
        let mut frames = blank_frames();
        frames[9] = RgbaImage::new((FRAME_WIDTH - 1) as u32, FRAME_HEIGHT as u32);
        assert!(
            pack_fixed_frames(&frames)
                .unwrap_err()
                .to_string()
                .contains("frame 9 size")
        );
    }

    #[test]
    fn packer_copies_complete_frames_to_fixed_coordinates() {
        let mut frames = blank_frames();
        frames[10].put_pixel(17, 23, Rgba([11, 22, 33, 255]));
        let sheet = pack_fixed_frames(&frames).unwrap();
        let (row, column) = frame_position(10).unwrap();
        assert_eq!(
            sheet.get_pixel(
                (column * FRAME_WIDTH + 17) as u32,
                (row * FRAME_HEIGHT + 23) as u32
            ),
            &Rgba([11, 22, 33, 255])
        );
    }

    #[test]
    fn extraction_round_trips_every_complete_cell() {
        let mut frames = blank_frames();
        for (index, frame) in frames.iter_mut().enumerate() {
            frame.put_pixel(1, 1, Rgba([index as u8, 2, 3, 255]));
        }
        let extracted = extract_fixed_frames(&pack_fixed_frames(&frames).unwrap()).unwrap();
        assert_eq!(extracted, frames);
    }
}

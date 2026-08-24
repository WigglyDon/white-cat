//! Canonical authored artwork for the production White Cat pet.
//!
//! `CANONICAL_MAP` is a literal transcription of the approved
//! `concept_design_of_pixel_art_cat.png` geometry and five-color palette. Each
//! logical pixel occupies an 8 x 8 runtime block. The map is authored into the
//! required 4x premultiplied source canvas and downsampled exactly once with
//! Lanczos for the production frame.

use image::{Rgba, RgbaImage, imageops::FilterType};

use crate::contract::{FRAME_HEIGHT, FRAME_WIDTH, GROUND_Y};

pub const CONCEPT_REFERENCE_FILE: &str = "concept_design_of_pixel_art_cat.png";
pub const CONCEPT_REFERENCE_SHA256: &str =
    "974bae7813b6b80a0626ca5b3d292244f5abf937f97a3f6c3102fb70180ea322";

pub const SOURCE_SCALE: u32 = 4;
pub const SOURCE_WIDTH: u32 = FRAME_WIDTH * SOURCE_SCALE;
pub const SOURCE_HEIGHT: u32 = FRAME_HEIGHT * SOURCE_SCALE;

pub const LOGICAL_WIDTH: usize = 24;
pub const LOGICAL_HEIGHT: usize = 26;
pub const RUNTIME_PIXEL_SIZE: u32 = 8;
pub const SOURCE_PIXEL_SIZE: u32 = RUNTIME_PIXEL_SIZE * SOURCE_SCALE;

pub const TRANSPARENT: [u8; 4] = [0, 0, 0, 0];
pub const OUTLINE: [u8; 4] = [42, 51, 64, 255];
pub const FUR: [u8; 4] = [244, 242, 232, 255];
pub const SHADOW: [u8; 4] = [205, 210, 216, 255];
pub const EYE: [u8; 4] = [134, 215, 168, 255];
pub const SILHOUETTE: [u8; 4] = OUTLINE;

pub const PALETTE: [(char, [u8; 4]); 5] = [
    ('.', TRANSPARENT),
    ('O', OUTLINE),
    ('W', FUR),
    ('S', SHADOW),
    ('E', EYE),
];

pub type PixelMap = [&'static str; LOGICAL_HEIGHT];

/// The approved profile, including its left-wrapped tail, single green eye,
/// stepped ears, long chest, separated paws, and row-23 planted baseline.
pub const CANONICAL_MAP: PixelMap = [
    "........................",
    "........................",
    ".............O....O.....",
    "............OWO..OWO....",
    "............OWWOOOWWO...",
    "...........OWWWWWWWWO...",
    "..........OWWWWWWWWWWO..",
    ".........OWWWWWWWEWWWWO.",
    "........OWWWWWWWWWWWWO..",
    "........OWWWWWWWWWWWWO..",
    "........OWWWWWWWWWWWO...",
    ".......OWWWWWWWWWWWO....",
    "......OWWWWWWWWWWWO.....",
    ".....OWWWWWWWWWWWWO.....",
    "....OWWWWWWWWWWWWWO.....",
    "....OWWWWWWWWWWWWWO.....",
    "...OWWWWWWWWWWWWWWO.....",
    "...OWWWWWWWWWWWWWWO.....",
    ".....OWWWWWWWWWWWWO.....",
    ".OO..OWWWWWWWWWWWO......",
    ".OWO.OWWWWWWWWWOWWO.....",
    ".OWWWWWWWWWWSOOWWO......",
    ".OWWSSWWWWWWWO.OWWO.....",
    ".OOOOOOOOOOOO..OOOO.....",
    "........................",
    "........................",
];

pub fn palette_color(symbol: char) -> Option<[u8; 4]> {
    PALETTE
        .iter()
        .find_map(|(candidate, color)| (*candidate == symbol).then_some(*color))
}

pub fn canonical_symbol(x: usize, y: usize) -> Option<char> {
    CANONICAL_MAP
        .get(y)?
        .as_bytes()
        .get(x)
        .map(|byte| *byte as char)
}

pub fn canonical_map_is_well_formed() -> bool {
    FRAME_WIDTH == LOGICAL_WIDTH as u32 * RUNTIME_PIXEL_SIZE
        && FRAME_HEIGHT == LOGICAL_HEIGHT as u32 * RUNTIME_PIXEL_SIZE
        && SOURCE_WIDTH == LOGICAL_WIDTH as u32 * SOURCE_PIXEL_SIZE
        && SOURCE_HEIGHT == LOGICAL_HEIGHT as u32 * SOURCE_PIXEL_SIZE
        && CANONICAL_MAP.iter().all(|row| {
            row.len() == LOGICAL_WIDTH && row.chars().all(|symbol| palette_color(symbol).is_some())
        })
}

fn canonical_premultiplied_source() -> RgbaImage {
    assert!(
        canonical_map_is_well_formed(),
        "canonical pixel map violates its fixed geometry or palette"
    );

    let mut source = RgbaImage::from_pixel(SOURCE_WIDTH, SOURCE_HEIGHT, Rgba(TRANSPARENT));
    for (logical_y, row) in CANONICAL_MAP.iter().enumerate() {
        for (logical_x, symbol) in row.chars().enumerate() {
            let color = palette_color(symbol).expect("validated canonical-map symbol");
            if color[3] == 0 {
                continue;
            }
            let origin_x = logical_x as u32 * SOURCE_PIXEL_SIZE;
            let origin_y = logical_y as u32 * SOURCE_PIXEL_SIZE;
            for y in origin_y..origin_y + SOURCE_PIXEL_SIZE {
                for x in origin_x..origin_x + SOURCE_PIXEL_SIZE {
                    source.put_pixel(x, y, Rgba(color));
                }
            }
        }
    }
    source
}

fn unpremultiply(mut image: RgbaImage) -> RgbaImage {
    for pixel in image.pixels_mut() {
        let alpha = u32::from(pixel[3]);
        if alpha == 0 {
            *pixel = Rgba(TRANSPARENT);
            continue;
        }
        for channel in 0..3 {
            pixel[channel] = ((u32::from(pixel[channel]) * 255 + alpha / 2) / alpha).min(255) as u8;
        }
    }
    image
}

fn premultiply(image: &RgbaImage) -> RgbaImage {
    let mut result = image.clone();
    for pixel in result.pixels_mut() {
        let alpha = u32::from(pixel[3]);
        for channel in 0..3 {
            pixel[channel] = ((u32::from(pixel[channel]) * alpha + 127) / 255) as u8;
        }
    }
    result
}

pub fn resize_rgba(image: &RgbaImage, width: u32, height: u32) -> RgbaImage {
    let premultiplied = premultiply(image);
    let resized = image::imageops::resize(&premultiplied, width, height, FilterType::Lanczos3);
    unpremultiply(resized)
}

pub fn render_source() -> RgbaImage {
    unpremultiply(canonical_premultiplied_source())
}

pub fn render_frame() -> RgbaImage {
    let mut frame = image::imageops::resize(
        &canonical_premultiplied_source(),
        FRAME_WIDTH,
        FRAME_HEIGHT,
        FilterType::Lanczos3,
    );

    for (x, y, pixel) in frame.enumerate_pixels_mut() {
        if y >= GROUND_Y || pixel[3] < 2 || x < 2 || x + 2 >= FRAME_WIDTH {
            *pixel = Rgba(TRANSPARENT);
            continue;
        }
        let alpha = u32::from(pixel[3]);
        for channel in 0..3 {
            pixel[channel] = ((u32::from(pixel[channel]) * 255 + alpha / 2) / alpha).min(255) as u8;
        }
    }
    frame
}

pub fn render_silhouette_source() -> RgbaImage {
    let source = render_source();
    let mut silhouette = RgbaImage::new(SOURCE_WIDTH, SOURCE_HEIGHT);
    for (target, original) in silhouette.pixels_mut().zip(source.pixels()) {
        if original[3] == 0 {
            *target = Rgba(TRANSPARENT);
        } else {
            *target = Rgba([SILHOUETTE[0], SILHOUETTE[1], SILHOUETTE[2], original[3]]);
        }
    }
    silhouette
}

pub fn render_silhouette_frame() -> RgbaImage {
    let frame = render_frame();
    let mut silhouette = RgbaImage::new(FRAME_WIDTH, FRAME_HEIGHT);
    for (target, original) in silhouette.pixels_mut().zip(frame.pixels()) {
        if original[3] == 0 {
            *target = Rgba(TRANSPARENT);
        } else {
            *target = Rgba([SILHOUETTE[0], SILHOUETTE[1], SILHOUETTE[2], original[3]]);
        }
    }
    silhouette
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_map_is_exactly_24_by_26_with_only_concept_colors() {
        assert!(canonical_map_is_well_formed());
        assert_eq!(CANONICAL_MAP.len(), 26);
        assert!(CANONICAL_MAP.iter().all(|row| row.len() == 24));
        assert_eq!(CANONICAL_MAP[2], ".............O....O.....");
        assert_eq!(CANONICAL_MAP[7], ".........OWWWWWWWEWWWWO.");
        assert_eq!(CANONICAL_MAP[23], ".OOOOOOOOOOOO..OOOO.....");
    }

    #[test]
    fn palette_matches_the_approved_concept() {
        assert_eq!(OUTLINE, [0x2a, 0x33, 0x40, 0xff]);
        assert_eq!(FUR, [0xf4, 0xf2, 0xe8, 0xff]);
        assert_eq!(SHADOW, [0xcd, 0xd2, 0xd8, 0xff]);
        assert_eq!(EYE, [0x86, 0xd7, 0xa8, 0xff]);
        assert_eq!(PALETTE.len(), 5);
    }

    #[test]
    fn canonical_render_is_exact_and_deterministic() {
        let first = render_frame();
        let second = render_frame();
        assert_eq!(first.dimensions(), (FRAME_WIDTH, FRAME_HEIGHT));
        assert_eq!(first.as_raw(), second.as_raw());
    }

    #[test]
    fn authored_source_is_four_times_native_size() {
        assert_eq!(render_source().dimensions(), (SOURCE_WIDTH, SOURCE_HEIGHT));
        assert_eq!(SOURCE_SCALE, 4);
        assert_eq!(SOURCE_PIXEL_SIZE, 32);
    }
}

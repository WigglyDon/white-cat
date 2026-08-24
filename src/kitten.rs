//! Frozen canonical artwork for the production White Cat pet.
//!
//! `CANONICAL_MAP` is the sole authored representation. Every production
//! runtime pixel is a direct 8 x 8 nearest-neighbor expansion of this matrix.

use image::{Rgba, RgbaImage};

use crate::contract::{FRAME_HEIGHT, FRAME_WIDTH};

pub const CONCEPT_REFERENCE_FILE: &str = "concept_design_of_pixel_art_cat.png";
pub const CONCEPT_REFERENCE_SHA256: &str =
    "974bae7813b6b80a0626ca5b3d292244f5abf937f97a3f6c3102fb70180ea322";

pub const LOGICAL_WIDTH: usize = 24;
pub const LOGICAL_HEIGHT: usize = 26;
pub const RUNTIME_PIXEL_SIZE: u32 = 8;

pub const NORMALIZED_MATRIX_BYTES: usize = 650;
pub const LOGICAL_RGBA_BYTES: usize = LOGICAL_WIDTH * LOGICAL_HEIGHT * 4;
pub const NORMALIZED_MATRIX_SHA256: &str =
    "9fff8b4d54bdae285fa048ce872857e93a55ba1e034622cab5435b672e9d6735";
pub const CONTIGUOUS_SYMBOL_SHA256: &str =
    "d67509d496e5b7f1c3e61af931a4de8ae43131e0ac84230247b228ea256bd330";
pub const LOGICAL_RGBA_SHA256: &str =
    "f031e0980557f6c9cfbe6855ff18afd8b41281a93f67e473629302efe558bbac";
pub const RUNTIME_RGBA_SHA256: &str =
    "cfb50024b6b044d37a9d55f0cf995f5688a4722907cc35b7cc30a350cab7992d";

pub const TRANSPARENT: [u8; 4] = [0, 0, 0, 0];
pub const OUTLINE: [u8; 4] = [42, 51, 64, 255];
pub const BODY: [u8; 4] = [244, 242, 232, 255];
pub const SHADE: [u8; 4] = [205, 210, 216, 255];
pub const EYE: [u8; 4] = [134, 215, 168, 255];
pub const SILHOUETTE: [u8; 4] = OUTLINE;

pub const PALETTE: [(char, [u8; 4]); 5] = [
    ('.', TRANSPARENT),
    ('O', OUTLINE),
    ('B', BODY),
    ('S', SHADE),
    ('E', EYE),
];

pub const EXPECTED_SYMBOL_COUNTS: [(char, usize); 5] =
    [('.', 303), ('O', 70), ('B', 224), ('S', 23), ('E', 4)];

pub type PixelMap = [&'static str; LOGICAL_HEIGHT];

/// Exact master-supplied canonical base pose. Do not edit coordinates without
/// a replacement frozen matrix contract from the artwork authority.
pub const CANONICAL_MAP: PixelMap = [
    "........................",
    "..............OOO...OO..",
    ".............OOBO..OBO..",
    ".............OBBBO.SBO..",
    ".............OBBBOOBBO..",
    "............OBBBBBBBBOO.",
    "............OBBBBBBBBBOO",
    "...........OBBBBBBBBBBBS",
    "...........OBBBBBBBEEBBB",
    "...........OBBBBBBBEEBBB",
    "...........OBBBBBBBBBBBB",
    "..........OBBBBBBBBBBBBB",
    "..........OBBBBBBBBBBBBO",
    ".........OBBBBBBBBBBBBO.",
    ".......OOSBBBBBBBBBOO...",
    ".......SBBBBBBBBBBSO....",
    "......OBBBBBBBBBBBS.....",
    ".....OBBBBBBBBBBBBSO....",
    ".....OBBBBBBBBBBBBBS....",
    ".....OBBBBBBBBSBB.BS....",
    ".OO..OBBBBBBBBSBBOBS....",
    "OBBO.OBBBBBBBSSBBOBS....",
    "OBBBBBBBBBBBSSSBBOSBO...",
    "..SBBSOBBBBBBBOBBSSBBO..",
    ".......OOOOOOOOOOOOOO...",
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

pub fn normalized_matrix_text() -> String {
    let mut normalized = String::with_capacity(NORMALIZED_MATRIX_BYTES);
    for row in CANONICAL_MAP {
        normalized.push_str(row);
        normalized.push('\n');
    }
    normalized
}

pub fn contiguous_symbols() -> Vec<u8> {
    CANONICAL_MAP.iter().flat_map(|row| row.bytes()).collect()
}

pub fn canonical_map_is_well_formed() -> bool {
    FRAME_WIDTH == LOGICAL_WIDTH as u32 * RUNTIME_PIXEL_SIZE
        && FRAME_HEIGHT == LOGICAL_HEIGHT as u32 * RUNTIME_PIXEL_SIZE
        && CANONICAL_MAP.iter().all(|row| {
            row.len() == LOGICAL_WIDTH && row.chars().all(|symbol| palette_color(symbol).is_some())
        })
}

pub fn render_logical() -> RgbaImage {
    assert!(
        canonical_map_is_well_formed(),
        "canonical pixel map violates its fixed geometry or palette"
    );
    let mut logical = RgbaImage::new(LOGICAL_WIDTH as u32, LOGICAL_HEIGHT as u32);
    for (y, row) in CANONICAL_MAP.iter().enumerate() {
        for (x, symbol) in row.chars().enumerate() {
            logical.put_pixel(
                x as u32,
                y as u32,
                Rgba(palette_color(symbol).expect("validated canonical-map symbol")),
            );
        }
    }
    logical
}

pub fn render_frame() -> RgbaImage {
    let logical = render_logical();
    let mut frame = RgbaImage::new(FRAME_WIDTH, FRAME_HEIGHT);
    for (runtime_x, runtime_y, pixel) in frame.enumerate_pixels_mut() {
        let logical_x = runtime_x / RUNTIME_PIXEL_SIZE;
        let logical_y = runtime_y / RUNTIME_PIXEL_SIZE;
        *pixel = *logical.get_pixel(logical_x, logical_y);
    }
    frame
}

pub fn render_silhouette_logical() -> RgbaImage {
    let logical = render_logical();
    let mut silhouette = RgbaImage::new(LOGICAL_WIDTH as u32, LOGICAL_HEIGHT as u32);
    for (target, original) in silhouette.pixels_mut().zip(logical.pixels()) {
        *target = if original[3] == 0 {
            Rgba(TRANSPARENT)
        } else {
            Rgba(SILHOUETTE)
        };
    }
    silhouette
}

pub fn render_silhouette_frame() -> RgbaImage {
    let frame = render_frame();
    let mut silhouette = RgbaImage::new(FRAME_WIDTH, FRAME_HEIGHT);
    for (target, original) in silhouette.pixels_mut().zip(frame.pixels()) {
        *target = if original[3] == 0 {
            Rgba(TRANSPARENT)
        } else {
            Rgba(SILHOUETTE)
        };
    }
    silhouette
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::digest::sha256_hex;

    #[test]
    fn frozen_matrix_contract_is_exact() {
        assert!(canonical_map_is_well_formed());
        assert_eq!(CANONICAL_MAP.len(), LOGICAL_HEIGHT);
        assert!(CANONICAL_MAP.iter().all(|row| row.len() == LOGICAL_WIDTH));
        assert_eq!(normalized_matrix_text().len(), NORMALIZED_MATRIX_BYTES);
        assert_eq!(contiguous_symbols().len(), LOGICAL_WIDTH * LOGICAL_HEIGHT);
        assert_eq!(
            sha256_hex(normalized_matrix_text().as_bytes()),
            NORMALIZED_MATRIX_SHA256
        );
        assert_eq!(sha256_hex(&contiguous_symbols()), CONTIGUOUS_SYMBOL_SHA256);
        assert_eq!(sha256_hex(render_logical().as_raw()), LOGICAL_RGBA_SHA256);
        assert_eq!(CANONICAL_MAP[25], "........................");
    }

    #[test]
    fn palette_matches_the_frozen_contract() {
        assert_eq!(OUTLINE, [0x2a, 0x33, 0x40, 0xff]);
        assert_eq!(BODY, [0xf4, 0xf2, 0xe8, 0xff]);
        assert_eq!(SHADE, [0xcd, 0xd2, 0xd8, 0xff]);
        assert_eq!(EYE, [0x86, 0xd7, 0xa8, 0xff]);
        assert_eq!(PALETTE.len(), 5);
    }

    #[test]
    fn runtime_is_direct_nearest_neighbor() {
        let frame = render_frame();
        assert_eq!(frame.dimensions(), (FRAME_WIDTH, FRAME_HEIGHT));
        assert_eq!(sha256_hex(frame.as_raw()), RUNTIME_RGBA_SHA256);
        for (x, y, pixel) in frame.enumerate_pixels() {
            let expected = palette_color(
                canonical_symbol(
                    (x / RUNTIME_PIXEL_SIZE) as usize,
                    (y / RUNTIME_PIXEL_SIZE) as usize,
                )
                .expect("complete canonical matrix"),
            )
            .expect("canonical palette symbol");
            assert_eq!(pixel.0, expected, "runtime mismatch at ({x},{y})");
        }
    }
}

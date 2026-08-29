use std::fs;
use std::path::{Path, PathBuf};

use image::codecs::png::PngEncoder;
use image::{ImageEncoder, Rgba, RgbaImage};

use crate::contract::{
    EXACT_REVIEW_HEIGHT, EXACT_REVIEW_WIDTH, FRAME_COUNT, FRAME_HEIGHT, FRAME_WIDTH, GRID_COLUMNS,
    MANIFEST_FILE, SHEET_FILE, TERMINAL_CELL_HEIGHT, TERMINAL_CELL_WIDTH,
};
use crate::error::Result;
use crate::evidence;
use crate::kitten;
use crate::manifest;
use crate::sheet;

pub const REVIEW_DIRECTORY: &str = "review";
pub const DARK_REVIEW_FILE: &str = "approved-pixel-cat-dark.png";
pub const LIGHT_REVIEW_FILE: &str = "approved-pixel-cat-light.png";
pub const EXACT_REVIEW_FILE: &str = "approved-pixel-cat-70x15.png";
pub const SOURCE_REVIEW_FILE: &str = "approved-pixel-cat-source.png";
pub const SILHOUETTE_REVIEW_FILE: &str = "approved-pixel-cat-silhouette.png";
pub const ANIMATION_STORYBOARD_FILE: &str = "approved-animation-storyboard.png";
pub const IDLE_STRIP_FILE: &str = "approved-idle-strip.png";
pub const LARGE_REVIEW_WIDTH: u32 = 960;
pub const LARGE_REVIEW_HEIGHT: u32 = 320;

#[derive(Clone, Copy, Debug)]
pub enum ReviewMode {
    Dark,
    Light,
    Native,
    Silhouette,
}

impl ReviewMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Dark => "dark prompt",
            Self::Light => "light prompt",
            Self::Native => "pixel inspection",
            Self::Silhouette => "one-color silhouette",
        }
    }
}

#[derive(Debug)]
pub struct GeneratedAssets {
    pub manifest: PathBuf,
    pub sheet: PathBuf,
    pub reviews: Vec<PathBuf>,
    pub evidence: Vec<PathBuf>,
}

pub fn review_relative_paths() -> Vec<PathBuf> {
    [
        DARK_REVIEW_FILE,
        LIGHT_REVIEW_FILE,
        EXACT_REVIEW_FILE,
        SOURCE_REVIEW_FILE,
        SILHOUETTE_REVIEW_FILE,
        ANIMATION_STORYBOARD_FILE,
        IDLE_STRIP_FILE,
    ]
    .into_iter()
    .map(|name| PathBuf::from(REVIEW_DIRECTORY).join(name))
    .collect()
}

fn fill_rect(image: &mut RgbaImage, x: u32, y: u32, width: u32, height: u32, color: Rgba<u8>) {
    let x1 = x.saturating_add(width).min(image.width());
    let y1 = y.saturating_add(height).min(image.height());
    for py in y..y1 {
        for px in x..x1 {
            image.put_pixel(px, py, color);
        }
    }
}

fn overlay_opaque(destination: &mut RgbaImage, source: &RgbaImage, x: u32, y: u32) {
    for sy in 0..source.height() {
        for sx in 0..source.width() {
            let dx = x + sx;
            let dy = y + sy;
            if dx >= destination.width() || dy >= destination.height() {
                continue;
            }
            let source_pixel = source.get_pixel(sx, sy);
            let alpha = u32::from(source_pixel[3]);
            if alpha == 0 {
                continue;
            }
            let target = destination.get_pixel_mut(dx, dy);
            let inverse = 255 - alpha;
            for channel in 0..3 {
                target[channel] = ((u32::from(source_pixel[channel]) * alpha
                    + u32::from(target[channel]) * inverse
                    + 127)
                    / 255) as u8;
            }
            target[3] = 255;
        }
    }
}

pub fn solid_animation_storyboard(packed: &RgbaImage) -> RgbaImage {
    let mut canvas =
        RgbaImage::from_pixel(packed.width(), packed.height(), Rgba([13, 17, 22, 255]));
    overlay_opaque(&mut canvas, packed, 0, 0);
    canvas
}

pub fn solid_idle_strip(frames: &[RgbaImage]) -> Result<RgbaImage> {
    if frames.len() < GRID_COLUMNS as usize {
        return crate::error::fail("idle strip requires the first eight animation frames");
    }
    let mut strip = RgbaImage::from_pixel(
        FRAME_WIDTH * GRID_COLUMNS,
        FRAME_HEIGHT,
        Rgba([13, 17, 22, 255]),
    );
    for (index, frame) in frames.iter().take(GRID_COLUMNS as usize).enumerate() {
        overlay_opaque(&mut strip, frame, index as u32 * FRAME_WIDTH, 0);
    }
    Ok(strip)
}

fn prompt_canvas(frame: &RgbaImage, width: u32, height: u32, dark: bool) -> RgbaImage {
    let background = if dark {
        Rgba([13, 17, 22, 255])
    } else {
        Rgba([244, 241, 234, 255])
    };
    let panel = if dark {
        Rgba([24, 30, 37, 255])
    } else {
        Rgba([255, 253, 248, 255])
    };
    let border = if dark {
        Rgba([71, 83, 95, 255])
    } else {
        Rgba([173, 180, 184, 255])
    };
    let text = if dark {
        Rgba([135, 149, 161, 255])
    } else {
        Rgba([91, 100, 108, 255])
    };
    let accent = if dark {
        Rgba([104, 196, 151, 255])
    } else {
        Rgba([38, 126, 92, 255])
    };

    let mut canvas = RgbaImage::from_pixel(width, height, background);
    let cat_width = frame.width();
    let cat_height = frame.height();
    let cat_x = 6;
    let cat_y = height.saturating_sub(cat_height + 4);
    overlay_opaque(&mut canvas, frame, cat_x, cat_y);

    let prompt_x = cat_x + cat_width + 16;
    let prompt_width = width.saturating_sub(prompt_x + 18);
    let prompt_height = (height / 4).clamp(42, 76).min(height.saturating_sub(8));
    let prompt_y = height.saturating_sub(prompt_height + height / 7);
    if prompt_width > 20 {
        fill_rect(
            &mut canvas,
            prompt_x,
            prompt_y,
            prompt_width,
            prompt_height,
            border,
        );
        fill_rect(
            &mut canvas,
            prompt_x + 2,
            prompt_y + 2,
            prompt_width.saturating_sub(4),
            prompt_height.saturating_sub(4),
            panel,
        );
        fill_rect(
            &mut canvas,
            prompt_x + 17,
            prompt_y + prompt_height / 2 - 2,
            (prompt_width * 48 / 100).max(8),
            4,
            text,
        );
        fill_rect(
            &mut canvas,
            prompt_x + 10,
            prompt_y + prompt_height / 2 - 7,
            3,
            15,
            accent,
        );
    }
    canvas
}

pub fn dark_review(frame: &RgbaImage) -> RgbaImage {
    prompt_canvas(frame, LARGE_REVIEW_WIDTH, LARGE_REVIEW_HEIGHT, true)
}

pub fn light_review(frame: &RgbaImage) -> RgbaImage {
    prompt_canvas(frame, LARGE_REVIEW_WIDTH, LARGE_REVIEW_HEIGHT, false)
}

pub fn exact_70x15_review(frame: &RgbaImage) -> RgbaImage {
    prompt_canvas(frame, EXACT_REVIEW_WIDTH, EXACT_REVIEW_HEIGHT, true)
}

fn inspection_canvas(frame: &RgbaImage, width: u32, height: u32, silhouette: bool) -> RgbaImage {
    let mut canvas = RgbaImage::from_pixel(width, height, Rgba([16, 21, 27, 255]));
    let rendered = if silhouette {
        let mut rendered = RgbaImage::new(frame.width(), frame.height());
        for (target, original) in rendered.pixels_mut().zip(frame.pixels()) {
            *target = if original[3] == 0 {
                Rgba(crate::kitten::TRANSPARENT)
            } else {
                Rgba(crate::kitten::SILHOUETTE)
            };
        }
        rendered
    } else {
        frame.clone()
    };
    overlay_opaque(
        &mut canvas,
        &rendered,
        width.saturating_sub(rendered.width()) / 2,
        height.saturating_sub(rendered.height()) / 2,
    );
    canvas
}

pub fn terminal_review(frame: &RgbaImage, columns: u16, rows: u16, mode: ReviewMode) -> RgbaImage {
    let width = u32::from(columns) * TERMINAL_CELL_WIDTH;
    let height = u32::from(rows) * TERMINAL_CELL_HEIGHT;
    match mode {
        ReviewMode::Dark => prompt_canvas(frame, width, height, true),
        ReviewMode::Light => prompt_canvas(frame, width, height, false),
        ReviewMode::Native => inspection_canvas(frame, width, height, false),
        ReviewMode::Silhouette => inspection_canvas(frame, width, height, true),
    }
}

pub fn encode_png(image: &RgbaImage) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    PngEncoder::new(&mut bytes).write_image(
        image.as_raw(),
        image.width(),
        image.height(),
        image::ExtendedColorType::Rgba8,
    )?;
    Ok(bytes)
}

fn write_png(path: &Path, image: &RgbaImage) -> Result<()> {
    sheet::write_atomic(path, &encode_png(image)?)
}

pub fn generate_project(project: &Path) -> Result<GeneratedAssets> {
    let frame = kitten::render_frame();
    let frames = kitten::build_frames();
    if frames.len() != FRAME_COUNT {
        return crate::error::fail(format!(
            "animation contract built {} frames, expected {FRAME_COUNT}",
            frames.len()
        ));
    }
    let packed = sheet::pack_fixed_frames(&frames)?;
    manifest::write_manifest(project)?;
    sheet::write_lossless_webp(&project.join(SHEET_FILE), &packed)?;

    let review_directory = project.join(REVIEW_DIRECTORY);
    fs::create_dir_all(&review_directory)?;
    let reviews = vec![
        review_directory.join(DARK_REVIEW_FILE),
        review_directory.join(LIGHT_REVIEW_FILE),
        review_directory.join(EXACT_REVIEW_FILE),
        review_directory.join(SOURCE_REVIEW_FILE),
        review_directory.join(SILHOUETTE_REVIEW_FILE),
        review_directory.join(ANIMATION_STORYBOARD_FILE),
        review_directory.join(IDLE_STRIP_FILE),
    ];
    write_png(&reviews[0], &dark_review(&frame))?;
    write_png(&reviews[1], &light_review(&frame))?;
    write_png(&reviews[2], &exact_70x15_review(&frame))?;
    write_png(&reviews[3], &kitten::render_logical())?;
    write_png(&reviews[4], &kitten::render_silhouette_logical())?;
    write_png(&reviews[5], &solid_animation_storyboard(&packed))?;
    write_png(&reviews[6], &solid_idle_strip(&frames)?)?;

    let decoded = sheet::load_rgba(&project.join(SHEET_FILE))?;
    let report = crate::validate::validate_packed(&decoded)?;
    let evidence = evidence::write_generation_evidence(project, &decoded, &report)?;

    Ok(GeneratedAssets {
        manifest: project.join(MANIFEST_FILE),
        sheet: project.join(SHEET_FILE),
        reviews,
        evidence,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_terminal_review_has_literal_70x15_pixel_geometry() {
        assert_eq!(
            exact_70x15_review(&kitten::render_frame()).dimensions(),
            (EXACT_REVIEW_WIDTH, EXACT_REVIEW_HEIGHT)
        );
    }

    #[test]
    fn production_reviews_are_deterministic() {
        let frame = kitten::render_frame();
        assert_eq!(
            encode_png(&dark_review(&frame)).expect("first PNG"),
            encode_png(&dark_review(&frame)).expect("second PNG")
        );
    }
}

use std::env;
use std::path::PathBuf;

use white_cat::artwork;
use white_cat::contract::{
    EXACT_REVIEW_COLUMNS, EXACT_REVIEW_ROWS, FRAME_COUNT, FRAME_HEIGHT, FRAME_WIDTH, GRID_COLUMNS,
    GRID_ROWS, PET_ID, PET_SELECTOR, SHEET_HEIGHT, SHEET_WIDTH,
};
use white_cat::error::{Result, fail};
use white_cat::{install, preview, validate};

fn project_root() -> Result<PathBuf> {
    env::current_dir().map_err(Into::into)
}

fn run() -> Result<()> {
    let mut arguments = env::args().skip(1);
    let command = arguments.next().unwrap_or_else(|| "preview".to_owned());
    let project = project_root()?;
    match command.as_str() {
        "status" => {
            println!("id: {PET_ID}");
            println!("selector: {PET_SELECTOR}");
            println!("frame: {FRAME_WIDTH}x{FRAME_HEIGHT}");
            println!(
                "sheet: {SHEET_WIDTH}x{SHEET_HEIGHT} ({GRID_COLUMNS}x{GRID_ROWS}, {FRAME_COUNT} held frames)"
            );
            println!("exact review: {EXACT_REVIEW_COLUMNS}x{EXACT_REVIEW_ROWS}");
            println!("art source: src/kitten.rs");
            println!("visual authority: concept_design_of_pixel_art_cat.png");
        }
        "generate" => {
            let generated = artwork::generate_project(&project)?;
            println!("generated {}", generated.manifest.display());
            println!("generated {}", generated.sheet.display());
            for review in generated.reviews {
                println!("generated {}", review.display());
            }
        }
        "validate" => {
            validate::validate_project(&project, true)?;
            println!("White Cat production assets are valid");
        }
        "preview" | "live" | "review" => {
            artwork::generate_project(&project)?;
            validate::validate_project(&project, true)?;
            preview::run(&project)?;
        }
        "install" => {
            let force = arguments.any(|argument| argument == "--force");
            let outcome = install::install_pet(&project, force)?;
            println!("installed: {}", outcome.target.display());
            if let Some(backup) = outcome.backup {
                println!("backup: {}", backup.display());
            }
        }
        _ => {
            return fail(format!(
                "unknown command {command:?}; use status, generate, validate, preview, or install [--force]"
            ));
        }
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("White Cat failed: {error}");
        std::process::exit(1);
    }
}

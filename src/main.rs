use std::env;
use std::path::PathBuf;

use white_cat::artwork;
use white_cat::contract::{
    EXACT_REVIEW_COLUMNS, EXACT_REVIEW_ROWS, FRAME_COUNT, FRAME_HEIGHT, FRAME_WIDTH, GRID_COLUMNS,
    GRID_ROWS, PET_ID, PET_SELECTOR, SHEET_HEIGHT, SHEET_WIDTH,
};
use white_cat::error::{Result, fail};
use white_cat::{evidence, install, preview, validate};

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
            println!("art authority: frozen CANONICAL_MAP");
            println!(
                "source SHA-256: {}",
                white_cat::kitten::NORMALIZED_MATRIX_SHA256
            );
            println!(
                "runtime RGBA SHA-256: {}",
                white_cat::kitten::RUNTIME_RGBA_SHA256
            );
        }
        "generate" => {
            let generated = artwork::generate_project(&project)?;
            println!("generated {}", generated.manifest.display());
            println!("generated {}", generated.sheet.display());
            for review in generated.reviews {
                println!("generated {}", review.display());
            }
            for artifact in generated.evidence {
                println!("generated {}", artifact.display());
            }
        }
        "generate-at" => {
            let destination = arguments.next().ok_or_else(|| {
                white_cat::error::WhiteCatError::new("generate-at requires a destination path")
            })?;
            let generated = artwork::generate_project(&PathBuf::from(destination))?;
            println!("generated {}", generated.sheet.display());
        }
        "compare-generations" => {
            let left = arguments.next().ok_or_else(|| {
                white_cat::error::WhiteCatError::new("compare-generations requires two paths")
            })?;
            let right = arguments.next().ok_or_else(|| {
                white_cat::error::WhiteCatError::new("compare-generations requires two paths")
            })?;
            evidence::compare_generations(&project, &PathBuf::from(left), &PathBuf::from(right))?;
            println!("isolated generations are byte-identical");
        }
        "compare-installed" => {
            let installed = arguments
                .next()
                .map(PathBuf::from)
                .unwrap_or(install::installed_pet_path()?);
            evidence::compare_installed(&project, &installed)?;
            println!("generated and installed payloads are byte-identical");
        }
        "compare-observed-frame" => {
            let observed = arguments.next().ok_or_else(|| {
                white_cat::error::WhiteCatError::new("compare-observed-frame requires a PNG path")
            })?;
            evidence::compare_observed_frame(&project, &PathBuf::from(observed))?;
            println!("observed frame is pixel-identical to the canonical runtime frame");
        }
        "compare-cache-directory" => {
            let frames = arguments.next().ok_or_else(|| {
                white_cat::error::WhiteCatError::new(
                    "compare-cache-directory requires a frames directory",
                )
            })?;
            evidence::compare_cache_directory(&project, &PathBuf::from(frames))?;
            println!("all fresh Codex cache frames are pixel-identical");
        }
        "record-runtime-capture" => {
            let capture = arguments.next().ok_or_else(|| {
                white_cat::error::WhiteCatError::new(
                    "record-runtime-capture requires a capture path",
                )
            })?;
            evidence::record_runtime_capture(&project, &PathBuf::from(capture))?;
            println!("recorded fresh-process runtime evidence");
        }
        "validate" => {
            validate::validate_project(&project, true)?;
            println!("White Cat production assets are valid");
        }
        "validate-final" => {
            validate::validate_project(&project, true)?;
            evidence::validate_external_evidence(&project)?;
            println!("White Cat production and external evidence are valid");
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
        "install-to" => {
            let root = arguments.next().ok_or_else(|| {
                white_cat::error::WhiteCatError::new("install-to requires a Codex root path")
            })?;
            let force = arguments.any(|argument| argument == "--force");
            let outcome = install::install_pet_to_root(&project, &PathBuf::from(root), force)?;
            println!("installed: {}", outcome.target.display());
            if let Some(backup) = outcome.backup {
                println!("backup: {}", backup.display());
            }
        }
        _ => {
            return fail(format!(
                "unknown command {command:?}; use status, generate, validate, validate-final, preview, install [--force], install-to, generate-at, compare-generations, compare-installed, compare-observed-frame, compare-cache-directory, or record-runtime-capture"
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

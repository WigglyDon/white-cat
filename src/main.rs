use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use white_cat::contract::{
    FRAME_COUNT, FRAME_HEIGHT, FRAME_WIDTH, PIXEL_SCALE, SHEET_HEIGHT, SHEET_WIDTH, SOURCE_HEIGHT,
    SOURCE_WIDTH,
};
use white_cat::install::install_pet;
use white_cat::manifest::generate;
use white_cat::preview::{render_plain, render_terminal, run_live};
use white_cat::validate::validate_project;
use white_cat::{Result, WhiteCatError};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn usage() {
    eprintln!(
        "White Cat pure-Rust toolchain\n\n\
         usage: white-cat <command> [options]\n\n\
         commands:\n\
           status                         show scaffold/runtime state\n\
           preview [--state S] [--frame N] [--plain]\n\
           live [--state S] [--source PATH]\n\
           generate [--project-dir PATH]\n\
           validate [--project-dir PATH] [--no-source-check]\n\
           install [--project-dir PATH] [--force]"
    );
}

fn value_after(args: &[String], index: &mut usize, option: &str) -> Result<String> {
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| WhiteCatError::new(format!("{option} requires a value")))
}

fn parse_project_dir(args: &[String], allow_force: bool) -> Result<(PathBuf, bool, bool)> {
    let mut project = project_root();
    let mut force = false;
    let mut no_source_check = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--project-dir" => {
                project = PathBuf::from(value_after(args, &mut index, "--project-dir")?)
            }
            "--force" if allow_force => force = true,
            "--no-source-check" if !allow_force => no_source_check = true,
            option => return Err(WhiteCatError::new(format!("unknown option {option:?}"))),
        }
        index += 1;
    }
    Ok((project, force, no_source_check))
}

fn command_status() {
    let root = project_root();
    let manifest = root.join("pet.json").is_file();
    let sheet = root.join("spritesheet.webp").is_file();
    println!("White Cat implementation: pure Rust");
    println!("White Cat design: approved simplified pixel-map concept");
    println!(
        "White Cat runtime assets: {}",
        match (manifest, sheet) {
            (true, true) => "built",
            (false, false) => "not built",
            _ => "incomplete",
        }
    );
    println!(
        "White Cat source: {SOURCE_WIDTH}x{SOURCE_HEIGHT} Rust string arrays at fixed {PIXEL_SCALE}x scale"
    );
}

fn command_preview(args: &[String], live: bool) -> Result<()> {
    let mut source = project_root().join("src/maps.rs");
    let mut state = "idle".to_owned();
    let mut frame = 0_usize;
    let mut plain = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--source" => source = PathBuf::from(value_after(args, &mut index, "--source")?),
            "--state" => state = value_after(args, &mut index, "--state")?,
            "--frame" if !live => {
                let value = value_after(args, &mut index, "--frame")?;
                frame = value.parse().map_err(|_| {
                    WhiteCatError::new(format!("--frame must be a non-negative integer: {value}"))
                })?;
            }
            "--plain" if !live => plain = true,
            option => return Err(WhiteCatError::new(format!("unknown option {option:?}"))),
        }
        index += 1;
    }

    if live {
        run_live(&source, &state)
    } else {
        let artwork = white_cat::artwork::load_maps_source(&source)?;
        let rendered = if plain {
            render_plain(&artwork, &state, frame)?
        } else {
            render_terminal(&artwork, &state, frame, "static Rust source preview")?
        };
        println!("{rendered}");
        Ok(())
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let Some(command) = args.first().map(String::as_str) else {
        usage();
        return Err(WhiteCatError::new("missing command"));
    };
    let options = &args[1..];
    match command {
        "status" => {
            if !options.is_empty() {
                return Err(WhiteCatError::new("status takes no options"));
            }
            command_status();
            Ok(())
        }
        "preview" => command_preview(options, false),
        "live" => command_preview(options, true),
        "generate" => {
            let (project, _, _) = parse_project_dir(options, false)?;
            let (manifest, sheet) = generate(&project)?;
            println!("Generated {}", manifest.display());
            println!("Generated {}", sheet.display());
            println!(
                "Source {SOURCE_WIDTH}x{SOURCE_HEIGHT}; scale {PIXEL_SCALE}x nearest; runtime {FRAME_WIDTH}x{FRAME_HEIGHT}; frames {FRAME_COUNT}"
            );
            Ok(())
        }
        "validate" => {
            let (project, _, no_source_check) = parse_project_dir(options, false)?;
            validate_project(&project, !no_source_check)?;
            println!(
                "Validated {FRAME_COUNT} fixed RGBA frames; sheet {SHEET_WIDTH}x{SHEET_HEIGHT}"
            );
            Ok(())
        }
        "install" => {
            let (project, force, _) = parse_project_dir(options, true)?;
            let outcome = install_pet(&project, force)?;
            if let Some(backup) = outcome.backup {
                println!("Previous installation backed up to: {}", backup.display());
            }
            println!("Installed White Cat to: {}", outcome.target.display());
            println!("Select White Cat from /pets; config.toml was not modified.");
            Ok(())
        }
        "help" | "--help" | "-h" => {
            usage();
            Ok(())
        }
        other => {
            usage();
            Err(WhiteCatError::new(format!("unknown command {other:?}")))
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("White Cat failed: {error}");
            ExitCode::FAILURE
        }
    }
}

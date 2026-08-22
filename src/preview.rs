use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::time::{Duration, Instant, SystemTime};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

use crate::artwork::{ParsedArtwork, frame_pose_names, load_maps_source, parsed_source_pose};
use crate::contract::{
    PIXEL_SCALE, PRIMARY_STATES, SOURCE_HEIGHT, SOURCE_WIDTH, animation_fps, resolve_state,
};
use crate::{Result, WhiteCatError};

const RESET: &str = "\x1b[0m";

fn foreground(pixel: char) -> Result<String> {
    let (red, green, blue) = match pixel {
        'O' => (42, 51, 64),
        'W' => (244, 242, 232),
        'S' => (205, 210, 216),
        'E' => (134, 215, 168),
        other => {
            return Err(WhiteCatError::new(format!(
                "unknown visible pixel {other:?}"
            )));
        }
    };
    Ok(format!("\x1b[38;2;{red};{green};{blue}m"))
}

fn background(pixel: char) -> Result<String> {
    let (red, green, blue) = match pixel {
        'O' => (42, 51, 64),
        'W' => (244, 242, 232),
        'S' => (205, 210, 216),
        'E' => (134, 215, 168),
        other => {
            return Err(WhiteCatError::new(format!(
                "unknown visible pixel {other:?}"
            )));
        }
    };
    Ok(format!("\x1b[48;2;{red};{green};{blue}m"))
}

fn terminal_pixel_pair(top: char, bottom: char) -> Result<String> {
    match (top == '.', bottom == '.') {
        (true, true) => Ok(format!("{RESET} ")),
        (false, true) => Ok(format!("{RESET}{}▀", foreground(top)?)),
        (true, false) => Ok(format!("{RESET}{}▄", foreground(bottom)?)),
        (false, false) => Ok(format!(
            "{RESET}{}{}▀",
            foreground(top)?,
            background(bottom)?
        )),
    }
}

pub fn render_plain(artwork: &ParsedArtwork, state: &str, frame_index: usize) -> Result<String> {
    let resolved = resolve_state(state)
        .ok_or_else(|| WhiteCatError::new(format!("unknown state {state:?}")))?;
    let names = frame_pose_names(resolved)?;
    let selected = frame_index % names.len();
    let rows = parsed_source_pose(artwork, resolved, selected)?;
    let mut output = format!(
        "White Cat | state={resolved} | frame={}/{} | source={}x{} | runtime={}x{}",
        selected + 1,
        names.len(),
        SOURCE_WIDTH,
        SOURCE_HEIGHT,
        SOURCE_WIDTH * PIXEL_SCALE,
        SOURCE_HEIGHT * PIXEL_SCALE
    );
    for row in rows {
        output.push('\n');
        output.push_str(row);
    }
    Ok(output)
}

pub fn render_terminal(
    artwork: &ParsedArtwork,
    state: &str,
    frame_index: usize,
    status: &str,
) -> Result<String> {
    let resolved = resolve_state(state)
        .ok_or_else(|| WhiteCatError::new(format!("unknown state {state:?}")))?;
    let names = frame_pose_names(resolved)?;
    let selected = frame_index % names.len();
    let rows = parsed_source_pose(artwork, resolved, selected)?;
    let fps = animation_fps(resolved).expect("known state");
    let mut lines = vec![
        format!(
            "White Cat live  state {resolved}  frame {}/{}  {fps} FPS  {}x{} -> {}x{}",
            selected + 1,
            names.len(),
            SOURCE_WIDTH,
            SOURCE_HEIGHT,
            SOURCE_WIDTH * PIXEL_SCALE,
            SOURCE_HEIGHT * PIXEL_SCALE
        ),
        "n/p state  space pause  r reload  q quit".to_owned(),
        status.to_owned(),
        String::new(),
    ];

    for row_index in (0..SOURCE_HEIGHT).step_by(2) {
        let top: Vec<char> = rows[row_index].chars().collect();
        let bottom: Vec<char> = rows[row_index + 1].chars().collect();
        let mut line = String::new();
        for (top_pixel, bottom_pixel) in top.into_iter().zip(bottom) {
            line.push_str(&terminal_pixel_pair(top_pixel, bottom_pixel)?);
        }
        line.push_str(RESET);
        lines.push(line);
    }
    Ok(lines.join("\n"))
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> Result<Self> {
        if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
            return Err(WhiteCatError::new(
                "live mode requires a terminal; use preview for one static frame",
            ));
        }
        enable_raw_mode()?;
        if let Err(error) = execute!(io::stdout(), EnterAlternateScreen, Hide) {
            let _ = disable_raw_mode();
            return Err(error.into());
        }
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SourceStamp {
    modified: Option<SystemTime>,
    length: u64,
}

fn source_stamp(path: &Path) -> Result<SourceStamp> {
    let metadata = fs::metadata(path)?;
    Ok(SourceStamp {
        modified: metadata.modified().ok(),
        length: metadata.len(),
    })
}

fn draw(
    output: &mut io::Stdout,
    artwork: &ParsedArtwork,
    state: &str,
    frame_index: usize,
    status: &str,
) -> Result<()> {
    execute!(output, MoveTo(0, 0), Clear(ClearType::All))?;
    output.write_all(render_terminal(artwork, state, frame_index, status)?.as_bytes())?;
    output.flush()?;
    Ok(())
}

pub fn run_live(source: &Path, initial_state: &str) -> Result<()> {
    let mut artwork = load_maps_source(source)?;
    let mut state = resolve_state(initial_state)
        .ok_or_else(|| WhiteCatError::new(format!("unknown state {initial_state:?}")))?;
    let mut state_index = PRIMARY_STATES
        .iter()
        .position(|candidate| *candidate == state)
        .expect("primary resolved state");
    let mut frame_index = 0;
    let mut paused = false;
    let mut status = format!("watching {}", source.display());
    let mut observed = source_stamp(source)?;
    let mut next_frame =
        Instant::now() + Duration::from_secs_f64(1.0 / animation_fps(state).unwrap() as f64);
    let _terminal = TerminalGuard::enter()?;
    let mut output = io::stdout();
    draw(&mut output, &artwork, state, frame_index, &status)?;

    loop {
        if let Ok(current) = source_stamp(source)
            && current != observed
        {
            observed = current;
            match load_maps_source(source) {
                Ok(refreshed) => {
                    artwork = refreshed;
                    frame_index = 0;
                    status = "reloaded valid Rust pixel maps".to_owned();
                    next_frame = Instant::now()
                        + Duration::from_secs_f64(1.0 / animation_fps(state).unwrap() as f64);
                }
                Err(error) => {
                    status = format!("reload failed; showing last valid maps: {error}");
                }
            }
            draw(&mut output, &artwork, state, frame_index, &status)?;
        }

        if event::poll(Duration::from_millis(20))?
            && let Event::Key(key) = event::read()?
        {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if key.code == KeyCode::Char('q')
                || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
            {
                return Ok(());
            }
            match key.code {
                KeyCode::Char('n') => {
                    state_index = (state_index + 1) % PRIMARY_STATES.len();
                    state = PRIMARY_STATES[state_index];
                    frame_index = 0;
                    status = format!("state {state}");
                }
                KeyCode::Char('p') => {
                    state_index = (state_index + PRIMARY_STATES.len() - 1) % PRIMARY_STATES.len();
                    state = PRIMARY_STATES[state_index];
                    frame_index = 0;
                    status = format!("state {state}");
                }
                KeyCode::Char(' ') => {
                    paused = !paused;
                    status = if paused { "paused" } else { "playing" }.to_owned();
                }
                KeyCode::Char('r') => match load_maps_source(source) {
                    Ok(refreshed) => {
                        artwork = refreshed;
                        frame_index = 0;
                        status = "reloaded valid Rust pixel maps".to_owned();
                    }
                    Err(error) => {
                        status = format!("reload failed; showing last valid maps: {error}");
                    }
                },
                _ => continue,
            }
            next_frame = Instant::now()
                + Duration::from_secs_f64(1.0 / animation_fps(state).unwrap() as f64);
            draw(&mut output, &artwork, state, frame_index, &status)?;
        }

        let now = Instant::now();
        if !paused && now >= next_frame {
            let frame_count = frame_pose_names(state)?.len();
            frame_index = (frame_index + 1) % frame_count;
            next_frame = now + Duration::from_secs_f64(1.0 / animation_fps(state).unwrap() as f64);
            draw(&mut output, &artwork, state, frame_index, &status)?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn artwork() -> ParsedArtwork {
        crate::artwork::parse_maps_source(include_str!("maps.rs")).unwrap()
    }

    #[test]
    fn plain_preview_is_exactly_one_source_map() {
        let rendered = render_plain(&artwork(), "idle", 0).unwrap();
        let lines: Vec<&str> = rendered.lines().collect();
        assert_eq!(lines.len(), 27);
        assert!(lines[1..].iter().all(|row| row.len() == 24));
    }

    #[test]
    fn terminal_preview_uses_truecolor_half_blocks() {
        let rendered = render_terminal(&artwork(), "idle", 0, "test").unwrap();
        assert!(rendered.contains("\x1b[38;2;42;51;64m"));
        assert!(rendered.contains("\x1b[38;2;244;242;232m"));
        assert!(rendered.contains('▀'));
        assert_eq!(rendered.matches('\n').count(), 16);
    }

    #[test]
    fn invalid_state_is_clear() {
        let error = render_plain(&artwork(), "seven-white-cats", 0).unwrap_err();
        assert!(error.to_string().contains("unknown state"));
    }

    #[test]
    fn invalid_edit_reports_a_parse_error() {
        let error = crate::artwork::parse_maps_source("pub const POSES: &[()] = &[];").unwrap_err();
        assert!(error.to_string().contains("defines no poses"));
    }
}

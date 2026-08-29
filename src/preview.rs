use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::time::{Duration, Instant};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
    enable_raw_mode,
};

use crate::artwork::{self, ReviewMode};
use crate::contract::{
    SHEET_FILE, STATES, TERMINAL_CELL_HEIGHT, TERMINAL_CELL_WIDTH, animation_fps,
    animation_timeline,
};
use crate::error::{Result, fail};
use crate::sheet;

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> Result<Self> {
        enable_raw_mode()?;
        execute!(
            io::stdout(),
            EnterAlternateScreen,
            Hide,
            Clear(ClearType::All)
        )?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(
            io::stdout(),
            Clear(ClearType::All),
            MoveTo(0, 0),
            Show,
            LeaveAlternateScreen
        );
        let _ = disable_raw_mode();
    }
}

fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let a = u32::from(chunk[0]);
        let b = u32::from(*chunk.get(1).unwrap_or(&0));
        let c = u32::from(*chunk.get(2).unwrap_or(&0));
        let value = (a << 16) | (b << 8) | c;
        encoded.push(TABLE[((value >> 18) & 63) as usize] as char);
        encoded.push(TABLE[((value >> 12) & 63) as usize] as char);
        encoded.push(if chunk.len() > 1 {
            TABLE[((value >> 6) & 63) as usize] as char
        } else {
            '='
        });
        encoded.push(if chunk.len() > 2 {
            TABLE[(value & 63) as usize] as char
        } else {
            '='
        });
    }
    encoded
}

fn kitty_available() -> bool {
    env::var_os("KITTY_WINDOW_ID").is_some()
        || env::var("TERM")
            .map(|terminal| terminal.contains("kitty"))
            .unwrap_or(false)
}

fn display_png(bytes: &[u8], columns: u16, rows: u16) -> Result<()> {
    let payload = base64(bytes);
    let chunks: Vec<&[u8]> = payload.as_bytes().chunks(4096).collect();
    let mut output = io::stdout().lock();
    write!(output, "\x1b_Ga=d,d=A,q=2;\x1b\\")?;
    for (index, chunk) in chunks.iter().enumerate() {
        let more = u8::from(index + 1 < chunks.len());
        let chunk = std::str::from_utf8(chunk).expect("base64 is ASCII");
        if index == 0 {
            write!(
                output,
                "\x1b_Ga=T,f=100,t=d,q=2,i=77,c={columns},r={rows},m={more};{chunk}\x1b\\"
            )?;
        } else {
            write!(output, "\x1b_Gm={more};{chunk}\x1b\\")?;
        }
    }
    output.flush()?;
    Ok(())
}

fn draw(
    packed: &image::RgbaImage,
    mode: ReviewMode,
    state_index: usize,
    timeline_position: usize,
) -> Result<()> {
    let (columns, rows) = terminal::size()?;
    if columns < 36 || rows < 15 {
        execute!(io::stdout(), Clear(ClearType::All), MoveTo(0, 0))?;
        print!("White Cat review needs 36x15; terminal is {columns}x{rows}. Q: quit");
        io::stdout().flush()?;
        return Ok(());
    }
    let image_rows = rows - 1;
    let state = STATES[state_index];
    let timeline = animation_timeline(state.name).expect("declared animation timeline");
    let frame_index = timeline[timeline_position];
    let frame = sheet::extract_frame(packed, frame_index)?;
    let canvas = artwork::terminal_review(&frame, columns, image_rows, mode);
    let expected_dimensions = (
        u32::from(columns) * TERMINAL_CELL_WIDTH,
        u32::from(image_rows) * TERMINAL_CELL_HEIGHT,
    );
    if canvas.dimensions() != expected_dimensions {
        return fail("terminal review renderer returned incorrect dimensions");
    }
    display_png(&artwork::encode_png(&canvas)?, columns, image_rows)?;
    execute!(
        io::stdout(),
        MoveTo(0, image_rows),
        Clear(ClearType::CurrentLine)
    )?;
    print!(
        "White Cat | {} frame {} | {} | ←/→ state D/L/N/S R Q",
        state.name,
        frame_index,
        mode.label(),
    );
    io::stdout().flush()?;
    Ok(())
}

pub fn run(project: &Path) -> Result<()> {
    if !kitty_available() {
        return fail("production review requires a Kitty graphics-capable terminal");
    }
    let _guard = TerminalGuard::enter()?;
    let mut packed = sheet::load_rgba(&project.join(SHEET_FILE))?;
    let mut mode = ReviewMode::Dark;
    let mut state_index = 0usize;
    let mut timeline_position = 0usize;
    let mut last_frame_at = Instant::now();
    draw(&packed, mode, state_index, timeline_position)?;
    loop {
        let frame_duration = Duration::from_secs_f64(
            1.0 / f64::from(animation_fps(STATES[state_index].name).expect("declared fps")),
        );
        let timeout = frame_duration.saturating_sub(last_frame_at.elapsed());
        if event::poll(timeout)? {
            match event::read()? {
                Event::Resize(_, _) => draw(&packed, mode, state_index, timeline_position)?,
                Event::Key(key) if key.kind != KeyEventKind::Release => match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        mode = ReviewMode::Dark;
                        draw(&packed, mode, state_index, timeline_position)?;
                    }
                    KeyCode::Char('l') | KeyCode::Char('L') => {
                        mode = ReviewMode::Light;
                        draw(&packed, mode, state_index, timeline_position)?;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        mode = ReviewMode::Native;
                        draw(&packed, mode, state_index, timeline_position)?;
                    }
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        mode = ReviewMode::Silhouette;
                        draw(&packed, mode, state_index, timeline_position)?;
                    }
                    KeyCode::Left => {
                        state_index = state_index.checked_sub(1).unwrap_or(STATES.len() - 1);
                        timeline_position = 0;
                        last_frame_at = Instant::now();
                        draw(&packed, mode, state_index, timeline_position)?;
                    }
                    KeyCode::Right => {
                        state_index = (state_index + 1) % STATES.len();
                        timeline_position = 0;
                        last_frame_at = Instant::now();
                        draw(&packed, mode, state_index, timeline_position)?;
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        packed = sheet::load_rgba(&project.join(SHEET_FILE))?;
                        timeline_position = 0;
                        last_frame_at = Instant::now();
                        draw(&packed, mode, state_index, timeline_position)?;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        if last_frame_at.elapsed() >= frame_duration {
            let timeline =
                animation_timeline(STATES[state_index].name).expect("declared animation timeline");
            timeline_position += 1;
            if timeline_position == timeline.len() {
                if STATES[state_index].loops {
                    timeline_position = 0;
                } else {
                    state_index = 0;
                    timeline_position = 0;
                }
            }
            last_frame_at = Instant::now();
            draw(&packed, mode, state_index, timeline_position)?;
        }
    }
    Ok(())
}

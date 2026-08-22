use std::env;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::time::{Duration, Instant, SystemTime};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
    enable_raw_mode, size,
};
use crossterm::{SynchronizedUpdate, execute, queue};
use image::ExtendedColorType;
use image::ImageEncoder as _;
use image::codecs::png::PngEncoder;

use crate::artwork::{
    ParsedArtwork, frame_pose_names, load_maps_source, parsed_source_pose, render_pose,
};
use crate::contract::{
    COLUMNS, FRAME_HEIGHT, FRAME_WIDTH, PIXEL_SCALE, PRIMARY_STATES, SOURCE_HEIGHT, SOURCE_WIDTH,
    animation_fps, animation_range, animation_timeline, resolve_state,
};
use crate::maps::AUDITION_CANDIDATES;
use crate::{Result, WhiteCatError};

const PET_TARGET_HEIGHT_PX: u16 = 75;
const TERMINAL_ROW_HEIGHT_PX: u16 = 15;
const PET_IMAGE_ID: u32 = 0x5743_4154;
const KITTY_CHUNK_SIZE: usize = 4096;
const VIEW_COLUMNS: usize = 40;
const VIEW_ROWS: usize = 11;

fn pet_image_size() -> (u16, u16) {
    let rows = (f64::from(PET_TARGET_HEIGHT_PX) / f64::from(TERMINAL_ROW_HEIGHT_PX))
        .round()
        .max(1.0) as u16;
    let aspect = FRAME_HEIGHT as f64 / FRAME_WIDTH as f64 * 0.52;
    let columns = (f64::from(rows) / aspect).round().max(1.0) as u16;
    (columns, rows)
}

fn kitty_graphics_available() -> bool {
    if env::var_os("TMUX").is_some()
        || env::var_os("TMUX_PANE").is_some()
        || env::var_os("ZELLIJ").is_some()
        || env::var_os("ZELLIJ_SESSION_NAME").is_some()
    {
        return false;
    }
    if env::var_os("KITTY_WINDOW_ID").is_some()
        || env::var_os("WEZTERM_EXECUTABLE").is_some()
        || env::var_os("WEZTERM_VERSION").is_some()
    {
        return true;
    }
    [env::var("TERM").ok(), env::var("TERM_PROGRAM").ok()]
        .into_iter()
        .flatten()
        .any(|value| {
            let value = value.to_ascii_lowercase();
            value.contains("kitty") || value.contains("ghostty") || value.contains("wezterm")
        })
}

fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        output.push(TABLE[(first >> 2) as usize] as char);
        output.push(TABLE[(((first & 0x03) << 4) | (second >> 4)) as usize] as char);
        if chunk.len() > 1 {
            output.push(TABLE[(((second & 0x0f) << 2) | (third >> 6)) as usize] as char);
        } else {
            output.push('=');
        }
        if chunk.len() > 2 {
            output.push(TABLE[(third & 0x3f) as usize] as char);
        } else {
            output.push('=');
        }
    }
    output
}

fn encode_pose_png(artwork: &ParsedArtwork, pose_name: &str) -> Result<Vec<u8>> {
    let rows = artwork.pose(pose_name)?;
    let frame = render_pose(pose_name, rows)?;
    let mut png = Vec::new();
    PngEncoder::new(&mut png).write_image(
        frame.as_raw(),
        FRAME_WIDTH as u32,
        FRAME_HEIGHT as u32,
        ExtendedColorType::Rgba8,
    )?;
    Ok(png)
}

fn kitty_delete_image() -> String {
    format!("\x1b_Ga=d,d=I,i={PET_IMAGE_ID},q=2;\x1b\\")
}

fn kitty_transmit_png(png: &[u8], columns: u16, rows: u16) -> String {
    let payload = encode_base64(png);
    let chunks = payload
        .as_bytes()
        .chunks(KITTY_CHUNK_SIZE)
        .collect::<Vec<_>>();
    let mut command = String::new();
    for (index, chunk) in chunks.iter().enumerate() {
        let chunk = std::str::from_utf8(chunk).expect("base64 is ASCII");
        let more = u8::from(index + 1 < chunks.len());
        if index == 0 {
            command.push_str(&format!(
                "\x1b_Ga=T,t=d,f=100,c={columns},r={rows},q=2,i={PET_IMAGE_ID},m={more};{chunk}\x1b\\"
            ));
        } else {
            command.push_str(&format!("\x1b_Gm={more};{chunk}\x1b\\"));
        }
    }
    command
}

fn status_text(paused: bool) -> String {
    if paused { "paused" } else { "playing" }.to_owned()
}

fn compact_status(status: &str) -> String {
    const LIMIT: usize = 20;
    if status.chars().count() <= LIMIT {
        return status.to_owned();
    }
    let mut compact: String = status.chars().take(LIMIT - 1).collect();
    compact.push('…');
    compact
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ViewerCatalog {
    Audition,
    RuntimeStates,
}

#[derive(Clone, Copy)]
struct ViewerItem {
    label: &'static str,
    poses: &'static [&'static str; COLUMNS],
    fps: u64,
    timeline: &'static [usize],
    sprite_offset: usize,
}

impl ViewerItem {
    fn source_frame(self, timeline_index: usize) -> Result<usize> {
        let sprite_index = self.timeline[timeline_index % self.timeline.len()];
        let Some(source_frame) = sprite_index.checked_sub(self.sprite_offset) else {
            return Err(WhiteCatError::new(format!(
                "{} timeline frame {sprite_index} precedes its allocation",
                self.label
            )));
        };
        if source_frame >= self.poses.len() {
            return Err(WhiteCatError::new(format!(
                "{} timeline frame {sprite_index} is outside its allocation",
                self.label
            )));
        }
        Ok(source_frame)
    }

    fn timeline_index_for_source_frame(self, source_frame: usize) -> Result<usize> {
        let sprite_index = self.sprite_offset + source_frame;
        self.timeline
            .iter()
            .position(|candidate| *candidate == sprite_index)
            .ok_or_else(|| {
                WhiteCatError::new(format!(
                    "{} source frame {source_frame} is absent from its timeline",
                    self.label
                ))
            })
    }

    fn frame_duration(self) -> Duration {
        Duration::from_secs_f64(1.0 / self.fps as f64)
    }
}

impl ViewerCatalog {
    fn len(self) -> usize {
        match self {
            Self::Audition => AUDITION_CANDIDATES.len(),
            Self::RuntimeStates => PRIMARY_STATES.len(),
        }
    }

    fn item(self, index: usize) -> Result<ViewerItem> {
        match self {
            Self::Audition => {
                let candidate = AUDITION_CANDIDATES.get(index).ok_or_else(|| {
                    WhiteCatError::new(format!("unknown audition candidate {}", index + 1))
                })?;
                Ok(ViewerItem {
                    label: candidate.display_name,
                    poses: &candidate.poses,
                    fps: animation_fps("idle").expect("idle fps"),
                    timeline: animation_timeline("idle").expect("idle timeline"),
                    sprite_offset: 0,
                })
            }
            Self::RuntimeStates => {
                let state = *PRIMARY_STATES.get(index).ok_or_else(|| {
                    WhiteCatError::new(format!("unknown runtime state {}", index + 1))
                })?;
                let allocation = animation_range(state).expect("primary state allocation");
                Ok(ViewerItem {
                    label: state,
                    poses: frame_pose_names(state)?,
                    fps: animation_fps(state).expect("primary state fps"),
                    timeline: animation_timeline(state).expect("primary state timeline"),
                    sprite_offset: allocation.start,
                })
            }
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Audition => "White Cat | live audition",
            Self::RuntimeStates => "White Cat | Codex runtime preview",
        }
    }

    fn controls(self) -> &'static str {
        match self {
            Self::Audition => "Up/Down cat | Left/Right variation",
            Self::RuntimeStates => "Up/Down state | Left/Right variation",
        }
    }

    fn status_line(self, item_index: usize, frame_index: usize, status: &str) -> Result<String> {
        let item = self.item(item_index)?;
        Ok(match self {
            Self::Audition => format!(
                "{}/{} {} | {}/{} | {} fps | {}",
                item_index + 1,
                self.len(),
                item.label,
                frame_index + 1,
                item.poses.len(),
                item.fps,
                compact_status(status)
            ),
            Self::RuntimeStates => format!(
                "{} | {}/{} | {} fps | {}",
                item.label,
                frame_index + 1,
                item.poses.len(),
                item.fps,
                compact_status(status)
            ),
        })
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

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> Result<Self> {
        if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
            return Err(WhiteCatError::new(
                "live preview requires an interactive terminal",
            ));
        }
        if !kitty_graphics_available() {
            return Err(WhiteCatError::new(
                "live preview requires the Kitty image protocol used by Codex in this terminal",
            ));
        }
        enable_raw_mode()?;
        let mut output = io::stdout();
        if let Err(error) = execute!(output, EnterAlternateScreen, Hide) {
            let _ = disable_raw_mode();
            return Err(error.into());
        }
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut output = io::stdout();
        let _ = output.write_all(kitty_delete_image().as_bytes());
        let _ = execute!(output, Show, LeaveAlternateScreen);
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
    catalog: ViewerCatalog,
    item_index: usize,
    frame_index: usize,
    status: &str,
    full_redraw: bool,
) -> Result<()> {
    let (columns, rows) = size()?;
    let too_small = usize::from(columns) < VIEW_COLUMNS || usize::from(rows) < VIEW_ROWS;
    let item = catalog.item(item_index)?;
    let selected = frame_index % item.poses.len();
    let png = (!too_small)
        .then(|| encode_pose_png(artwork, item.poses[selected]))
        .transpose()?;

    output.sync_update(|output| -> Result<()> {
        output.write_all(kitty_delete_image().as_bytes())?;
        if full_redraw || too_small {
            queue!(output, MoveTo(0, 0), Clear(ClearType::All))?;
        }

        if too_small {
            write_at(output, 0, 0, "White Cat preview")?;
            write_at(
                output,
                0,
                1.min(rows.saturating_sub(1)),
                &format!("terminal is {columns}x{rows}; needs {VIEW_COLUMNS}x{VIEW_ROWS}"),
            )?;
            write_at(output, 0, 2.min(rows.saturating_sub(1)), "Q quits")?;
            return Ok(());
        }

        let panel_x = (columns - VIEW_COLUMNS as u16) / 2;
        if full_redraw {
            write_at(output, panel_x, 0, catalog.title())?;
            write_at(output, panel_x, 9, catalog.controls())?;
            write_at(output, panel_x, 10, "Space play/pause | R reload | Q quit")?;
        }
        queue!(output, MoveTo(panel_x, 1), Clear(ClearType::CurrentLine))?;
        write_at(
            output,
            panel_x,
            1,
            &catalog.status_line(item_index, selected, status)?,
        )?;

        let png = png.as_deref().expect("normal layout has an encoded frame");
        let (image_columns, image_rows) = pet_image_size();
        let image_x = panel_x + (VIEW_COLUMNS as u16 - image_columns) / 2;
        queue!(output, MoveTo(image_x, 3))?;
        output.write_all(kitty_transmit_png(png, image_columns, image_rows).as_bytes())?;
        queue!(output, MoveTo(0, 0))?;
        Ok(())
    })??;
    Ok(())
}

fn write_at(output: &mut impl Write, x: u16, y: u16, text: &str) -> io::Result<()> {
    queue!(output, MoveTo(x, y))?;
    output.write_all(text.as_bytes())?;
    Ok(())
}

pub fn run_live(source: &Path, initial_candidate: usize) -> Result<()> {
    if initial_candidate >= ViewerCatalog::Audition.len() {
        return Err(WhiteCatError::new(format!(
            "candidate must be between 1 and {}",
            ViewerCatalog::Audition.len()
        )));
    }
    run_viewer(
        source,
        ViewerCatalog::Audition,
        initial_candidate,
        /*initial_frame*/ 0,
    )
}

pub fn run_preview(source: &Path, initial_state: &str, initial_frame: usize) -> Result<()> {
    let state = resolve_state(initial_state)
        .ok_or_else(|| WhiteCatError::new(format!("unknown state {initial_state:?}")))?;
    let state_index = PRIMARY_STATES
        .iter()
        .position(|candidate| *candidate == state)
        .expect("resolved primary state");
    run_viewer(
        source,
        ViewerCatalog::RuntimeStates,
        state_index,
        initial_frame,
    )
}

fn run_viewer(
    source: &Path,
    catalog: ViewerCatalog,
    initial_item_index: usize,
    initial_frame: usize,
) -> Result<()> {
    let mut artwork = load_maps_source(source)?;
    let mut item_index = initial_item_index % catalog.len();
    let mut item = catalog.item(item_index)?;
    let mut frame_index = initial_frame % item.poses.len();
    let mut timeline_index = item.timeline_index_for_source_frame(frame_index)?;
    let mut paused = false;
    let mut status = status_text(paused);
    let mut observed = source_stamp(source)?;
    let mut next_frame = Instant::now() + item.frame_duration();
    let _terminal = TerminalGuard::enter()?;
    let mut output = io::stdout();
    draw(
        &mut output,
        &artwork,
        catalog,
        item_index,
        frame_index,
        &status,
        true,
    )?;

    loop {
        if let Ok(current) = source_stamp(source)
            && current != observed
        {
            observed = current;
            match load_maps_source(source) {
                Ok(refreshed) => {
                    artwork = refreshed;
                    timeline_index = 0;
                    item = catalog.item(item_index)?;
                    frame_index = item.source_frame(timeline_index)?;
                    status = "reloaded valid Rust pixel maps".to_owned();
                    next_frame = Instant::now() + item.frame_duration();
                }
                Err(error) => {
                    status = format!("reload failed; showing last valid maps: {error}");
                }
            }
            draw(
                &mut output,
                &artwork,
                catalog,
                item_index,
                frame_index,
                &status,
                false,
            )?;
        }

        if event::poll(Duration::from_millis(20))? {
            let key = match event::read()? {
                Event::Resize(_, _) => {
                    draw(
                        &mut output,
                        &artwork,
                        catalog,
                        item_index,
                        frame_index,
                        &status,
                        true,
                    )?;
                    continue;
                }
                Event::Key(key) => key,
                _ => continue,
            };
            if key.kind == KeyEventKind::Release {
                continue;
            }
            if key.code == KeyCode::Char('q')
                || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
            {
                return Ok(());
            }
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if item_index == 0 {
                        item_index = catalog.len() - 1;
                    } else {
                        item_index -= 1;
                    }
                    item = catalog.item(item_index)?;
                    timeline_index = 0;
                    frame_index = item.source_frame(timeline_index)?;
                    status = status_text(paused);
                }
                KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('p') | KeyCode::Char('n') => {
                    item_index = (item_index + 1) % catalog.len();
                    item = catalog.item(item_index)?;
                    timeline_index = 0;
                    frame_index = item.source_frame(timeline_index)?;
                    status = status_text(paused);
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    let variation_count = item.poses.len();
                    frame_index = (frame_index + variation_count - 1) % variation_count;
                    timeline_index = item.timeline_index_for_source_frame(frame_index)?;
                    paused = true;
                    status = status_text(paused);
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    let variation_count = item.poses.len();
                    frame_index = (frame_index + 1) % variation_count;
                    timeline_index = item.timeline_index_for_source_frame(frame_index)?;
                    paused = true;
                    status = status_text(paused);
                }
                KeyCode::Char(' ') => {
                    paused = !paused;
                    status = status_text(paused);
                }
                KeyCode::Char('r') => match load_maps_source(source) {
                    Ok(refreshed) => {
                        artwork = refreshed;
                        timeline_index = 0;
                        item = catalog.item(item_index)?;
                        frame_index = item.source_frame(timeline_index)?;
                        status = status_text(paused);
                    }
                    Err(error) => {
                        status = format!("reload failed; showing last valid maps: {error}");
                    }
                },
                _ => continue,
            }
            next_frame = Instant::now() + item.frame_duration();
            draw(
                &mut output,
                &artwork,
                catalog,
                item_index,
                frame_index,
                &status,
                false,
            )?;
        }

        let now = Instant::now();
        if !paused && now >= next_frame {
            timeline_index = (timeline_index + 1) % item.timeline.len();
            frame_index = item.source_frame(timeline_index)?;
            status = status_text(paused);
            next_frame = now + item.frame_duration();
            draw(
                &mut output,
                &artwork,
                catalog,
                item_index,
                frame_index,
                &status,
                false,
            )?;
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
    fn runtime_preview_uses_codex_pet_dimensions() {
        assert_eq!(pet_image_size(), (9, 5));
        assert_eq!(PET_TARGET_HEIGHT_PX, 75);
    }

    #[test]
    fn runtime_preview_png_is_the_exact_production_frame() {
        let artwork = artwork();
        let png = encode_pose_png(&artwork, "neutral").unwrap();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        let decoded = image::load_from_memory(&png).unwrap().to_rgba8();
        let expected = render_pose("neutral", artwork.pose("neutral").unwrap()).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn runtime_preview_follows_the_manifest_timeline() {
        let idle = ViewerCatalog::RuntimeStates.item(0).unwrap();
        let idle_frames = (0..10)
            .map(|index| idle.source_frame(index).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(idle_frames, vec![0, 0, 0, 1, 1, 0, 0, 2, 2, 0]);
        let jumping = ViewerCatalog::RuntimeStates.item(4).unwrap();
        assert_eq!(jumping.source_frame(0).unwrap(), 0);
        assert_eq!(jumping.source_frame(10).unwrap(), 7);
        assert_eq!(idle.timeline_index_for_source_frame(7).unwrap(), 27);
    }

    #[test]
    fn live_catalog_contains_five_actual_cat_candidates() {
        assert_eq!(ViewerCatalog::Audition.len(), 5);
        let labels = (0..ViewerCatalog::Audition.len())
            .map(|index| ViewerCatalog::Audition.item(index).unwrap().label)
            .collect::<Vec<_>>();
        assert_eq!(
            labels,
            vec!["Authority", "Compact", "Tall Ear", "Forward", "High Tail"]
        );
        for index in 0..ViewerCatalog::Audition.len() {
            let item = ViewerCatalog::Audition.item(index).unwrap();
            assert_eq!(item.poses.len(), 8);
            assert_eq!(item.fps, 8);
            for pose_name in item.poses {
                assert!(artwork().pose(pose_name).is_ok());
            }
        }
    }

    #[test]
    fn kitty_payload_matches_codex_png_transport() {
        assert_eq!(encode_base64(b""), "");
        assert_eq!(encode_base64(b"f"), "Zg==");
        assert_eq!(encode_base64(b"fo"), "Zm8=");
        assert_eq!(encode_base64(b"foo"), "Zm9v");

        let command = kitty_transmit_png(b"png", 9, 5);
        assert!(command.starts_with("\x1b_Ga=T,t=d,f=100,c=9,r=5,q=2"));
        assert!(command.contains(";cG5n"));
        assert!(command.ends_with("\x1b\\"));
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

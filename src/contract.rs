pub const PET_ID: &str = "white-cat";
pub const PET_SELECTOR: &str = "custom:white-cat";
pub const MANIFEST_FILE: &str = "pet.json";
pub const SHEET_FILE: &str = "spritesheet.webp";

pub const FRAME_WIDTH: u32 = 192;
pub const FRAME_HEIGHT: u32 = 208;
pub const GRID_COLUMNS: u32 = 8;
pub const GRID_ROWS: u32 = 9;
pub const FRAME_COUNT: usize = (GRID_COLUMNS * GRID_ROWS) as usize;
pub const FRAMES_PER_STATE: usize = GRID_COLUMNS as usize;
pub const SHEET_WIDTH: u32 = FRAME_WIDTH * GRID_COLUMNS;
pub const SHEET_HEIGHT: u32 = FRAME_HEIGHT * GRID_ROWS;

pub const GROUND_Y: u32 = 200;
pub const LAST_PLANTED_Y: u32 = GROUND_Y - 1;

pub const TERMINAL_CELL_WIDTH: u32 = 8;
pub const TERMINAL_CELL_HEIGHT: u32 = 16;
pub const EXACT_REVIEW_COLUMNS: u32 = 70;
pub const EXACT_REVIEW_ROWS: u32 = 15;
pub const EXACT_REVIEW_WIDTH: u32 = EXACT_REVIEW_COLUMNS * TERMINAL_CELL_WIDTH;
pub const EXACT_REVIEW_HEIGHT: u32 = EXACT_REVIEW_ROWS * TERMINAL_CELL_HEIGHT;

#[derive(Clone, Copy, Debug)]
pub struct StateContract {
    pub name: &'static str,
    pub start: usize,
    pub end: usize,
    pub loops: bool,
}

pub const STATES: [StateContract; 9] = [
    StateContract {
        name: "idle",
        start: 0,
        end: 7,
        loops: true,
    },
    StateContract {
        name: "running-right",
        start: 8,
        end: 15,
        loops: true,
    },
    StateContract {
        name: "running-left",
        start: 16,
        end: 23,
        loops: true,
    },
    StateContract {
        name: "waving",
        start: 24,
        end: 31,
        loops: false,
    },
    StateContract {
        name: "jumping",
        start: 32,
        end: 39,
        loops: false,
    },
    StateContract {
        name: "failed",
        start: 40,
        end: 47,
        loops: false,
    },
    StateContract {
        name: "waiting",
        start: 48,
        end: 55,
        loops: true,
    },
    StateContract {
        name: "running",
        start: 56,
        end: 63,
        loops: true,
    },
    StateContract {
        name: "review",
        start: 64,
        end: 71,
        loops: true,
    },
];

pub const ALIASES: [(&str, &str); 5] = [
    ("move_right", "running-right"),
    ("move_left", "running-left"),
    ("wave", "waving"),
    ("bounce", "jumping"),
    ("sad", "failed"),
];

const IDLE_TIMELINE: &[usize] = &[
    0, 0, 0, 1, 1, 0, 0, 2, 2, 0, 0, 3, 3, 0, 0, 4, 4, 0, 0, 5, 5, 0, 0, 6, 6, 0, 0, 7, 7, 0,
];
const RUNNING_RIGHT_TIMELINE: &[usize] = &[8, 9, 10, 11, 12, 13, 14, 15];
const RUNNING_LEFT_TIMELINE: &[usize] = &[16, 17, 18, 19, 20, 21, 22, 23];
const WAVING_TIMELINE: &[usize] = &[24, 24, 25, 26, 27, 27, 28, 28, 29, 30, 31, 31];
const JUMPING_TIMELINE: &[usize] = &[32, 32, 33, 34, 35, 35, 36, 37, 38, 38, 39];
const FAILED_TIMELINE: &[usize] = &[40, 41, 42, 43, 44, 44, 44, 45, 46, 47, 47];
const WAITING_TIMELINE: &[usize] = &[48, 48, 49, 50, 50, 51, 51, 52, 53, 53, 54, 55, 55];
const RUNNING_TIMELINE: &[usize] = &[56, 56, 57, 58, 58, 59, 59, 60, 61, 61, 62, 63, 63];
const REVIEW_TIMELINE: &[usize] = &[64, 64, 65, 66, 66, 67, 67, 68, 68, 69, 70, 71, 71];

pub fn state_named(name: &str) -> Option<StateContract> {
    STATES.iter().copied().find(|state| state.name == name)
}

pub fn resolve_state(name: &str) -> Option<StateContract> {
    state_named(name).or_else(|| {
        ALIASES
            .iter()
            .find_map(|(alias, target)| (*alias == name).then(|| state_named(target)).flatten())
    })
}

pub fn animation_timeline(name: &str) -> Option<&'static [usize]> {
    match resolve_state(name)?.name {
        "idle" => Some(IDLE_TIMELINE),
        "running-right" => Some(RUNNING_RIGHT_TIMELINE),
        "running-left" => Some(RUNNING_LEFT_TIMELINE),
        "waving" => Some(WAVING_TIMELINE),
        "jumping" => Some(JUMPING_TIMELINE),
        "failed" => Some(FAILED_TIMELINE),
        "waiting" => Some(WAITING_TIMELINE),
        "running" => Some(RUNNING_TIMELINE),
        "review" => Some(REVIEW_TIMELINE),
        _ => None,
    }
}

pub fn animation_fps(name: &str) -> Option<u32> {
    match resolve_state(name)?.name {
        "running-right" | "running-left" | "jumping" => Some(12),
        "waving" | "running" => Some(10),
        "idle" | "failed" | "waiting" | "review" => Some(8),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_allocations_cover_the_fixed_sheet_once() {
        assert_eq!(STATES.len(), GRID_ROWS as usize);
        for (row, state) in STATES.iter().enumerate() {
            assert_eq!(state.start, row * FRAMES_PER_STATE);
            assert_eq!(state.end, state.start + FRAMES_PER_STATE - 1);
            let timeline = animation_timeline(state.name).expect("declared timeline");
            assert!(!timeline.is_empty());
            assert!(
                timeline
                    .iter()
                    .all(|frame| (state.start..=state.end).contains(frame))
            );
            assert!(animation_fps(state.name).expect("declared fps") > 1);
        }
    }

    #[test]
    fn aliases_share_their_target_timing() {
        for (alias, target) in ALIASES {
            assert_eq!(animation_timeline(alias), animation_timeline(target));
            assert_eq!(animation_fps(alias), animation_fps(target));
            assert_eq!(resolve_state(alias).map(|state| state.name), Some(target));
        }
    }
}

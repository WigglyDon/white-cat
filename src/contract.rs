pub const PET_ID: &str = "white-cat";
pub const PET_SELECTOR: &str = "custom:white-cat";
pub const MANIFEST_FILE: &str = "pet.json";
pub const SHEET_FILE: &str = "spritesheet.webp";

pub const FRAME_WIDTH: u32 = 192;
pub const FRAME_HEIGHT: u32 = 208;
pub const GRID_COLUMNS: u32 = 8;
pub const GRID_ROWS: u32 = 9;
pub const FRAME_COUNT: usize = (GRID_COLUMNS * GRID_ROWS) as usize;
pub const SHEET_WIDTH: u32 = FRAME_WIDTH * GRID_COLUMNS;
pub const SHEET_HEIGHT: u32 = FRAME_HEIGHT * GRID_ROWS;

pub const GROUND_Y: u32 = 192;
pub const LAST_PLANTED_Y: u32 = GROUND_Y - 1;
pub const FRAME_MARGIN: u32 = 4;

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

pub fn state_named(name: &str) -> Option<StateContract> {
    STATES.iter().copied().find(|state| state.name == name)
}

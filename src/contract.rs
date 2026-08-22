pub const SOURCE_WIDTH: usize = 24;
pub const SOURCE_HEIGHT: usize = 26;
pub const PIXEL_SCALE: usize = 8;

pub const FRAME_WIDTH: usize = 192;
pub const FRAME_HEIGHT: usize = 208;
pub const COLUMNS: usize = 8;
pub const ROWS: usize = 9;
pub const FRAME_COUNT: usize = COLUMNS * ROWS;
pub const SHEET_WIDTH: usize = FRAME_WIDTH * COLUMNS;
pub const SHEET_HEIGHT: usize = FRAME_HEIGHT * ROWS;
pub const GROUND_Y: usize = 192;
pub const GROUND_PIXEL_Y: usize = GROUND_Y - 1;
pub const TRANSPARENT_MARGIN: usize = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AnimationRange {
    pub name: &'static str,
    pub start: usize,
    pub end: usize,
}

pub const ANIMATION_RANGES: [AnimationRange; ROWS] = [
    AnimationRange {
        name: "idle",
        start: 0,
        end: 7,
    },
    AnimationRange {
        name: "running-right",
        start: 8,
        end: 15,
    },
    AnimationRange {
        name: "running-left",
        start: 16,
        end: 23,
    },
    AnimationRange {
        name: "waving",
        start: 24,
        end: 31,
    },
    AnimationRange {
        name: "jumping",
        start: 32,
        end: 39,
    },
    AnimationRange {
        name: "failed",
        start: 40,
        end: 47,
    },
    AnimationRange {
        name: "waiting",
        start: 48,
        end: 55,
    },
    AnimationRange {
        name: "running",
        start: 56,
        end: 63,
    },
    AnimationRange {
        name: "review",
        start: 64,
        end: 71,
    },
];

pub const ALIASES: [(&str, &str); 5] = [
    ("sad", "failed"),
    ("move_right", "running-right"),
    ("move_left", "running-left"),
    ("wave", "waving"),
    ("bounce", "jumping"),
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

pub const PRIMARY_STATES: [&str; ROWS] = [
    "idle",
    "running-right",
    "running-left",
    "waving",
    "jumping",
    "failed",
    "waiting",
    "running",
    "review",
];

pub const GROUNDED_ANIMATIONS: [&str; 8] = [
    "idle",
    "running-right",
    "running-left",
    "waving",
    "failed",
    "waiting",
    "running",
    "review",
];

pub const JUMP_GROUNDED_FRAMES: [usize; 4] = [32, 33, 38, 39];

pub fn resolve_state(name: &str) -> Option<&'static str> {
    if let Some(state) = PRIMARY_STATES.iter().copied().find(|state| *state == name) {
        return Some(state);
    }
    ALIASES
        .iter()
        .find_map(|(alias, target)| (*alias == name).then_some(*target))
}

pub fn animation_range(name: &str) -> Option<AnimationRange> {
    let resolved = resolve_state(name)?;
    ANIMATION_RANGES
        .iter()
        .copied()
        .find(|range| range.name == resolved)
}

pub fn animation_timeline(name: &str) -> Option<&'static [usize]> {
    match resolve_state(name)? {
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

pub fn animation_fps(name: &str) -> Option<u64> {
    match resolve_state(name)? {
        "running-right" | "running-left" | "jumping" => Some(12),
        "waving" | "running" => Some(10),
        "idle" | "failed" | "waiting" | "review" => Some(8),
        _ => None,
    }
}

pub fn animation_loops(name: &str) -> bool {
    matches!(
        name,
        "idle"
            | "running"
            | "waiting"
            | "review"
            | "running-right"
            | "running-left"
            | "move_right"
            | "move_left"
    )
}

pub fn frame_position(index: usize) -> Option<(usize, usize)> {
    (index < FRAME_COUNT).then_some((index / COLUMNS, index % COLUMNS))
}

pub fn state_for_index(index: usize) -> Option<&'static str> {
    ANIMATION_RANGES
        .iter()
        .find_map(|range| (range.start <= index && index <= range.end).then_some(range.name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_and_index_mapping_are_fixed() {
        assert_eq!((FRAME_WIDTH, FRAME_HEIGHT), (192, 208));
        assert_eq!((COLUMNS, ROWS, FRAME_COUNT), (8, 9, 72));
        assert_eq!((SHEET_WIDTH, SHEET_HEIGHT), (1536, 1872));
        assert_eq!((GROUND_Y, GROUND_PIXEL_Y), (192, 191));
        assert_eq!(frame_position(0), Some((0, 0)));
        assert_eq!(frame_position(71), Some((8, 7)));
        assert_eq!(frame_position(72), None);
    }

    #[test]
    fn aliases_share_state_contracts() {
        assert_eq!(resolve_state("move_right"), Some("running-right"));
        assert_eq!(animation_timeline("wave"), animation_timeline("waving"));
        assert_eq!(animation_fps("bounce"), Some(12));
    }
}

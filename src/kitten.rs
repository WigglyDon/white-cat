//! Frozen canonical artwork and animation poses for the production White Cat pet.
//!
//! Every production pose is a literal 24 x 26 map. Every production runtime
//! pixel is a direct 8 x 8 expansion of its selected map cell.

use image::{Rgba, RgbaImage};

use crate::contract::{FRAME_HEIGHT, FRAME_WIDTH};

pub const CONCEPT_REFERENCE_FILE: &str = "concept_design_of_pixel_art_cat.png";
pub const CONCEPT_REFERENCE_SHA256: &str =
    "974bae7813b6b80a0626ca5b3d292244f5abf937f97a3f6c3102fb70180ea322";

pub const LOGICAL_WIDTH: usize = 24;
pub const LOGICAL_HEIGHT: usize = 26;
pub const RUNTIME_PIXEL_SIZE: u32 = 8;

pub const NORMALIZED_MATRIX_BYTES: usize = 650;
pub const LOGICAL_RGBA_BYTES: usize = LOGICAL_WIDTH * LOGICAL_HEIGHT * 4;
pub const NORMALIZED_MATRIX_SHA256: &str =
    "9fff8b4d54bdae285fa048ce872857e93a55ba1e034622cab5435b672e9d6735";
pub const CONTIGUOUS_SYMBOL_SHA256: &str =
    "d67509d496e5b7f1c3e61af931a4de8ae43131e0ac84230247b228ea256bd330";
pub const LOGICAL_RGBA_SHA256: &str =
    "f031e0980557f6c9cfbe6855ff18afd8b41281a93f67e473629302efe558bbac";
pub const RUNTIME_RGBA_SHA256: &str =
    "cfb50024b6b044d37a9d55f0cf995f5688a4722907cc35b7cc30a350cab7992d";
pub const FROZEN_ANIMATION_CONTRACT_SHA256: &str =
    "1ef0ede95c41c0cc43f5500ca8d600a6e663d634d971b5cd63bdac3424ec2304";

pub const TRANSPARENT: [u8; 4] = [0, 0, 0, 0];
pub const OUTLINE: [u8; 4] = [42, 51, 64, 255];
pub const BODY: [u8; 4] = [244, 242, 232, 255];
pub const SHADE: [u8; 4] = [205, 210, 216, 255];
pub const EYE: [u8; 4] = [134, 215, 168, 255];
pub const SILHOUETTE: [u8; 4] = OUTLINE;

pub const PALETTE: [(char, [u8; 4]); 5] = [
    ('.', TRANSPARENT),
    ('O', OUTLINE),
    ('B', BODY),
    ('S', SHADE),
    ('E', EYE),
];

pub const EXPECTED_SYMBOL_COUNTS: [(char, usize); 5] =
    [('.', 303), ('O', 70), ('B', 224), ('S', 23), ('E', 4)];

pub type PixelMap = [&'static str; LOGICAL_HEIGHT];

/// Exact master-supplied canonical base pose. Do not edit coordinates without
/// a replacement frozen matrix contract from the artwork authority.
pub const CANONICAL_MAP: PixelMap = [
    "........................",
    "..............OOO...OO..",
    ".............OOBO..OBO..",
    ".............OBBBO.SBO..",
    ".............OBBBOOBBO..",
    "............OBBBBBBBBOO.",
    "............OBBBBBBBBBOO",
    "...........OBBBBBBBBBBBS",
    "...........OBBBBBBBEEBBB",
    "...........OBBBBBBBEEBBB",
    "...........OBBBBBBBBBBBB",
    "..........OBBBBBBBBBBBBB",
    "..........OBBBBBBBBBBBBO",
    ".........OBBBBBBBBBBBBO.",
    ".......OOSBBBBBBBBBOO...",
    ".......SBBBBBBBBBBSO....",
    "......OBBBBBBBBBBBS.....",
    ".....OBBBBBBBBBBBBSO....",
    ".....OBBBBBBBBBBBBBS....",
    ".....OBBBBBBBBSBB.BS....",
    ".OO..OBBBBBBBBSBBOBS....",
    "OBBO.OBBBBBBBSSBBOBS....",
    "OBBBBBBBBBBBSSSBBOSBO...",
    "..SBBSOBBBBBBBOBBSSBBO..",
    ".......OOOOOOOOOOOOOO...",
    "........................",
];

/// Closed-eye idle keyframe. The approved base silhouette remains unchanged.
pub const BLINK_MAP: PixelMap = [
    "........................",
    "..............OOO...OO..",
    ".............OOBO..OBO..",
    ".............OBBBO.SBO..",
    ".............OBBBOOBBO..",
    "............OBBBBBBBBOO.",
    "............OBBBBBBBBBOO",
    "...........OBBBBBBBBBBBS",
    "...........OBBBBBBBOOBBB",
    "...........OBBBBBBBBBBBB",
    "...........OBBBBBBBBBBBB",
    "..........OBBBBBBBBBBBBB",
    "..........OBBBBBBBBBBBBO",
    ".........OBBBBBBBBBBBBO.",
    ".......OOSBBBBBBBBBOO...",
    ".......SBBBBBBBBBBSO....",
    "......OBBBBBBBBBBBS.....",
    ".....OBBBBBBBBBBBBSO....",
    ".....OBBBBBBBBBBBBBS....",
    ".....OBBBBBBBBSBB.BS....",
    ".OO..OBBBBBBBBSBBOBS....",
    "OBBO.OBBBBBBBSSBBOBS....",
    "OBBBBBBBBBBBSSSBBOSBO...",
    "..SBBSOBBBBBBBOBBSSBBO..",
    ".......OOOOOOOOOOOOOO...",
    "........................",
];

/// One-cell right-ear twitch with the canonical body held still.
pub const EAR_TWITCH_MAP: PixelMap = [
    "........................",
    "..............OOO....OO.",
    ".............OOBO..OBO..",
    ".............OBBBO.SBO..",
    ".............OBBBOOBBO..",
    "............OBBBBBBBBOO.",
    "............OBBBBBBBBBOO",
    "...........OBBBBBBBBBBBS",
    "...........OBBBBBBBEEBBB",
    "...........OBBBBBBBEEBBB",
    "...........OBBBBBBBBBBBB",
    "..........OBBBBBBBBBBBBB",
    "..........OBBBBBBBBBBBBO",
    ".........OBBBBBBBBBBBBO.",
    ".......OOSBBBBBBBBBOO...",
    ".......SBBBBBBBBBBSO....",
    "......OBBBBBBBBBBBS.....",
    ".....OBBBBBBBBBBBBSO....",
    ".....OBBBBBBBBBBBBBS....",
    ".....OBBBBBBBBSBB.BS....",
    ".OO..OBBBBBBBBSBBOBS....",
    "OBBO.OBBBBBBBSSBBOBS....",
    "OBBBBBBBBBBBSSSBBOSBO...",
    "..SBBSOBBBBBBBOBBSSBBO..",
    ".......OOOOOOOOOOOOOO...",
    "........................",
];

/// Lifted-tail idle keyframe.
pub const TAIL_LIFT_MAP: PixelMap = [
    "........................",
    "..............OOO...OO..",
    ".............OOBO..OBO..",
    ".............OBBBO.SBO..",
    ".............OBBBOOBBO..",
    "............OBBBBBBBBOO.",
    "............OBBBBBBBBBOO",
    "...........OBBBBBBBBBBBS",
    "...........OBBBBBBBEEBBB",
    "...........OBBBBBBBEEBBB",
    "...........OBBBBBBBBBBBB",
    "..........OBBBBBBBBBBBBB",
    "..........OBBBBBBBBBBBBO",
    ".........OBBBBBBBBBBBBO.",
    ".......OOSBBBBBBBBBOO...",
    ".......SBBBBBBBBBBSO....",
    "......OBBBBBBBBBBBS.....",
    "..OO.OBBBBBBBBBBBBSO....",
    ".OBBOOBBBBBBBBBBBBBS....",
    "OBBBOOBBBBBBBBSBB.BS....",
    ".OBBOOBBBBBBBBSBBOBS....",
    "OBBO.OBBBBBBBSSBBOBS....",
    "OBBBBBBBBBBBSSSBBOSBO...",
    "..SBBSOBBBBBBBOBBSSBBO..",
    ".......OOOOOOOOOOOOOO...",
    "........................",
];

/// Waiting keyframe combines the alert ear and lifted tail.
pub const WAITING_MAP: PixelMap = [
    "........................",
    "..............OOO....OO.",
    ".............OOBO..OBO..",
    ".............OBBBO.SBO..",
    ".............OBBBOOBBO..",
    "............OBBBBBBBBOO.",
    "............OBBBBBBBBBOO",
    "...........OBBBBBBBBBBBS",
    "...........OBBBBBBBEEBBB",
    "...........OBBBBBBBEEBBB",
    "...........OBBBBBBBBBBBB",
    "..........OBBBBBBBBBBBBB",
    "..........OBBBBBBBBBBBBO",
    ".........OBBBBBBBBBBBBO.",
    ".......OOSBBBBBBBBBOO...",
    ".......SBBBBBBBBBBSO....",
    "......OBBBBBBBBBBBS.....",
    "..OO.OBBBBBBBBBBBBSO....",
    ".OBBOOBBBBBBBBBBBBBS....",
    "OBBBOOBBBBBBBBSBB.BS....",
    ".OBBOOBBBBBBBBSBBOBS....",
    "OBBO.OBBBBBBBSSBBOBS....",
    "OBBBBBBBBBBBSSSBBOSBO...",
    "..SBBSOBBBBBBBOBBSSBBO..",
    ".......OOOOOOOOOOOOOO...",
    "........................",
];

pub const WAVE_LOW_MAP: PixelMap = [
    "........................",
    "..............OOO...OO..",
    ".............OOBO..OBO..",
    ".............OBBBO.SBO..",
    ".............OBBBOOBBO..",
    "............OBBBBBBBBOO.",
    "............OBBBBBBBBBOO",
    "...........OBBBBBBBBBBBS",
    "...........OBBBBBBBEEBBB",
    "...........OBBBBBBBEEBBB",
    "...........OBBBBBBBBBBBB",
    "..........OBBBBBBBBBBBBB",
    "..........OBBBBBBBBBBBBO",
    ".........OBBBBBBBBBBBBO.",
    ".......OOSBBBBBBBBBOO...",
    ".......SBBBBBBBBBBSO....",
    "......OBBBBBBBBBBBSOBBO.",
    ".....OBBBBBBBBBBBBSBBO..",
    ".....OBBBBBBBBBBBBBS....",
    ".....OBBBBBBBBSBB.BS....",
    ".OO..OBBBBBBBBSBBOBS....",
    "OBBO.OBBBBBBBSSBBOBS....",
    "OBBBBBBBBBBBSSSBBOSBO...",
    "..SBBSOBBBBBBBOBBSSBBO..",
    ".......OOOOOOOOOOOOOO...",
    "........................",
];

pub const WAVE_HIGH_MAP: PixelMap = [
    "........................",
    "..............OOO...OO..",
    ".............OOBO..OBO..",
    ".............OBBBO.SBO..",
    ".............OBBBOOBBO..",
    "............OBBBBBBBBOO.",
    "............OBBBBBBBBBOO",
    "...........OBBBBBBBBBBBS",
    "...........OBBBBBBBEEBBB",
    "...........OBBBBBBBEEBBB",
    "...........OBBBBBBBBBBBB",
    "..........OBBBBBBBBBBBBB",
    "..........OBBBBBBBBBBBBO",
    ".........OBBBBBBBBBBBBO.",
    ".......OOSBBBBBBBBBOOBO.",
    ".......SBBBBBBBBBBSOBBO.",
    "......OBBBBBBBBBBBSOBO..",
    ".....OBBBBBBBBBBBBSO....",
    ".....OBBBBBBBBBBBBBS....",
    ".....OBBBBBBBBSBB.BS....",
    ".OO..OBBBBBBBBSBBOBS....",
    "OBBO.OBBBBBBBSSBBOBS....",
    "OBBBBBBBBBBBSSSBBOSBO...",
    "..SBBSOBBBBBBBOBBSSBBO..",
    ".......OOOOOOOOOOOOOO...",
    "........................",
];

pub const RUN_RIGHT_A_MAP: PixelMap = [
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    ".................OO.O...",
    "................OBO.OBO.",
    "...............OBBBBBBO.",
    "..............OBBBBEEBBO",
    "...OO......OOBBBBBBBBBO.",
    "..OBBO...OOBBBBBBBBBBO..",
    ".OBBBO.OOBBBBBBBBBBO....",
    ".OBBBBOBBBBBBBBBBBBBO...",
    "..OBBBBBBBBBBBBBBBBBO...",
    "...OBBBBBSSBBBBBBBBO....",
    "....OBBBBBSBBBBBO.OBO...",
    ".....OBBBO...OBBBO......",
    "....OBBBO......OBBBO....",
    "....OBBBO......OBBBO....",
    "....OOOOO......OOOOO....",
    "........................",
];

pub const RUN_RIGHT_B_MAP: PixelMap = [
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    ".................OO.O...",
    "................OBO.OBO.",
    "...............OBBBBBBO.",
    "..............OBBBBEEBBO",
    ".OOO.......OOBBBBBBBBBO.",
    ".OBBO....OOBBBBBBBBBBO..",
    ".OBBBO.OOBBBBBBBBBBO....",
    "..OBBBOOBBBBBBBBBBBBO...",
    "...OBBBBBBBBBBBBBBBBO...",
    "....OBBBBSSBBBBBBBBO....",
    ".....OBBBBO.OBBBBO......",
    ".......OBBBOOBBBO.......",
    "......OBBBO...OBBBO.....",
    "......OBBBO...OBBBO.....",
    "......OOOOO...OOOOO.....",
    "........................",
];

pub const RUN_LEFT_A_MAP: PixelMap = [
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "...O.OO.................",
    ".OBO.OBO................",
    ".OBBBBBBO...............",
    "OBBEEBBBBO..............",
    ".OBBBBBBBBBOO......OO...",
    "..OBBBBBBBBBBOO...OBBO..",
    "....OBBBBBBBBBBOO.OBBBO.",
    "...OBBBBBBBBBBBBBOBBBBO.",
    "...OBBBBBBBBBBBBBBBBBO..",
    "....OBBBBBBBBSSBBBBBO...",
    "...OBO.OBBBBBSBBBBBO....",
    "......OBBBO...OBBBO.....",
    "....OBBBO......OBBBO....",
    "....OBBBO......OBBBO....",
    "....OOOOO......OOOOO....",
    "........................",
];

pub const RUN_LEFT_B_MAP: PixelMap = [
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "...O.OO.................",
    ".OBO.OBO................",
    ".OBBBBBBO...............",
    "OBBEEBBBBO..............",
    ".OBBBBBBBBBOO.......OOO.",
    "..OBBBBBBBBBBOO....OBBO.",
    "....OBBBBBBBBBBOO.OBBBO.",
    "...OBBBBBBBBBBBBOOBBBO..",
    "...OBBBBBBBBBBBBBBBBO...",
    "....OBBBBBBBBSSBBBBO....",
    "......OBBBBO.OBBBBO.....",
    ".......OBBBOOBBBO.......",
    ".....OBBBO...OBBBO......",
    ".....OBBBO...OBBBO......",
    ".....OOOOO...OOOOO......",
    "........................",
];

pub const JUMP_RISE_MAP: PixelMap = [
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    ".................OO.O...",
    "................OBO.OBO.",
    "...............OBBBBBBO.",
    "..............OBBBBEEBBO",
    "...OO......OOBBBBBBBBBO.",
    "..OBBO...OOBBBBBBBBBBO..",
    ".OBBBO.OOBBBBBBBBBBO....",
    ".OBBBBOBBBBBBBBBBBBBO...",
    "..OBBBBBBBBBBBBBBBBBO...",
    "...OBBBBBSSBBBBBBBBO....",
    "....OBBBBBSBBBBBO.OBO...",
    ".....OBBBO...OBBBO......",
    "....OBBBO......OBBBO....",
    "....OBBBO......OBBBO....",
    "....OOOOO......OOOOO....",
    "........................",
    "........................",
    "........................",
];

pub const JUMP_APEX_MAP: PixelMap = [
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    ".................OO.O...",
    "................OBO.OBO.",
    "...............OBBBBBBO.",
    "..............OBBBBEEBBO",
    "...OO......OOBBBBBBBBBO.",
    "..OBBO...OOBBBBBBBBBBO..",
    ".OBBBO.OOBBBBBBBBBBO....",
    ".OBBBBOBBBBBBBBBBBBBO...",
    "..OBBBBBBBBBBBBBBBBBO...",
    "...OBBBBBSSBBBBBBBBO....",
    "....OBBBBBSBBBBBO.OBO...",
    ".....OBBBO...OBBBO......",
    "....OBBBO......OBBBO....",
    "....OBBBO......OBBBO....",
    "....OOOOO......OOOOO....",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
];

pub const FAILED_MAP: PixelMap = [
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    ".................OO.O...",
    "................OBO.OBO.",
    "...............OBBBBBBO.",
    "..OO.........OOBBBBEEBBO",
    ".OBBO.....OOBBBBBBBBBBO.",
    ".OBBBO.OOBBBBBBBBBBBO...",
    ".OBBBBBBBBBBBSSBBBBBBO..",
    "..OSBBBBBBBBBBBBBBBO....",
    "...OOOOOOOOOOOOOOOOO....",
    "........................",
];

pub const FAILED_BLINK_MAP: PixelMap = [
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
    ".................OO.O...",
    "................OBO.OBO.",
    "...............OBBBBBBO.",
    "..OO.........OOBBBBOOBBO",
    ".OBBO.....OOBBBBBBBBBBO.",
    ".OBBBO.OOBBBBBBBBBBBO...",
    ".OBBBBBBBBBBBSSBBBBBBO..",
    "..OSBBBBBBBBBBBBBBBO....",
    "...OOOOOOOOOOOOOOOOO....",
    "........................",
];

pub const WORK_UP_MAP: PixelMap = [
    "........................",
    "..............OOO...OO..",
    ".............OOBO..OBO..",
    ".............OBBBO.SBO..",
    ".............OBBBOOBBO..",
    "............OBBBBBBBBOO.",
    "............OBBBBBBBBBOO",
    "...........OBBBBBBBBBBBS",
    "...........OBBBBBBBEEBBB",
    "...........OBBBBBBBEEBBB",
    "...........OBBBBBBBBBBBB",
    "..........OBBBBBBBBBBBBB",
    "..........OBBBBBBBBBBBBO",
    ".........OBBBBBBBBBBBBO.",
    ".......OOSBBBBBBBBBOO...",
    ".......SBBBBBBBBBBSO....",
    "......OBBBBBBBBBBBS.....",
    ".....OBBBBBBBBBBBBSO....",
    ".....OBBBBBBBBBBBBBSOBO.",
    ".....OBBBBBBBBSBB.BSOBBO",
    ".OO..OBBBBBBBBSBBOBS....",
    "OBBO.OBBBBBBBSSBBOBS....",
    "OBBBBBBBBBBBSSSBBOSBO...",
    "..SBBSOBBBBBBBOBBSSBBO..",
    ".......OOOOOOOOOOOOOO...",
    "........................",
];

pub const WORK_DOWN_MAP: PixelMap = [
    "........................",
    "..............OOO...OO..",
    ".............OOBO..OBO..",
    ".............OBBBO.SBO..",
    ".............OBBBOOBBO..",
    "............OBBBBBBBBOO.",
    "............OBBBBBBBBBOO",
    "...........OBBBBBBBBBBBS",
    "...........OBBBBBBBEEBBB",
    "...........OBBBBBBBEEBBB",
    "...........OBBBBBBBBBBBB",
    "..........OBBBBBBBBBBBBB",
    "..........OBBBBBBBBBBBBO",
    ".........OBBBBBBBBBBBBO.",
    ".......OOSBBBBBBBBBOO...",
    ".......SBBBBBBBBBBSO....",
    "......OBBBBBBBBBBBS.....",
    ".....OBBBBBBBBBBBBSO....",
    ".....OBBBBBBBBBBBBBS....",
    ".....OBBBBBBBBSBB.BSOBBO",
    ".OO..OBBBBBBBBSBBOBSOBO.",
    "OBBO.OBBBBBBBSSBBOBS....",
    "OBBBBBBBBBBBSSSBBOSBO...",
    "..SBBSOBBBBBBBOBBSSBBO..",
    ".......OOOOOOOOOOOOOO...",
    "........................",
];

pub const REVIEW_GLANCE_MAP: PixelMap = [
    "........................",
    "..............OOO...OO..",
    ".............OOBO..OBO..",
    ".............OBBBO.SBO..",
    ".............OBBBOOBBO..",
    "............OBBBBBBBBOO.",
    "............OBBBBBBBBBOO",
    "...........OBBBBBBBBBBBS",
    "...........OBBBBBBEEBBBB",
    "...........OBBBBBBEEBBBB",
    "...........OBBBBBBBBBBBB",
    "..........OBBBBBBBBBBBBB",
    "..........OBBBBBBBBBBBBO",
    ".........OBBBBBBBBBBBBO.",
    ".......OOSBBBBBBBBBOO...",
    ".......SBBBBBBBBBBSO....",
    "......OBBBBBBBBBBBS.....",
    ".....OBBBBBBBBBBBBSO....",
    ".....OBBBBBBBBBBBBBS....",
    ".....OBBBBBBBBSBB.BS....",
    ".OO..OBBBBBBBBSBBOBS....",
    "OBBO.OBBBBBBBSSBBOBS....",
    "OBBBBBBBBBBBSSSBBOSBO...",
    "..SBBSOBBBBBBBOBBSSBBO..",
    ".......OOOOOOOOOOOOOO...",
    "........................",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PoseId {
    Canonical,
    Blink,
    EarTwitch,
    TailLift,
    Waiting,
    WaveLow,
    WaveHigh,
    RunRightA,
    RunRightB,
    RunLeftA,
    RunLeftB,
    JumpRise,
    JumpApex,
    Failed,
    FailedBlink,
    WorkUp,
    WorkDown,
    ReviewGlance,
}

pub const ALL_POSES: [PoseId; 18] = [
    PoseId::Canonical,
    PoseId::Blink,
    PoseId::EarTwitch,
    PoseId::TailLift,
    PoseId::Waiting,
    PoseId::WaveLow,
    PoseId::WaveHigh,
    PoseId::RunRightA,
    PoseId::RunRightB,
    PoseId::RunLeftA,
    PoseId::RunLeftB,
    PoseId::JumpRise,
    PoseId::JumpApex,
    PoseId::Failed,
    PoseId::FailedBlink,
    PoseId::WorkUp,
    PoseId::WorkDown,
    PoseId::ReviewGlance,
];

/// Eight literal sheet-cell poses for each state row in `contract::STATES` order.
pub const FRAME_POSES: [[PoseId; 8]; 9] = [
    [
        PoseId::Canonical,
        PoseId::Canonical,
        PoseId::Blink,
        PoseId::Canonical,
        PoseId::EarTwitch,
        PoseId::Canonical,
        PoseId::TailLift,
        PoseId::Canonical,
    ],
    [
        PoseId::RunRightA,
        PoseId::RunRightB,
        PoseId::RunRightA,
        PoseId::RunRightB,
        PoseId::RunRightA,
        PoseId::RunRightB,
        PoseId::RunRightA,
        PoseId::RunRightB,
    ],
    [
        PoseId::RunLeftA,
        PoseId::RunLeftB,
        PoseId::RunLeftA,
        PoseId::RunLeftB,
        PoseId::RunLeftA,
        PoseId::RunLeftB,
        PoseId::RunLeftA,
        PoseId::RunLeftB,
    ],
    [
        PoseId::Canonical,
        PoseId::WaveLow,
        PoseId::WaveHigh,
        PoseId::WaveHigh,
        PoseId::WaveLow,
        PoseId::WaveHigh,
        PoseId::WaveLow,
        PoseId::Canonical,
    ],
    [
        PoseId::RunRightA,
        PoseId::RunRightB,
        PoseId::JumpRise,
        PoseId::JumpApex,
        PoseId::JumpApex,
        PoseId::JumpRise,
        PoseId::RunRightB,
        PoseId::RunRightA,
    ],
    [
        PoseId::Failed,
        PoseId::Failed,
        PoseId::FailedBlink,
        PoseId::Failed,
        PoseId::Failed,
        PoseId::FailedBlink,
        PoseId::Failed,
        PoseId::Failed,
    ],
    [
        PoseId::Canonical,
        PoseId::Waiting,
        PoseId::Waiting,
        PoseId::Blink,
        PoseId::Waiting,
        PoseId::Waiting,
        PoseId::TailLift,
        PoseId::Canonical,
    ],
    [
        PoseId::WorkUp,
        PoseId::WorkUp,
        PoseId::WorkDown,
        PoseId::WorkDown,
        PoseId::WorkUp,
        PoseId::WorkUp,
        PoseId::WorkDown,
        PoseId::WorkUp,
    ],
    [
        PoseId::Canonical,
        PoseId::ReviewGlance,
        PoseId::ReviewGlance,
        PoseId::Blink,
        PoseId::ReviewGlance,
        PoseId::ReviewGlance,
        PoseId::Canonical,
        PoseId::Canonical,
    ],
];

pub fn pose_name(pose: PoseId) -> &'static str {
    match pose {
        PoseId::Canonical => "canonical",
        PoseId::Blink => "blink",
        PoseId::EarTwitch => "ear-twitch",
        PoseId::TailLift => "tail-lift",
        PoseId::Waiting => "waiting",
        PoseId::WaveLow => "wave-low",
        PoseId::WaveHigh => "wave-high",
        PoseId::RunRightA => "run-right-a",
        PoseId::RunRightB => "run-right-b",
        PoseId::RunLeftA => "run-left-a",
        PoseId::RunLeftB => "run-left-b",
        PoseId::JumpRise => "jump-rise",
        PoseId::JumpApex => "jump-apex",
        PoseId::Failed => "failed",
        PoseId::FailedBlink => "failed-blink",
        PoseId::WorkUp => "work-up",
        PoseId::WorkDown => "work-down",
        PoseId::ReviewGlance => "review-glance",
    }
}

pub fn pose_map(pose: PoseId) -> &'static PixelMap {
    match pose {
        PoseId::Canonical => &CANONICAL_MAP,
        PoseId::Blink => &BLINK_MAP,
        PoseId::EarTwitch => &EAR_TWITCH_MAP,
        PoseId::TailLift => &TAIL_LIFT_MAP,
        PoseId::Waiting => &WAITING_MAP,
        PoseId::WaveLow => &WAVE_LOW_MAP,
        PoseId::WaveHigh => &WAVE_HIGH_MAP,
        PoseId::RunRightA => &RUN_RIGHT_A_MAP,
        PoseId::RunRightB => &RUN_RIGHT_B_MAP,
        PoseId::RunLeftA => &RUN_LEFT_A_MAP,
        PoseId::RunLeftB => &RUN_LEFT_B_MAP,
        PoseId::JumpRise => &JUMP_RISE_MAP,
        PoseId::JumpApex => &JUMP_APEX_MAP,
        PoseId::Failed => &FAILED_MAP,
        PoseId::FailedBlink => &FAILED_BLINK_MAP,
        PoseId::WorkUp => &WORK_UP_MAP,
        PoseId::WorkDown => &WORK_DOWN_MAP,
        PoseId::ReviewGlance => &REVIEW_GLANCE_MAP,
    }
}

pub fn palette_color(symbol: char) -> Option<[u8; 4]> {
    PALETTE
        .iter()
        .find_map(|(candidate, color)| (*candidate == symbol).then_some(*color))
}

pub fn canonical_symbol(x: usize, y: usize) -> Option<char> {
    map_symbol(&CANONICAL_MAP, x, y)
}

pub fn map_symbol(map: &PixelMap, x: usize, y: usize) -> Option<char> {
    map.get(y)?.as_bytes().get(x).map(|byte| *byte as char)
}

pub fn normalized_matrix_text() -> String {
    let mut normalized = String::with_capacity(NORMALIZED_MATRIX_BYTES);
    for row in CANONICAL_MAP {
        normalized.push_str(row);
        normalized.push('\n');
    }
    normalized
}

pub fn contiguous_symbols() -> Vec<u8> {
    CANONICAL_MAP.iter().flat_map(|row| row.bytes()).collect()
}

pub fn map_is_well_formed(map: &PixelMap) -> bool {
    FRAME_WIDTH == LOGICAL_WIDTH as u32 * RUNTIME_PIXEL_SIZE
        && FRAME_HEIGHT == LOGICAL_HEIGHT as u32 * RUNTIME_PIXEL_SIZE
        && map.iter().all(|row| {
            row.len() == LOGICAL_WIDTH && row.chars().all(|symbol| palette_color(symbol).is_some())
        })
}

pub fn canonical_map_is_well_formed() -> bool {
    map_is_well_formed(&CANONICAL_MAP)
}

pub fn render_logical_map(map: &PixelMap) -> RgbaImage {
    assert!(
        map_is_well_formed(map),
        "pixel map violates its fixed geometry or palette"
    );
    let mut logical = RgbaImage::new(LOGICAL_WIDTH as u32, LOGICAL_HEIGHT as u32);
    for (y, row) in map.iter().enumerate() {
        for (x, symbol) in row.chars().enumerate() {
            logical.put_pixel(
                x as u32,
                y as u32,
                Rgba(palette_color(symbol).expect("validated map symbol")),
            );
        }
    }
    logical
}

pub fn render_logical() -> RgbaImage {
    render_logical_map(&CANONICAL_MAP)
}

pub fn render_map(map: &PixelMap) -> RgbaImage {
    let logical = render_logical_map(map);
    let mut frame = RgbaImage::new(FRAME_WIDTH, FRAME_HEIGHT);
    for (runtime_x, runtime_y, pixel) in frame.enumerate_pixels_mut() {
        let logical_x = runtime_x / RUNTIME_PIXEL_SIZE;
        let logical_y = runtime_y / RUNTIME_PIXEL_SIZE;
        *pixel = *logical.get_pixel(logical_x, logical_y);
    }
    frame
}

pub fn render_pose(pose: PoseId) -> RgbaImage {
    render_map(pose_map(pose))
}

pub fn render_frame() -> RgbaImage {
    render_pose(PoseId::Canonical)
}

pub fn build_frames() -> Vec<RgbaImage> {
    FRAME_POSES
        .iter()
        .flatten()
        .copied()
        .map(render_pose)
        .collect()
}

pub fn frozen_animation_contract_text() -> String {
    let mut text = String::new();
    for pose in ALL_POSES {
        text.push_str("pose\t");
        text.push_str(pose_name(pose));
        text.push('\n');
        for row in pose_map(pose) {
            text.push_str(row);
            text.push('\n');
        }
    }
    for (state_row, poses) in FRAME_POSES.iter().enumerate() {
        text.push_str(&format!("state-row\t{state_row}"));
        for pose in poses {
            text.push('\t');
            text.push_str(pose_name(*pose));
        }
        text.push('\n');
    }
    text
}

pub fn render_silhouette_logical() -> RgbaImage {
    let logical = render_logical();
    let mut silhouette = RgbaImage::new(LOGICAL_WIDTH as u32, LOGICAL_HEIGHT as u32);
    for (target, original) in silhouette.pixels_mut().zip(logical.pixels()) {
        *target = if original[3] == 0 {
            Rgba(TRANSPARENT)
        } else {
            Rgba(SILHOUETTE)
        };
    }
    silhouette
}

pub fn render_silhouette_frame() -> RgbaImage {
    let frame = render_frame();
    let mut silhouette = RgbaImage::new(FRAME_WIDTH, FRAME_HEIGHT);
    for (target, original) in silhouette.pixels_mut().zip(frame.pixels()) {
        *target = if original[3] == 0 {
            Rgba(TRANSPARENT)
        } else {
            Rgba(SILHOUETTE)
        };
    }
    silhouette
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::digest::sha256_hex;

    #[test]
    fn frozen_matrix_contract_is_exact() {
        assert!(canonical_map_is_well_formed());
        assert_eq!(CANONICAL_MAP.len(), LOGICAL_HEIGHT);
        assert!(CANONICAL_MAP.iter().all(|row| row.len() == LOGICAL_WIDTH));
        assert_eq!(normalized_matrix_text().len(), NORMALIZED_MATRIX_BYTES);
        assert_eq!(contiguous_symbols().len(), LOGICAL_WIDTH * LOGICAL_HEIGHT);
        assert_eq!(
            sha256_hex(normalized_matrix_text().as_bytes()),
            NORMALIZED_MATRIX_SHA256
        );
        assert_eq!(sha256_hex(&contiguous_symbols()), CONTIGUOUS_SYMBOL_SHA256);
        assert_eq!(sha256_hex(render_logical().as_raw()), LOGICAL_RGBA_SHA256);
        assert_eq!(CANONICAL_MAP[25], "........................");
    }

    #[test]
    fn palette_matches_the_frozen_contract() {
        assert_eq!(OUTLINE, [0x2a, 0x33, 0x40, 0xff]);
        assert_eq!(BODY, [0xf4, 0xf2, 0xe8, 0xff]);
        assert_eq!(SHADE, [0xcd, 0xd2, 0xd8, 0xff]);
        assert_eq!(EYE, [0x86, 0xd7, 0xa8, 0xff]);
        assert_eq!(PALETTE.len(), 5);
    }

    #[test]
    fn runtime_is_direct_nearest_neighbor() {
        let frame = render_frame();
        assert_eq!(frame.dimensions(), (FRAME_WIDTH, FRAME_HEIGHT));
        assert_eq!(sha256_hex(frame.as_raw()), RUNTIME_RGBA_SHA256);
        for (x, y, pixel) in frame.enumerate_pixels() {
            let expected = palette_color(
                canonical_symbol(
                    (x / RUNTIME_PIXEL_SIZE) as usize,
                    (y / RUNTIME_PIXEL_SIZE) as usize,
                )
                .expect("complete canonical matrix"),
            )
            .expect("canonical palette symbol");
            assert_eq!(pixel.0, expected, "runtime mismatch at ({x},{y})");
        }
    }

    #[test]
    fn every_animation_pose_is_literal_and_direct() {
        for pose in ALL_POSES {
            let map = pose_map(pose);
            assert!(map_is_well_formed(map), "{}", pose_name(pose));
            assert_eq!(map[25], "........................", "{}", pose_name(pose));
            let frame = render_pose(pose);
            for (x, y, pixel) in frame.enumerate_pixels() {
                let symbol = map_symbol(
                    map,
                    (x / RUNTIME_PIXEL_SIZE) as usize,
                    (y / RUNTIME_PIXEL_SIZE) as usize,
                )
                .expect("complete literal pose");
                assert_eq!(
                    pixel.0,
                    palette_color(symbol).expect("declared palette symbol"),
                    "{} mismatch at ({x},{y})",
                    pose_name(pose)
                );
            }
        }
    }

    #[test]
    fn directional_run_poses_are_exact_mirrors() {
        for (right, left) in [
            (&RUN_RIGHT_A_MAP, &RUN_LEFT_A_MAP),
            (&RUN_RIGHT_B_MAP, &RUN_LEFT_B_MAP),
        ] {
            for (right_row, left_row) in right.iter().zip(left) {
                assert_eq!(right_row.chars().rev().collect::<String>(), *left_row);
            }
        }
    }

    #[test]
    fn frame_plan_populates_all_cells_and_uses_every_pose() {
        let frames: Vec<PoseId> = FRAME_POSES.iter().flatten().copied().collect();
        assert_eq!(frames.len(), 72);
        for pose in ALL_POSES {
            assert!(frames.contains(&pose), "unused pose {}", pose_name(pose));
        }
        assert_eq!(build_frames().len(), 72);
    }

    #[test]
    fn animation_contract_is_frozen() {
        assert_eq!(
            sha256_hex(frozen_animation_contract_text().as_bytes()),
            FROZEN_ANIMATION_CONTRACT_SHA256
        );
    }
}

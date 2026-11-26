pub const WIDTH: usize = 40;
pub const HEIGHT: usize = 15;
pub const NUM_CELLS: usize = WIDTH * HEIGHT;
pub const WIN_SCORE: u16 = 301;
pub const P2_KOMI: u16 = 18;
pub const TENSOR_CHANNELS: usize = 8;
pub const TENSOR_SIZE: usize = TENSOR_CHANNELS * HEIGHT * WIDTH;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffortLevel {
    Low,
    Med,
    High,
    XHigh,
}

impl EffortLevel {
    pub fn from_opt_str(s: Option<&str>) -> Result<Self, String> {
        match s.map(|v| v.trim().to_ascii_lowercase()).as_deref() {
            None | Some("") | Some("med") | Some("medium") => Ok(Self::Med),
            Some("low") => Ok(Self::Low),
            Some("high") => Ok(Self::High),
            Some("xhigh") | Some("x-high") => Ok(Self::XHigh),
            Some(other) => Err(format!("Unsupported effort level: '{}'", other)),
        }
    }
}

pub const REP_DICE_BUCKETS: [(u8, u8, f32); 5] = [
    (1, 3, 6.0 / 36.0),
    (2, 4, 10.0 / 36.0),
    (3, 4, 7.0 / 36.0),
    (4, 5, 7.0 / 36.0),
    (5, 6, 6.0 / 36.0),
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Bitboard(pub [u64; 10]);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Move {
    pub x: u8,
    pub y: u8,
    pub w: u8,
    pub h: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiceRoll(pub u8, pub u8);

pub const DICE_OUTCOMES: [(u8, u8, f32); 21] = [
    (1,1, 1.0/36.0), (2,2, 1.0/36.0), (3,3, 1.0/36.0),
    (4,4, 1.0/36.0), (5,5, 1.0/36.0), (6,6, 1.0/36.0),
    (1,2, 2.0/36.0), (1,3, 2.0/36.0), (1,4, 2.0/36.0), (1,5, 2.0/36.0), (1,6, 2.0/36.0),
    (2,3, 2.0/36.0), (2,4, 2.0/36.0), (2,5, 2.0/36.0), (2,6, 2.0/36.0),
    (3,4, 2.0/36.0), (3,5, 2.0/36.0), (3,6, 2.0/36.0),
    (4,5, 2.0/36.0), (4,6, 2.0/36.0),
    (5,6, 2.0/36.0),
];

#[derive(Clone, Copy, Debug)]
pub struct GameState {
    pub my_bb: Bitboard,
    pub opp_bb: Bitboard,
    pub my_score: u16,
    pub opp_score: u16,
    pub turn: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct Afterstate {
    pub my_bb: Bitboard,
    pub opp_bb: Bitboard,
    pub my_score: u16,
    pub opp_score: u16,
    pub turn: u16,
}

#[derive(Clone, Debug)]
pub struct TopoFeatures {
    pub cauldron_size_map: [f32; NUM_CELLS],
    pub p_seal_my: [f32; NUM_CELLS],
    pub p_seal_opp: [f32; NUM_CELLS],
    pub my_elasticity: f32,
    pub opp_elasticity: f32,
    pub max_my_cauldron: u16,
    pub max_opp_cauldron: u16,
}

pub trait Evaluator: Send + Sync {
    fn evaluate(&self, state: &Afterstate, features: &TopoFeatures) -> f32;
    fn policy_top_k(&self, state: &GameState, moves: &[Move], k: usize) -> Vec<Move>;
    fn evaluate_batch(&self, items: &[(Afterstate, TopoFeatures)]) -> Vec<f32> {
        items.iter().map(|(s, f)| self.evaluate(s, f)).collect()
    }
}

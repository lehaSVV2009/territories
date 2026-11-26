use std::sync::OnceLock;

use super::{WhaleBoard, WhalePlayer};
use crate::types::NUM_CELLS;

struct ZobristTable {
    cells: [[u64; 3]; NUM_CELLS],
    empty_base: u64,
    player: [u64; 2],
}

static ZOBRIST: OnceLock<ZobristTable> = OnceLock::new();

struct SplitMix64(u64);

impl SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
}

fn get_zobrist() -> &'static ZobristTable {
    ZOBRIST.get_or_init(|| {
        let mut rng = SplitMix64(0x8A5CD78963522B8B);
        let mut cells = [[0u64; 3]; NUM_CELLS];
        let mut empty_base = 0u64;
        for row in cells.iter_mut() {
            for s in 0..3 {
                row[s] = rng.next_u64();
            }
            empty_base ^= row[0];
        }
        let player = [rng.next_u64(), rng.next_u64()];
        ZobristTable {
            cells,
            empty_base,
            player,
        }
    })
}

pub(super) fn hash_board(board: &WhaleBoard, player: WhalePlayer) -> u64 {
    let table = get_zobrist();
    let mut hash = table.empty_base ^ table.player[player.index()];

    for word_idx in 0..10 {
        let mut w1 = board.p1_bb.0[word_idx];
        while w1 != 0 {
            let bit = w1.trailing_zeros() as usize;
            let idx = word_idx * 64 + bit;
            if idx < NUM_CELLS {
                hash ^= table.cells[idx][0] ^ table.cells[idx][1];
            }
            w1 &= w1 - 1;
        }
        let mut w2 = board.p2_bb.0[word_idx];
        while w2 != 0 {
            let bit = w2.trailing_zeros() as usize;
            let idx = word_idx * 64 + bit;
            if idx < NUM_CELLS {
                hash ^= table.cells[idx][0] ^ table.cells[idx][2];
            }
            w2 &= w2 - 1;
        }
    }
    hash
}

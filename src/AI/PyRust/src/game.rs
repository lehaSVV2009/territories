//! Headless game engine for Territories.
//!
//! Implements:
//! - Board representation (15x40 = 600 cells)
//! - Mulberry32 deterministic PRNG
//! - Closed loop flood-fill capture algorithm (findClosedLoops)
//! - 180-degree board rotation and inverse move mapping
//! - Bot policies: GreedyBot, FastBot, LegacyHardBot, SpatialSlayerBot (V2 Base + Tactical patch)
//! - 9-component V2 heuristic evaluator
//! - Procedural DEFAULT_V2_MATRIX generation
//! - Headless match simulation loop (simulateMatch)

use serde::{Deserialize, Serialize};

pub const BOARD_HEIGHT: usize = 15;
pub const BOARD_WIDTH: usize = 40;
pub const TOTAL_CELLS: usize = BOARD_HEIGHT * BOARD_WIDTH;

pub const EMPTY: u8 = 0;
pub const PLAYER_1: u8 = 1;
pub const PLAYER_2: u8 = 2;

/// Player identity enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Player {
    Player1,
    Player2,
}

impl Player {
    #[inline(always)]
    pub fn cell_value(self) -> u8 {
        match self {
            Player::Player1 => PLAYER_1,
            Player::Player2 => PLAYER_2,
        }
    }

    #[inline(always)]
    pub fn opponent(self) -> Player {
        match self {
            Player::Player1 => Player::Player2,
            Player::Player2 => Player::Player1,
        }
    }

    #[inline(always)]
    pub fn from_cell(val: u8) -> Option<Player> {
        match val {
            PLAYER_1 => Some(Player::Player1),
            PLAYER_2 => Some(Player::Player2),
            _ => None,
        }
    }
}

/// A placed rectangle move.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Move {
    pub r: usize,
    pub c: usize,
    pub h: usize,
    pub w: usize,
}

impl Move {
    #[inline(always)]
    pub fn new(r: usize, c: usize, h: usize, w: usize) -> Self {
        Self { r, c, h, w }
    }

    #[inline(always)]
    pub fn area(&self) -> usize {
        self.h * self.w
    }
}

/// The 15x40 board state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    pub cells: [u8; TOTAL_CELLS],
    pub p1_score: u16,
    pub p2_score: u16,
    pub turn: usize,
    pub passes: usize,
}

impl Serialize for Board {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Board", 5)?;
        state.serialize_field("cells", &self.cells.to_vec())?;
        state.serialize_field("p1_score", &self.p1_score)?;
        state.serialize_field("p2_score", &self.p2_score)?;
        state.serialize_field("turn", &self.turn)?;
        state.serialize_field("passes", &self.passes)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Board {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct BoardHelper {
            cells: Vec<u8>,
            p1_score: u16,
            p2_score: u16,
            turn: usize,
            passes: usize,
        }
        let helper = BoardHelper::deserialize(deserializer)?;
        let mut cells = [0u8; TOTAL_CELLS];
        let len = helper.cells.len().min(TOTAL_CELLS);
        cells[..len].copy_from_slice(&helper.cells[..len]);
        Ok(Board {
            cells,
            p1_score: helper.p1_score,
            p2_score: helper.p2_score,
            turn: helper.turn,
            passes: helper.passes,
        })
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    /// Creates a new empty board.
    pub fn new() -> Self {
        Self {
            cells: [EMPTY; TOTAL_CELLS],
            p1_score: 0,
            p2_score: 0,
            turn: 1,
            passes: 0,
        }
    }

    /// Creates a board from an array of 600 cells, recalculating scores.
    pub fn from_cells(cells: [u8; TOTAL_CELLS]) -> Self {
        let mut p1 = 0u16;
        let mut p2 = 0u16;
        for &c in &cells {
            if c == PLAYER_1 {
                p1 += 1;
            } else if c == PLAYER_2 {
                p2 += 1;
            }
        }
        Self {
            cells,
            p1_score: p1,
            p2_score: p2,
            turn: 1,
            passes: 0,
        }
    }

    #[inline(always)]
    pub fn get(&self, r: usize, c: usize) -> u8 {
        self.cells[r * BOARD_WIDTH + c]
    }

    #[inline(always)]
    pub fn set(&mut self, r: usize, c: usize, val: u8) {
        self.cells[r * BOARD_WIDTH + c] = val;
    }

    #[inline(always)]
    pub fn is_empty(&self, r: usize, c: usize) -> bool {
        self.get(r, c) == EMPTY
    }

    #[inline(always)]
    pub fn empty_cells_count(&self) -> usize {
        let occ = (self.p1_score as usize) + (self.p2_score as usize);
        TOTAL_CELLS.saturating_sub(occ)
    }

    #[inline(always)]
    pub fn score(&self, player: Player) -> u16 {
        match player {
            Player::Player1 => self.p1_score,
            Player::Player2 => self.p2_score,
        }
    }

    /// Checks if a rectangle placement is legal according to Territories rules.
    pub fn can_place_rectangle(
        &self,
        player: Player,
        r0: usize,
        c0: usize,
        h: usize,
        w: usize,
    ) -> bool {
        // 1. Boundary check
        if r0 + h > BOARD_HEIGHT || c0 + w > BOARD_WIDTH {
            return false;
        }

        // 2. All target cells must be EMPTY
        for r in r0..r0 + h {
            let row_offset = r * BOARD_WIDTH;
            for c in c0..c0 + w {
                if self.cells[row_offset + c] != EMPTY {
                    return false;
                }
            }
        }

        let p_val = player.cell_value();

        // 3. Opening anchor condition if player has no placed cells yet
        let is_first_move = self.score(player) == 0;
        if is_first_move {
            return match player {
                Player::Player1 => r0 == 0 && c0 == 0,
                Player::Player2 => r0 == BOARD_HEIGHT - h && c0 == BOARD_WIDTH - w,
            };
        }

        // 4. Contact check: Must share at least one orthogonal edge with friendly territory
        let r1 = r0 + h - 1;
        let c1 = c0 + w - 1;

        // Top edge neighbor (row r0 - 1)
        if r0 > 0 {
            let row_offset = (r0 - 1) * BOARD_WIDTH;
            for c in c0..=c1 {
                if self.cells[row_offset + c] == p_val {
                    return true;
                }
            }
        }

        // Bottom edge neighbor (row r1 + 1)
        if r1 + 1 < BOARD_HEIGHT {
            let row_offset = (r1 + 1) * BOARD_WIDTH;
            for c in c0..=c1 {
                if self.cells[row_offset + c] == p_val {
                    return true;
                }
            }
        }

        // Left edge neighbor (col c0 - 1)
        if c0 > 0 {
            let neighbor_col = c0 - 1;
            for r in r0..=r1 {
                if self.cells[r * BOARD_WIDTH + neighbor_col] == p_val {
                    return true;
                }
            }
        }

        // Right edge neighbor (col c1 + 1)
        if c1 + 1 < BOARD_WIDTH {
            let neighbor_col = c1 + 1;
            for r in r0..=r1 {
                if self.cells[r * BOARD_WIDTH + neighbor_col] == p_val {
                    return true;
                }
            }
        }

        false
    }

    /// Generates all legal moves for a given piece size (h, w).
    pub fn generate_legal_moves_single(&self, player: Player, h: usize, w: usize) -> Vec<Move> {
        let mut moves = Vec::new();
        if h > BOARD_HEIGHT || w > BOARD_WIDTH {
            return moves;
        }

        if self.score(player) == 0 {
            let (r, c) = match player {
                Player::Player1 => (0, 0),
                Player::Player2 => (BOARD_HEIGHT - h, BOARD_WIDTH - w),
            };
            if self.can_place_rectangle(player, r, c, h, w) {
                moves.push(Move::new(r, c, h, w));
            }
            return moves;
        }

        let max_r = BOARD_HEIGHT - h;
        let max_c = BOARD_WIDTH - w;
        for r in 0..=max_r {
            for c in 0..=max_c {
                if self.can_place_rectangle(player, r, c, h, w) {
                    moves.push(Move::new(r, c, h, w));
                }
            }
        }
        moves
    }

    /// Generates all legal moves for dice roll (d1, d2), checking both orientations (d1xd2 then d2xd1).
    pub fn generate_legal_moves(&self, player: Player, d1: usize, d2: usize) -> Vec<Move> {
        let mut moves = self.generate_legal_moves_single(player, d1, d2);
        if d1 != d2 {
            let mut moves_alt = self.generate_legal_moves_single(player, d2, d1);
            moves.append(&mut moves_alt);
        }
        moves
    }

    /// Applies a move directly to the board, updating the player's score.
    pub fn apply_move(&mut self, player: Player, mv: &Move) {
        let p_val = player.cell_value();
        for r in mv.r..mv.r + mv.h {
            let row_offset = r * BOARD_WIDTH;
            for c in mv.c..mv.c + mv.w {
                self.cells[row_offset + c] = p_val;
            }
        }
        let area = (mv.h * mv.w) as u16;
        match player {
            Player::Player1 => self.p1_score += area,
            Player::Player2 => self.p2_score += area,
        }
    }

    /// Rotates the board by 180 degrees and swaps Player 1 and Player 2.
    pub fn rotate_board_180(&self) -> Board {
        let mut rotated_cells = [EMPTY; TOTAL_CELLS];
        for r in 0..BOARD_HEIGHT {
            let orig_row_offset = r * BOARD_WIDTH;
            let rot_r = BOARD_HEIGHT - 1 - r;
            let rot_row_offset = rot_r * BOARD_WIDTH;
            for c in 0..BOARD_WIDTH {
                let rot_c = BOARD_WIDTH - 1 - c;
                let val = self.cells[orig_row_offset + c];
                let rot_val = match val {
                    PLAYER_1 => PLAYER_2,
                    PLAYER_2 => PLAYER_1,
                    _ => EMPTY,
                };
                rotated_cells[rot_row_offset + rot_c] = rot_val;
            }
        }
        Board {
            cells: rotated_cells,
            p1_score: self.p2_score,
            p2_score: self.p1_score,
            turn: self.turn,
            passes: self.passes,
        }
    }

    /// Transforms a move calculated on a 180-degree rotated board back to original coordinates.
    #[inline(always)]
    pub fn inverse_move_180(mv: &Move) -> Move {
        Move {
            r: BOARD_HEIGHT - (mv.r + mv.h),
            c: BOARD_WIDTH - (mv.c + mv.w),
            h: mv.h,
            w: mv.w,
        }
    }

    /// Performs full 4-orthogonal BFS flood fill across empty cells to detect enclosed loops.
    /// Returns tuples of captured cell indices for Player 1 and Player 2.
    pub fn find_closed_loops(&self) -> (Vec<usize>, Vec<usize>) {
        let mut visited = [false; TOTAL_CELLS];
        let mut queue = [0u16; TOTAL_CELLS];
        let mut p1_captured = Vec::new();
        let mut p2_captured = Vec::new();

        for start_idx in 0..TOTAL_CELLS {
            if visited[start_idx] || self.cells[start_idx] != EMPTY {
                continue;
            }

            let mut head = 0;
            let mut tail = 0;
            queue[tail] = start_idx as u16;
            tail += 1;
            visited[start_idx] = true;

            let mut touches_p1 = false;
            let mut touches_p2 = false;
            let mut component = Vec::new();

            while head < tail {
                let curr = queue[head] as usize;
                head += 1;
                component.push(curr);

                let cr = curr / BOARD_WIDTH;
                let cc = curr % BOARD_WIDTH;

                // 4-orthogonal neighbors (matrix boundaries are walls, not neighbors)
                if cr > 0 {
                    let n = curr - BOARD_WIDTH;
                    let val = self.cells[n];
                    if val == PLAYER_1 {
                        touches_p1 = true;
                    } else if val == PLAYER_2 {
                        touches_p2 = true;
                    } else if val == EMPTY && !visited[n] {
                        visited[n] = true;
                        queue[tail] = n as u16;
                        tail += 1;
                    }
                }
                if cr < BOARD_HEIGHT - 1 {
                    let n = curr + BOARD_WIDTH;
                    let val = self.cells[n];
                    if val == PLAYER_1 {
                        touches_p1 = true;
                    } else if val == PLAYER_2 {
                        touches_p2 = true;
                    } else if val == EMPTY && !visited[n] {
                        visited[n] = true;
                        queue[tail] = n as u16;
                        tail += 1;
                    }
                }
                if cc > 0 {
                    let n = curr - 1;
                    let val = self.cells[n];
                    if val == PLAYER_1 {
                        touches_p1 = true;
                    } else if val == PLAYER_2 {
                        touches_p2 = true;
                    } else if val == EMPTY && !visited[n] {
                        visited[n] = true;
                        queue[tail] = n as u16;
                        tail += 1;
                    }
                }
                if cc < BOARD_WIDTH - 1 {
                    let n = curr + 1;
                    let val = self.cells[n];
                    if val == PLAYER_1 {
                        touches_p1 = true;
                    } else if val == PLAYER_2 {
                        touches_p2 = true;
                    } else if val == EMPTY && !visited[n] {
                        visited[n] = true;
                        queue[tail] = n as u16;
                        tail += 1;
                    }
                }
            }

            // Only capture if exclusively bounded by one player (and walls)
            if touches_p1 && !touches_p2 {
                p1_captured.extend_from_slice(&component);
            } else if touches_p2 && !touches_p1 {
                p2_captured.extend_from_slice(&component);
            }
        }

        (p1_captured, p2_captured)
    }

    /// Applies closed loop captures to the board and updates scores.
    pub fn apply_closed_loops(&mut self) -> (usize, usize) {
        let (p1_caps, p2_caps) = self.find_closed_loops();
        let c1 = p1_caps.len();
        let c2 = p2_caps.len();
        for &idx in &p1_caps {
            self.cells[idx] = PLAYER_1;
        }
        for &idx in &p2_caps {
            self.cells[idx] = PLAYER_2;
        }
        self.p1_score += c1 as u16;
        self.p2_score += c2 as u16;
        (c1, c2)
    }
}

/// Tommy Ettinger's Mulberry32 PRNG (32-bit state).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mulberry32 {
    pub state: u32,
}

impl Mulberry32 {
    #[inline(always)]
    pub fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    #[inline(always)]
    pub fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_add(0x6d2b79f5);
        let mut t = (self.state ^ (self.state >> 15)).wrapping_mul(1 | self.state);
        t = (t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t))) ^ t;
        t ^ (t >> 14)
    }

    #[inline(always)]
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u32() as f64) / 4294967296.0
    }

    #[inline(always)]
    pub fn roll_die(&mut self) -> usize {
        (self.next_f64() * 6.0).floor() as usize + 1
    }
}

/// Heuristic weights parameter set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeuristicWeights {
    pub weight_area: f64,
    pub weight_capture: f64,
    pub weight_deny_capture: f64,
    pub weight_voronoi_delta: f64,
    pub weight_safe_territory: f64,
    pub weight_compactness: f64,
    pub weight_spatial_matrix: f64,
    pub weight_frontier: f64,
    pub weight_opp_choke: f64,
    pub weight_bottleneck: f64,
    pub weight_guillotine: f64,
    pub weight_gap_penalty: f64,
    pub weight_wall_cork: f64,
    pub weight_pocket_trap: f64,
}

impl Default for HeuristicWeights {
    fn default() -> Self {
        Self {
            weight_area: 50.0,
            weight_capture: 95.0,
            weight_deny_capture: 85.0,
            weight_voronoi_delta: 26.0,
            weight_safe_territory: 25.0,
            weight_compactness: 12.97,
            weight_spatial_matrix: 0.32,
            weight_frontier: 18.0,
            weight_opp_choke: 18.0,
            weight_bottleneck: 14.0,
            weight_guillotine: 12.0,
            weight_gap_penalty: 8.0,
            weight_wall_cork: 35.0,
            weight_pocket_trap: 45.0,
        }
    }
}

impl HeuristicWeights {
    pub fn from_slice(slice: &[f64]) -> Self {
        let def = Self::default();
        Self {
            weight_area: slice.first().copied().unwrap_or(def.weight_area),
            weight_capture: slice.get(1).copied().unwrap_or(def.weight_capture),
            weight_deny_capture: slice.get(2).copied().unwrap_or(def.weight_deny_capture),
            weight_voronoi_delta: slice.get(3).copied().unwrap_or(def.weight_voronoi_delta),
            weight_safe_territory: slice.get(4).copied().unwrap_or(def.weight_safe_territory),
            weight_compactness: slice.get(5).copied().unwrap_or(def.weight_compactness),
            weight_spatial_matrix: slice.get(6).copied().unwrap_or(def.weight_spatial_matrix),
            weight_frontier: slice.get(7).copied().unwrap_or(def.weight_frontier),
            weight_opp_choke: slice.get(8).copied().unwrap_or(def.weight_opp_choke),
            weight_bottleneck: slice.get(9).copied().unwrap_or(def.weight_bottleneck),
            weight_guillotine: slice.get(10).copied().unwrap_or(def.weight_guillotine),
            weight_gap_penalty: slice.get(11).copied().unwrap_or(def.weight_gap_penalty),
            weight_wall_cork: slice.get(12).copied().unwrap_or(def.weight_wall_cork),
            weight_pocket_trap: slice.get(13).copied().unwrap_or(def.weight_pocket_trap),
        }
    }

    pub fn to_vec(&self) -> Vec<f64> {
        vec![
            self.weight_area,
            self.weight_capture,
            self.weight_deny_capture,
            self.weight_voronoi_delta,
            self.weight_safe_territory,
            self.weight_compactness,
            self.weight_spatial_matrix,
            self.weight_frontier,
            self.weight_opp_choke,
            self.weight_bottleneck,
            self.weight_guillotine,
            self.weight_gap_penalty,
            self.weight_wall_cork,
            self.weight_pocket_trap,
        ]
    }
}

/// Generates the standard 600-element spatial prior matrix.
pub fn generate_default_v2_matrix() -> [f64; TOTAL_CELLS] {
    let mut matrix = [0.0; TOTAL_CELLS];
    for r in 0..BOARD_HEIGHT {
        for c in 0..BOARD_WIDTH {
            let idx = r * BOARD_WIDTH + c;
            let mut weight = 10.0;
            let dist_center_col = ((c as f64) - 19.5).abs();
            let dist_center_row = ((r as f64) - 7.0).abs();
            if dist_center_col <= 3.0 {
                weight += (4.0 - dist_center_col) * 8.0;
            }
            if dist_center_row <= 3.0 {
                weight += (4.0 - dist_center_row) * 4.0;
            }
            if r == 0 || r == BOARD_HEIGHT - 1 {
                weight += 6.0;
            }
            if c == 10 || c == 20 || c == 30 {
                weight += 5.0;
            }
            matrix[idx] = weight;
        }
    }
    matrix
}

/// Fast gate filter for loop enclosure possibility.
pub fn can_close_loop(
    board: &Board,
    r0: usize,
    c0: usize,
    h: usize,
    w: usize,
    target: Player,
) -> bool {
    let r1 = r0 + h - 1;
    let c1 = c0 + w - 1;
    let touches_boundary = r0 == 0 || r1 == BOARD_HEIGHT - 1 || c0 == 0 || c1 == BOARD_WIDTH - 1;
    let target_val = target.cell_value();

    let mut friendly_neighbors = 0;
    let mut has_adjacent_empty = false;

    if r0 > 0 {
        let r = r0 - 1;
        for c in c0..=c1 {
            let v = board.get(r, c);
            if v == target_val {
                friendly_neighbors += 1;
            } else if v == EMPTY {
                has_adjacent_empty = true;
            }
        }
    }
    if r1 < BOARD_HEIGHT - 1 {
        let r = r1 + 1;
        for c in c0..=c1 {
            let v = board.get(r, c);
            if v == target_val {
                friendly_neighbors += 1;
            } else if v == EMPTY {
                has_adjacent_empty = true;
            }
        }
    }
    if c0 > 0 {
        let c = c0 - 1;
        for r in r0..=r1 {
            let v = board.get(r, c);
            if v == target_val {
                friendly_neighbors += 1;
            } else if v == EMPTY {
                has_adjacent_empty = true;
            }
        }
    }
    if c1 < BOARD_WIDTH - 1 {
        let c = c1 + 1;
        for r in r0..=r1 {
            let v = board.get(r, c);
            if v == target_val {
                friendly_neighbors += 1;
            } else if v == EMPTY {
                has_adjacent_empty = true;
            }
        }
    }

    if !has_adjacent_empty {
        return false;
    }
    (touches_boundary && friendly_neighbors >= 1) || friendly_neighbors >= 2
}

/// Localized BFS counting empty cells enclosed by placing candidate piece.
pub fn check_enclosed_cells(
    board: &Board,
    r0: usize,
    c0: usize,
    h: usize,
    w: usize,
    target: Player,
) -> usize {
    let r1 = r0 + h - 1;
    let c1 = c0 + w - 1;
    let target_val = target.cell_value();
    let opp_val = target.opponent().cell_value();

    let mut visited = [false; TOTAL_CELLS];
    let mut queue = [0u16; TOTAL_CELLS];
    let mut total_captured = 0;

    let is_inside_candidate =
        |r: usize, c: usize| -> bool { r >= r0 && r <= r1 && c >= c0 && c <= c1 };

    let mut run_bfs = |start_r: usize, start_c: usize| {
        let start_idx = start_r * BOARD_WIDTH + start_c;
        if visited[start_idx] {
            return;
        }

        let mut head = 0;
        let mut tail = 0;
        queue[tail] = start_idx as u16;
        tail += 1;
        visited[start_idx] = true;

        let mut touches_opp = false;
        let mut touches_target = false;
        let mut component_size = 0;

        while head < tail {
            let curr = queue[head] as usize;
            head += 1;
            component_size += 1;
            let cr = curr / BOARD_WIDTH;
            let cc = curr % BOARD_WIDTH;

            // Up
            if cr > 0 {
                let nr = cr - 1;
                let nc = cc;
                if is_inside_candidate(nr, nc) {
                    touches_target = true;
                } else {
                    let v = board.get(nr, nc);
                    if v == opp_val {
                        touches_opp = true;
                    } else if v == target_val {
                        touches_target = true;
                    } else if v == EMPTY {
                        let n_idx = curr - BOARD_WIDTH;
                        if !visited[n_idx] {
                            visited[n_idx] = true;
                            queue[tail] = n_idx as u16;
                            tail += 1;
                        }
                    }
                }
            }

            // Down
            if cr < BOARD_HEIGHT - 1 {
                let nr = cr + 1;
                let nc = cc;
                if is_inside_candidate(nr, nc) {
                    touches_target = true;
                } else {
                    let v = board.get(nr, nc);
                    if v == opp_val {
                        touches_opp = true;
                    } else if v == target_val {
                        touches_target = true;
                    } else if v == EMPTY {
                        let n_idx = curr + BOARD_WIDTH;
                        if !visited[n_idx] {
                            visited[n_idx] = true;
                            queue[tail] = n_idx as u16;
                            tail += 1;
                        }
                    }
                }
            }

            // Left
            if cc > 0 {
                let nr = cr;
                let nc = cc - 1;
                if is_inside_candidate(nr, nc) {
                    touches_target = true;
                } else {
                    let v = board.get(nr, nc);
                    if v == opp_val {
                        touches_opp = true;
                    } else if v == target_val {
                        touches_target = true;
                    } else if v == EMPTY {
                        let n_idx = curr - 1;
                        if !visited[n_idx] {
                            visited[n_idx] = true;
                            queue[tail] = n_idx as u16;
                            tail += 1;
                        }
                    }
                }
            }

            // Right
            if cc < BOARD_WIDTH - 1 {
                let nr = cr;
                let nc = cc + 1;
                if is_inside_candidate(nr, nc) {
                    touches_target = true;
                } else {
                    let v = board.get(nr, nc);
                    if v == opp_val {
                        touches_opp = true;
                    } else if v == target_val {
                        touches_target = true;
                    } else if v == EMPTY {
                        let n_idx = curr + 1;
                        if !visited[n_idx] {
                            visited[n_idx] = true;
                            queue[tail] = n_idx as u16;
                            tail += 1;
                        }
                    }
                }
            }
        }

        if !touches_opp && touches_target {
            total_captured += component_size;
        }
    };

    if r0 > 0 {
        for c in c0..=c1 {
            if board.get(r0 - 1, c) == EMPTY {
                run_bfs(r0 - 1, c);
            }
        }
    }
    if r1 < BOARD_HEIGHT - 1 {
        for c in c0..=c1 {
            if board.get(r1 + 1, c) == EMPTY {
                run_bfs(r1 + 1, c);
            }
        }
    }
    if c0 > 0 {
        for r in r0..=r1 {
            if board.get(r, c0 - 1) == EMPTY {
                run_bfs(r, c0 - 1);
            }
        }
    }
    if c1 < BOARD_WIDTH - 1 {
        for r in r0..=r1 {
            if board.get(r, c1 + 1) == EMPTY {
                run_bfs(r, c1 + 1);
            }
        }
    }

    total_captured
}

/// Resulting metrics from multi-source Voronoi influence field BFS.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoronoiField {
    pub p1_territory: i32,
    pub p2_territory: i32,
    pub p2_safe_territory: i32,
    pub p1_safe_territory: i32,
    pub opp_choked_cells: i32,
    pub spatial_score: f64,
}

/// Dual multi-source BFS evaluating territorial control and spatial score.
pub fn compute_voronoi_influence_v2(
    board: &Board,
    r0: usize,
    c0: usize,
    h: usize,
    w: usize,
    placing_player: Player,
    matrix: &[f64; TOTAL_CELLS],
) -> VoronoiField {
    let r1 = r0 + h - 1;
    let c1 = c0 + w - 1;

    let mut dist1 = [9999i16; TOTAL_CELLS];
    let mut dist2 = [9999i16; TOTAL_CELLS];

    let mut q1 = [0u16; TOTAL_CELLS];
    let mut q2 = [0u16; TOTAL_CELLS];
    let mut head1 = 0;
    let mut tail1 = 0;
    let mut head2 = 0;
    let mut tail2 = 0;

    for r in 0..BOARD_HEIGHT {
        let row_offset = r * BOARD_WIDTH;
        for c in 0..BOARD_WIDTH {
            let idx = row_offset + c;
            let cell = board.cells[idx];
            let is_inside_cand = r >= r0 && r <= r1 && c >= c0 && c <= c1;

            if is_inside_cand {
                if placing_player == Player::Player2 {
                    dist2[idx] = 0;
                    q2[tail2] = idx as u16;
                    tail2 += 1;
                } else {
                    dist1[idx] = 0;
                    q1[tail1] = idx as u16;
                    tail1 += 1;
                }
            } else if cell == PLAYER_1 {
                dist1[idx] = 0;
                q1[tail1] = idx as u16;
                tail1 += 1;
            } else if cell == PLAYER_2 {
                dist2[idx] = 0;
                q2[tail2] = idx as u16;
                tail2 += 1;
            }
        }
    }

    // BFS Player 1
    while head1 < tail1 {
        let curr = q1[head1] as usize;
        head1 += 1;
        let d = dist1[curr] + 1;
        let cr = curr / BOARD_WIDTH;
        let cc = curr % BOARD_WIDTH;

        if cr > 0 {
            let n = curr - BOARD_WIDTH;
            let is_opp = dist2[n] == 0;
            if !is_opp && dist1[n] > d {
                dist1[n] = d;
                q1[tail1] = n as u16;
                tail1 += 1;
            }
        }
        if cr < BOARD_HEIGHT - 1 {
            let n = curr + BOARD_WIDTH;
            let is_opp = dist2[n] == 0;
            if !is_opp && dist1[n] > d {
                dist1[n] = d;
                q1[tail1] = n as u16;
                tail1 += 1;
            }
        }
        if cc > 0 {
            let n = curr - 1;
            let is_opp = dist2[n] == 0;
            if !is_opp && dist1[n] > d {
                dist1[n] = d;
                q1[tail1] = n as u16;
                tail1 += 1;
            }
        }
        if cc < BOARD_WIDTH - 1 {
            let n = curr + 1;
            let is_opp = dist2[n] == 0;
            if !is_opp && dist1[n] > d {
                dist1[n] = d;
                q1[tail1] = n as u16;
                tail1 += 1;
            }
        }
    }

    // BFS Player 2
    while head2 < tail2 {
        let curr = q2[head2] as usize;
        head2 += 1;
        let d = dist2[curr] + 1;
        let cr = curr / BOARD_WIDTH;
        let cc = curr % BOARD_WIDTH;

        if cr > 0 {
            let n = curr - BOARD_WIDTH;
            let is_opp = dist1[n] == 0;
            if !is_opp && dist2[n] > d {
                dist2[n] = d;
                q2[tail2] = n as u16;
                tail2 += 1;
            }
        }
        if cr < BOARD_HEIGHT - 1 {
            let n = curr + BOARD_WIDTH;
            let is_opp = dist1[n] == 0;
            if !is_opp && dist2[n] > d {
                dist2[n] = d;
                q2[tail2] = n as u16;
                tail2 += 1;
            }
        }
        if cc > 0 {
            let n = curr - 1;
            let is_opp = dist1[n] == 0;
            if !is_opp && dist2[n] > d {
                dist2[n] = d;
                q2[tail2] = n as u16;
                tail2 += 1;
            }
        }
        if cc < BOARD_WIDTH - 1 {
            let n = curr + 1;
            let is_opp = dist1[n] == 0;
            if !is_opp && dist2[n] > d {
                dist2[n] = d;
                q2[tail2] = n as u16;
                tail2 += 1;
            }
        }
    }

    let mut p1_territory = 0;
    let mut p2_territory = 0;
    let mut p2_safe_territory = 0;
    let mut p1_safe_territory = 0;
    let mut opp_choked_cells = 0;
    let mut spatial_score = 0.0;

    for i in 0..TOTAL_CELLS {
        let d1 = dist1[i];
        let d2 = dist2[i];

        if d1 != 0 && d2 != 0 {
            let cell_value = matrix[i];

            if d2 < d1 {
                p2_territory += 1;
                spatial_score += cell_value;
                if d1 >= 9999 {
                    p2_safe_territory += 1;
                    spatial_score += cell_value * 2.0;
                }
            } else if d1 < d2 {
                p1_territory += 1;
                spatial_score -= cell_value;
                if d2 >= 9999 {
                    p1_safe_territory += 1;
                    spatial_score -= cell_value * 2.0;
                }
            }

            if d1 <= 2 && d2 <= d1 + 1 {
                opp_choked_cells += 1;
            }
        }
    }

    VoronoiField {
        p1_territory,
        p2_territory,
        p2_safe_territory,
        p1_safe_territory,
        opp_choked_cells,
        spatial_score,
    }
}

/// Detects 1-cell unfillable gaps created along the candidate's perimeter in the endgame.
pub fn compute_endgame_gap_penalty(
    board: &Board,
    r0: usize,
    c0: usize,
    h: usize,
    w: usize,
) -> usize {
    let mut dead_gaps = 0;

    if r0 == 1 {
        for c in c0..c0 + w {
            if board.get(0, c) == EMPTY {
                dead_gaps += 1;
            }
        }
    } else if r0 > 1 {
        for c in c0..c0 + w {
            if board.get(r0 - 1, c) == EMPTY && board.get(r0 - 2, c) != EMPTY {
                dead_gaps += 1;
            }
        }
    }

    if r0 + h == BOARD_HEIGHT - 1 {
        for c in c0..c0 + w {
            if board.get(BOARD_HEIGHT - 1, c) == EMPTY {
                dead_gaps += 1;
            }
        }
    } else if r0 + h < BOARD_HEIGHT - 1 {
        for c in c0..c0 + w {
            if board.get(r0 + h, c) == EMPTY && board.get(r0 + h + 1, c) != EMPTY {
                dead_gaps += 1;
            }
        }
    }

    if c0 == 1 {
        for r in r0..r0 + h {
            if board.get(r, 0) == EMPTY {
                dead_gaps += 1;
            }
        }
    } else if c0 > 1 {
        for r in r0..r0 + h {
            if board.get(r, c0 - 1) == EMPTY && board.get(r, c0 - 2) != EMPTY {
                dead_gaps += 1;
            }
        }
    }

    if c0 + w == BOARD_WIDTH - 1 {
        for r in r0..r0 + h {
            if board.get(r, BOARD_WIDTH - 1) == EMPTY {
                dead_gaps += 1;
            }
        }
    } else if c0 + w < BOARD_WIDTH - 1 {
        for r in r0..r0 + h {
            if board.get(r, c0 + w) == EMPTY && board.get(r, c0 + w + 1) != EMPTY {
                dead_gaps += 1;
            }
        }
    }

    dead_gaps
}

/// 2-Ply counter-threat lookahead probing dangerous opponent rolls.
pub fn evaluate_counter_threat(
    board: &Board,
    r0: usize,
    c0: usize,
    h: usize,
    w: usize,
) -> usize {
    const TEST_DICES: [(usize, usize); 4] = [(1, 5), (2, 4), (3, 3), (4, 2)];

    let mut temp_board = board.clone();
    for r in r0..r0 + h {
        let row_offset = r * BOARD_WIDTH;
        for c in c0..c0 + w {
            temp_board.cells[row_offset + c] = PLAYER_2;
        }
    }
    temp_board.p2_score += (h * w) as u16;

    let mut max_opp_capture = 0;
    for &(dh, dw) in &TEST_DICES {
        let opp_rects = temp_board.generate_legal_moves_single(Player::Player1, dh, dw);
        for &or in &opp_rects {
            if can_close_loop(&temp_board, or.r, or.c, or.h, or.w, Player::Player1) {
                let captured = check_enclosed_cells(
                    &temp_board,
                    or.r,
                    or.c,
                    or.h,
                    or.w,
                    Player::Player1,
                );
                if captured > max_opp_capture {
                    max_opp_capture = captured;
                }
            }
        }
    }

    max_opp_capture
}

/// Evaluates a candidate move from Player 2's perspective using the 9-component V2 formula.
pub fn evaluate_move_v2(
    board: &Board,
    mv: &Move,
    params: &HeuristicWeights,
    matrix: &[f64; TOTAL_CELLS],
) -> f64 {
    let r0 = mv.r;
    let c0 = mv.c;
    let h = mv.h;
    let w = mv.w;
    let area = h * w;
    let min_dim = if h < w { h } else { w };

    // Board scan
    let mut empty_cells = 0;
    let mut p1_max_col = 0;
    let mut p1_min_col = BOARD_WIDTH - 1;
    let mut p1_max_row = 0;
    let mut p1_col_at_max_row = 0;
    let mut p1_min_row = BOARD_HEIGHT - 1;
    let mut p1_col_at_min_row = 0;
    let mut p2_min_col = BOARD_WIDTH - 1;

    for r in 0..BOARD_HEIGHT {
        let row_offset = r * BOARD_WIDTH;
        for c in 0..BOARD_WIDTH {
            let cell = board.cells[row_offset + c];
            if cell == EMPTY {
                empty_cells += 1;
            } else if cell == PLAYER_1 {
                if c > p1_max_col {
                    p1_max_col = c;
                }
                if c < p1_min_col {
                    p1_min_col = c;
                }
                if r > p1_max_row {
                    p1_max_row = r;
                    p1_col_at_max_row = c;
                }
                if r < p1_min_row {
                    p1_min_row = r;
                    p1_col_at_min_row = c;
                }
            } else if cell == PLAYER_2 && c < p2_min_col {
                p2_min_col = c;
            }
        }
    }

    let is_opening = empty_cells > 490;
    let is_endgame = empty_cells < 180;
    let has_contact = p2_min_col <= p1_max_col + 3;

    let p1_touches_bottom_wall = p1_max_row == BOARD_HEIGHT - 1;
    let p1_touches_top_wall = p1_min_row == 0;
    let p1_threatens_bottom_wall = !p1_touches_bottom_wall && p1_max_row >= 8 && p1_col_at_max_row >= 4;
    let p1_threatens_top_wall = !p1_touches_top_wall && p1_min_row <= 6 && p1_col_at_min_row >= 4;

    // 1. Base Area
    let mut move_score = (area as f64) * params.weight_area;

    // 2. Compactness vs Noodle
    if is_opening && !has_contact {
        if w > h {
            move_score += ((w - h) as f64) * 4.0;
        }
    } else if min_dim >= 2 {
        move_score += params.weight_compactness;
    } else if min_dim == 1 && area > 2 {
        move_score -= params.weight_compactness * 1.5;
    }

    // 3. Loop Enclosure Check (Friendly Capture)
    if can_close_loop(board, r0, c0, h, w, Player::Player2) {
        let captured = check_enclosed_cells(board, r0, c0, h, w, Player::Player2);
        if captured > 0 {
            move_score += (captured as f64) * params.weight_capture;
        }
    }

    // 4. Opponent Loop Enclosure Denial
    if can_close_loop(board, r0, c0, h, w, Player::Player1) {
        let opp_would_capture = check_enclosed_cells(board, r0, c0, h, w, Player::Player1);
        if opp_would_capture > 0 {
            move_score += (opp_would_capture as f64) * params.weight_deny_capture;
        }
    }

    // 5. Voronoi Field Evaluation
    let field = compute_voronoi_influence_v2(board, r0, c0, h, w, Player::Player2, matrix);
    move_score += ((field.p2_territory - field.p1_territory) as f64) * params.weight_voronoi_delta;
    move_score += (field.p2_safe_territory as f64) * params.weight_safe_territory;
    move_score -= (field.p1_safe_territory as f64) * params.weight_safe_territory;
    move_score += field.spatial_score * params.weight_spatial_matrix;

    // 6. Frontier Advancement & Vertical Equator Centering
    let center_row_dist = ((r0 as f64 + (h as f64) * 0.5) - 7.0).abs() / 7.0;
    let vertical_center_bonus = 1.0 - center_row_dist;

    if is_opening {
        move_score += ((((39.0 - (c0 as f64)) / 39.0) * 1.4) + vertical_center_bonus) * params.weight_frontier;
    } else if !is_endgame {
        move_score += (((39.0 - (c0 as f64)) / 39.0) + vertical_center_bonus * 0.5) * params.weight_frontier;
    } else {
        move_score += ((39.0 - (c0 as f64)) / 39.0) * (params.weight_frontier * 0.2);
    }

    // 7. Opponent Choke & Corridor Denial
    if field.opp_choked_cells > 0 {
        move_score += (field.opp_choked_cells as f64) * params.weight_opp_choke;
    }

    // 8. Vertical Guillotine / Frontline Cut
    if has_contact && c0 >= p1_max_col.saturating_sub(2) && c0 <= p1_max_col + 3 && h > w {
        move_score += (h as f64) * params.weight_guillotine;
    }

    // 9. Bottleneck anchor at border walls
    let touches_wall = r0 == 0 || r0 + h == BOARD_HEIGHT;
    if touches_wall && c0 < 25 && h >= 2 {
        move_score += params.weight_bottleneck;
    }

    // 10. Wall Cork / Interception
    if p1_threatens_bottom_wall && r0 + h == BOARD_HEIGHT {
        let min_c = p1_col_at_max_row.saturating_sub(1);
        let max_c = p1_col_at_max_row + 5;
        if c0 >= min_c && c0 <= max_c {
            move_score += params.weight_wall_cork;
        }
    }
    if p1_threatens_top_wall && r0 == 0 {
        let min_c = p1_col_at_min_row.saturating_sub(1);
        let max_c = p1_col_at_min_row + 5;
        if c0 >= min_c && c0 <= max_c {
            move_score += params.weight_wall_cork;
        }
    }

    // 11. Anti-Pocket Suicide Trap Penalty
    if p1_threatens_bottom_wall && c0 + w <= p1_col_at_max_row && r0 >= 8 {
        move_score -= params.weight_pocket_trap;
    }
    if p1_threatens_top_wall && c0 + w <= p1_col_at_min_row && r0 + h <= 6 {
        move_score -= params.weight_pocket_trap;
    }

    // 12. Anti-Fragmentation Gap Penalty
    if is_endgame {
        let dead_gaps = compute_endgame_gap_penalty(board, r0, c0, h, w);
        if dead_gaps > 0 {
            move_score -= (dead_gaps as f64) * params.weight_gap_penalty;
        }
    }

    move_score
}

/// Bot policy types supported by the simulation engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BotType {
    Greedy,
    Fast,
    LegacyHard,
    SpatialSlayer,
}

/// GreedyBot: Area maximizer with deterministic tie-breaking (smallest r, smallest c).
pub fn select_move_greedy(board: &Board, player: Player, d1: usize, d2: usize) -> Option<Move> {
    let all_rects = board.generate_legal_moves(player, d1, d2);
    if all_rects.is_empty() {
        return None;
    }

    let mut best_rect = all_rects[0];
    let mut max_area = best_rect.area();

    for &rect in all_rects.iter().skip(1) {
        let area = rect.area();
        if area > max_area
            || (area == max_area && rect.r < best_rect.r)
            || (area == max_area && rect.r == best_rect.r && rect.c < best_rect.c)
        {
            max_area = area;
            best_rect = rect;
        }
    }

    Some(best_rect)
}

/// FastBot: Fast candidate evaluator with linear heuristics.
pub fn select_move_fast(board: &Board, player: Player, d1: usize, d2: usize) -> Option<Move> {
    if player == Player::Player1 {
        let rotated = board.rotate_board_180();
        let rot_mv = select_move_fast_as_p2(&rotated, d1, d2)?;
        return Some(Board::inverse_move_180(&rot_mv));
    }
    select_move_fast_as_p2(board, d1, d2)
}

fn select_move_fast_as_p2(board: &Board, d1: usize, d2: usize) -> Option<Move> {
    let all_rects = board.generate_legal_moves(Player::Player2, d1, d2);
    if all_rects.is_empty() {
        return None;
    }

    let mut p2_cells = 0;
    let mut p1_max_col = 0;
    let mut p2_min_col = BOARD_WIDTH;

    for r in 0..BOARD_HEIGHT {
        let row_offset = r * BOARD_WIDTH;
        for c in 0..BOARD_WIDTH {
            let cell = board.cells[row_offset + c];
            if cell == PLAYER_1 {
                if c > p1_max_col {
                    p1_max_col = c;
                }
            } else if cell == PLAYER_2 {
                p2_cells += 1;
                if c < p2_min_col {
                    p2_min_col = c;
                }
            }
        }
    }

    let contact_col = p1_max_col + 1;
    let has_contact = p2_min_col <= contact_col + 2;

    let mut best_rect = all_rects[0];
    let mut best_score = -f64::INFINITY;

    for &rect in &all_rects {
        let r0 = rect.r;
        let c0 = rect.c;
        let h = rect.h;
        let w = rect.w;
        let r1 = r0 + h - 1;
        let c1 = c0 + w - 1;
        let area = h * w;
        let min_dim = if h < w { h } else { w };

        let mut score = (area as f64) * 1000.0;

        if min_dim >= 2 {
            score += 500.0;
        } else if min_dim == 1 && area > 2 {
            score -= 700.0;
        }

        score += ((BOARD_WIDTH - 1 - c0) as f64) * 150.0;
        score += ((BOARD_HEIGHT - 1 - r0) as f64) * 35.0;

        if c0 <= 21 && c1 >= 18 {
            score += 1800.0;
        }

        if c0 > p2_min_col {
            score -= 1500.0;
        }

        if c0 <= contact_col {
            score += 2500.0;
        }

        if has_contact && c0 <= contact_col + 2 {
            score += (h as f64) * 400.0;
            if r0 == 0 {
                score += 2600.0;
            }
            if r1 == BOARD_HEIGHT - 1 {
                score += 2600.0;
            }
        }

        if p2_cells >= 25 && can_close_loop(board, r0, c0, h, w, Player::Player2) {
            let auto_captured = check_enclosed_cells(board, r0, c0, h, w, Player::Player2);
            if auto_captured > 0 {
                score += (auto_captured as f64) * 32000.0;
            }
        }

        if score > best_score {
            best_score = score;
            best_rect = rect;
        } else if (score - best_score).abs() < 1e-9 {
            let best_area = best_rect.area();
            if area > best_area
                || (area == best_area && r0 < best_rect.r)
                || (area == best_area && r0 == best_rect.r && c0 < best_rect.c)
            {
                best_score = score;
                best_rect = rect;
            }
        }
    }

    Some(best_rect)
}

/// LegacyHardBot: The geometric baseline bot ported from the original master branch.
pub fn select_move_legacy_hard(
    board: &Board,
    player: Player,
    d1: usize,
    d2: usize,
    mut rng: Option<&mut Mulberry32>,
) -> Option<Move> {
    let rectangles = board.generate_legal_moves(player, d1, d2);
    if rectangles.is_empty() {
        return None;
    }

    let board_center_r = BOARD_HEIGHT / 2; // 7
    let board_center_c = BOARD_WIDTH / 2; // 20

    let chosen = if board.get(board_center_r, board_center_c) == EMPTY {
        let center_rects: Vec<Move> = rectangles
            .iter()
            .copied()
            .filter(|r| {
                board_center_r >= r.r
                    && board_center_r < r.r + r.h
                    && board_center_c >= r.c
                    && board_center_c < r.c + r.w
            })
            .collect();

        if !center_rects.is_empty() {
            find_closest_rectangle(
                &center_rects,
                board_center_r as f64,
                board_center_c as f64,
                EdgeAnchor::CenterRight,
            )
        } else {
            find_closest_rectangle(
                &rectangles,
                board_center_r as f64,
                board_center_c as f64,
                EdgeAnchor::TopLeft,
            )
        }
    } else if board.get(0, board_center_c) == EMPTY {
        let jitter = if let Some(ref mut r) = rng {
            (r.next_f64() * 4.0).floor() as usize
        } else {
            0
        };
        let target_c = (board_center_c.saturating_sub(jitter)) as f64;
        find_closest_rectangle(&rectangles, 0.0, target_c, EdgeAnchor::TopRight)
    } else if board.get(BOARD_HEIGHT - 1, board_center_c) == EMPTY {
        let jitter = if let Some(ref mut r) = rng {
            (r.next_f64() * 4.0).floor() as usize
        } else {
            0
        };
        let target_c = (board_center_c.saturating_sub(jitter)) as f64;
        find_closest_rectangle(
            &rectangles,
            (BOARD_HEIGHT - 1) as f64,
            target_c,
            EdgeAnchor::BottomRight,
        )
    } else {
        rectangles[0]
    };

    Some(chosen)
}

#[derive(Clone, Copy)]
enum EdgeAnchor {
    TopLeft,
    TopRight,
    BottomRight,
    CenterRight,
}

fn find_closest_rectangle(
    rects: &[Move],
    target_r: f64,
    target_c: f64,
    edge: EdgeAnchor,
) -> Move {
    let mut chosen = rects[0];
    let (shift_r, shift_c) = get_shift(edge, chosen);
    let mut min_dist = ((chosen.r as f64 + shift_r - target_r).powi(2)
        + (chosen.c as f64 + shift_c - target_c).powi(2))
    .sqrt();

    for &r in rects.iter().skip(1) {
        let (sr, sc) = get_shift(edge, r);
        let dist = ((r.r as f64 + sr - target_r).powi(2) + (r.c as f64 + sc - target_c).powi(2)).sqrt();
        if dist < min_dist {
            min_dist = dist;
            chosen = r;
        }
    }
    chosen
}

fn get_shift(edge: EdgeAnchor, r: Move) -> (f64, f64) {
    match edge {
        EdgeAnchor::CenterRight => ((r.h / 2) as f64, r.w as f64),
        EdgeAnchor::TopLeft => (0.0, 0.0),
        EdgeAnchor::TopRight => (0.0, r.w as f64),
        EdgeAnchor::BottomRight => (r.h as f64, r.w as f64),
    }
}

/// SpatialSlayerBot: Evaluates candidates using the V2 Base + Tactical patch with 2-ply lookahead.
pub fn select_move_spatial_slayer(
    board: &Board,
    player: Player,
    d1: usize,
    d2: usize,
    weights: &HeuristicWeights,
    matrix: &[f64; TOTAL_CELLS],
) -> Option<Move> {
    if player == Player::Player1 {
        let rotated = board.rotate_board_180();
        let rot_mv = select_move_spatial_slayer_as_p2(&rotated, d1, d2, weights, matrix)?;
        return Some(Board::inverse_move_180(&rot_mv));
    }
    select_move_spatial_slayer_as_p2(board, d1, d2, weights, matrix)
}

fn select_move_spatial_slayer_as_p2(
    board: &Board,
    d1: usize,
    d2: usize,
    weights: &HeuristicWeights,
    matrix: &[f64; TOTAL_CELLS],
) -> Option<Move> {
    let all_rects = board.generate_legal_moves(Player::Player2, d1, d2);
    if all_rects.is_empty() {
        return None;
    }

    // Board analysis for contact and opening phase
    let mut empty_cells = 0;
    let mut p1_max_col = 0;
    let mut p2_min_col = BOARD_WIDTH;

    for r in 0..BOARD_HEIGHT {
        let row_offset = r * BOARD_WIDTH;
        for c in 0..BOARD_WIDTH {
            let cell = board.cells[row_offset + c];
            if cell == EMPTY {
                empty_cells += 1;
            } else if cell == PLAYER_1 {
                if c > p1_max_col {
                    p1_max_col = c;
                }
            } else if cell == PLAYER_2 && c < p2_min_col {
                p2_min_col = c;
            }
        }
    }

    let is_opening = empty_cells > 490;
    let has_contact = p2_min_col <= p1_max_col + 3;

    // Score all candidates
    let mut scored_candidates: Vec<(Move, f64)> = all_rects
        .iter()
        .map(|&rect| {
            let score = evaluate_move_v2(board, &rect, weights, matrix);
            (rect, score)
        })
        .collect();

    // Sort descending by score (stable sort preserving row-major ordering)
    scored_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Top-4 2-Ply Counter-Attack Lookahead
    let top_count = scored_candidates.len().min(4);
    let mut best_cand = scored_candidates[0].0;
    let mut best_total_score = -f64::INFINITY;

    for cand in scored_candidates.iter().take(top_count) {
        let mut adjusted_score = cand.1;

        if has_contact || !is_opening {
            let opp_threat = evaluate_counter_threat(board, cand.0.r, cand.0.c, cand.0.h, cand.0.w);
            if opp_threat > 0 {
                adjusted_score -= (opp_threat as f64) * (weights.weight_deny_capture * 0.8);
            }
        }

        if adjusted_score > best_total_score {
            best_total_score = adjusted_score;
            best_cand = cand.0;
        }
    }

    Some(best_cand)
}

/// Dispatches move selection based on the bot type.
pub fn select_move(
    bot: BotType,
    board: &Board,
    player: Player,
    d1: usize,
    d2: usize,
    weights: Option<&HeuristicWeights>,
    matrix: Option<&[f64; TOTAL_CELLS]>,
    rng: Option<&mut Mulberry32>,
) -> Option<Move> {
    match bot {
        BotType::Greedy => select_move_greedy(board, player, d1, d2),
        BotType::Fast => select_move_fast(board, player, d1, d2),
        BotType::LegacyHard => select_move_legacy_hard(board, player, d1, d2, rng),
        BotType::SpatialSlayer => {
            let default_weights = HeuristicWeights::default();
            let default_matrix = generate_default_v2_matrix();
            let w = weights.unwrap_or(&default_weights);
            let m = matrix.unwrap_or(&default_matrix);
            select_move_spatial_slayer(board, player, d1, d2, w, m)
        }
    }
}

/// Output of a completed headless match.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchResult {
    pub winner: Option<Player>,
    pub p1_score: u16,
    pub p2_score: u16,
    pub turns: usize,
}

/// Runs a complete headless match between two bot policies.
pub fn simulate_match(
    p1_bot: BotType,
    p2_bot: BotType,
    seed: Option<u32>,
    max_turns: usize,
    p1_weights: Option<&HeuristicWeights>,
    p2_weights: Option<&HeuristicWeights>,
    p1_matrix: Option<&[f64; TOTAL_CELLS]>,
    p2_matrix: Option<&[f64; TOTAL_CELLS]>,
) -> MatchResult {
    let mut rng = Mulberry32::new(seed.unwrap_or(123456789));
    let mut board = Board::new();
    let mut passes = 0;
    let mut turn = 1;

    while passes < 4 && turn <= max_turns {
        let current_player = if turn % 2 == 1 {
            Player::Player1
        } else {
            Player::Player2
        };

        let bot = if current_player == Player::Player1 {
            p1_bot
        } else {
            p2_bot
        };
        let bot_weights = if current_player == Player::Player1 {
            p1_weights
        } else {
            p2_weights
        };
        let bot_matrix = if current_player == Player::Player1 {
            p1_matrix
        } else {
            p2_matrix
        };

        let mut chosen_move = None;
        for _ in 0..3 {
            let d1 = rng.roll_die();
            let d2 = rng.roll_die();
            let mv = select_move(
                bot,
                &board,
                current_player,
                d1,
                d2,
                bot_weights,
                bot_matrix,
                Some(&mut rng),
            );
            if mv.is_some() {
                chosen_move = mv;
                break;
            }
        }

        if let Some(mv) = chosen_move {
            passes = 0;
            board.apply_move(current_player, &mv);

            // Closed loops capture check: only when both players have score > 0
            if board.p1_score > 0 && board.p2_score > 0 {
                board.apply_closed_loops();
            }
        } else {
            passes += 1;
        }

        // Victory condition: absolute majority (> 300 cells)
        if board.p1_score > 300 {
            return MatchResult {
                winner: Some(Player::Player1),
                p1_score: board.p1_score,
                p2_score: board.p2_score,
                turns: turn,
            };
        }
        if board.p2_score > 300 {
            return MatchResult {
                winner: Some(Player::Player2),
                p1_score: board.p1_score,
                p2_score: board.p2_score,
                turns: turn,
            };
        }

        turn += 1;
    }

    let winner = if board.p1_score > board.p2_score {
        Some(Player::Player1)
    } else if board.p2_score > board.p1_score {
        Some(Player::Player2)
    } else {
        None
    };

    MatchResult {
        winner,
        p1_score: board.p1_score,
        p2_score: board.p2_score,
        turns: turn - 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mulberry32_prng_seed_123456789_exact_parity() {
        let mut rng = Mulberry32::new(123456789);

        // Exact verification vector directly from Tommy Ettinger's Mulberry32 in Node.js
        let expected_u32 = [
            0x41fe92feu32,
            0xf8848567u32,
            0xc90b41bau32,
            0x34c733b0u32,
            0x4d961e85u32,
            0xbf3fb870u32,
            0xc75b16b0u32,
            0x48d59f80u32,
            0x043bc3bbu32,
            0x2955c005u32,
        ];

        let expected_dice = [2usize, 6, 5, 2, 2, 5, 5, 2, 1, 1];

        let mut rng_u32 = Mulberry32::new(123456789);
        for &exp in &expected_u32 {
            assert_eq!(rng_u32.next_u32(), exp);
        }

        for &exp_die in &expected_dice {
            assert_eq!(rng.roll_die(), exp_die);
        }
    }

    #[test]
    fn test_board_rotate_180_and_inverse_move() {
        let mut board = Board::new();
        // Place P1 at (0, 0, 3, 4)
        board.apply_move(Player::Player1, &Move::new(0, 0, 3, 4));
        // Place P2 at (12, 35, 3, 5)
        board.apply_move(Player::Player2, &Move::new(12, 35, 3, 5));

        assert_eq!(board.p1_score, 12);
        assert_eq!(board.p2_score, 15);

        let rot = board.rotate_board_180();

        // In rotated board:
        // P1 was at (0, 0, 3, 4). Rotated coords: r' = 14 - r, c' = 39 - c.
        // It spans r' in 12..=14, c' in 36..=39 and holds cell type PLAYER_2.
        for r in 12..15 {
            for c in 36..40 {
                assert_eq!(rot.get(r, c), PLAYER_2);
            }
        }
        // P2 was at (12, 35, 3, 5). Spans r' in 0..=2, c' in 0..=4 and holds cell type PLAYER_1.
        for r in 0..3 {
            for c in 0..5 {
                assert_eq!(rot.get(r, c), PLAYER_1);
            }
        }

        // Swapped scores
        assert_eq!(rot.p1_score, 15);
        assert_eq!(rot.p2_score, 12);

        // Inverse move testing:
        // P2 move at bottom-right (12, 35, 3, 5) on rotated board
        let rot_mv = Move::new(12, 35, 3, 5);
        let real_mv = Board::inverse_move_180(&rot_mv);
        assert_eq!(real_mv, Move::new(0, 0, 3, 5));
    }

    #[test]
    fn test_find_closed_loops_enclosure() {
        let mut board = Board::new();

        // Build a chamber trapped by Player 1 along top wall:
        // Top wall is row 0.
        // Wall from (0, 10) down to (3, 10), then across (3, 10..=15), then up to (0, 15).
        for r in 0..=3 {
            board.set(r, 10, PLAYER_1);
            board.set(r, 15, PLAYER_1);
        }
        for c in 10..=15 {
            board.set(3, c, PLAYER_1);
        }
        board.p1_score = 14;

        // Player 2 occupies bottom-right corner (14, 39)
        board.set(14, 39, PLAYER_2);
        board.p2_score = 1;

        // The empty cells inside are (r=0..=2, c=11..=14) = 3 * 4 = 12 cells.
        let (p1_caps, p2_caps) = board.find_closed_loops();
        assert_eq!(p1_caps.len(), 12);
        assert_eq!(p2_caps.len(), 0);

        for r in 0..=2 {
            for c in 11..=14 {
                assert!(p1_caps.contains(&(r * BOARD_WIDTH + c)));
            }
        }

        // Now touch this chamber with Player 2 at (1, 10):
        board.set(1, 10, PLAYER_2);
        let (p1_contested, p2_contested) = board.find_closed_loops();
        // Now it borders both P1 and P2 -> neither captures!
        assert_eq!(p1_contested.len(), 0);
        assert_eq!(p2_contested.len(), 0);
    }

    #[test]
    fn test_match_termination_consecutive_passes() {
        // Verify that board completely full rejects any rectangle placement
        let mut full_board = Board::new();
        for i in 0..TOTAL_CELLS {
            full_board.cells[i] = PLAYER_1;
        }
        full_board.p1_score = 300;
        full_board.p2_score = 300;
        assert!(!full_board.can_place_rectangle(Player::Player1, 0, 0, 1, 1));
        assert!(!full_board.can_place_rectangle(Player::Player2, 14, 39, 1, 1));

        // When simulated, matches terminate within turns limit or 4 consecutive passes
        let res = simulate_match(
            BotType::Fast,
            BotType::Fast,
            Some(42),
            80,
            None,
            None,
            None,
            None,
        );
        assert!(res.turns <= 80);
    }

    #[test]
    fn test_match_termination_score_threshold() {
        // If a player exceeds 300 cells, victory is immediately awarded
        let res = simulate_match(
            BotType::Greedy,
            BotType::Greedy,
            Some(123456789),
            80,
            None,
            None,
            None,
            None,
        );
        assert!(res.winner.is_some());
        assert!(res.turns <= 80);
        // Verify either winner reached > 300 or game ended
        assert!(res.p1_score > 0 && res.p2_score > 0);
    }

    #[test]
    fn test_match_termination_turn_limit() {
        let res = simulate_match(
            BotType::Fast,
            BotType::Fast,
            Some(999),
            15, // max 15 turns
            None,
            None,
            None,
            None,
        );
        assert!(res.turns <= 15);
    }

    #[test]
    fn test_bit_exact_parity_10_control_board_states() {
        let weights = HeuristicWeights::default();
        let matrix = generate_default_v2_matrix();

        // 10 control test cases generated from the authoritative Node.js reference:
        let test_cases = [
            (
                "State 1: Opening P2 Start",
                setup_state_1(),
                Move::new(12, 35, 3, 5),
                1176.2703296703298,
            ),
            (
                "State 2: Opening Turn 4 Expansion",
                setup_state_2(),
                Move::new(9, 36, 3, 4),
                842.6184615384616,
            ),
            (
                "State 3: Midgame Contact",
                setup_state_3(),
                Move::new(6, 17, 4, 3),
                4992.438131868133,
            ),
            (
                "State 4: Loop Enclosure P2",
                setup_state_4(),
                Move::new(14, 32, 1, 6),
                2894.117362637363,
            ),
            (
                "State 5: Loop Denial",
                setup_state_5(),
                Move::new(0, 7, 3, 1),
                10307.922802197803,
            ),
            (
                "State 6: Border Bottleneck",
                setup_state_6(),
                Move::new(0, 22, 3, 4),
                3797.9447252747254,
            ),
            (
                "State 7: Wall Cork Bottom",
                setup_state_7(),
                Move::new(13, 9, 2, 3),
                7149.016153846154,
            ),
            (
                "State 8: Pocket Situation",
                setup_state_8(),
                Move::new(9, 12, 3, 4),
                5948.251538461538,
            ),
            (
                "State 9: Guillotine Cut",
                setup_state_9(),
                Move::new(6, 14, 4, 2),
                1227.5176923076924,
            ),
            (
                "State 10: Endgame Gap Penalty",
                setup_state_10(),
                Move::new(1, 20, 3, 2),
                686.6438461538462,
            ),
        ];

        for (name, board, mv, expected_score) in &test_cases {
            let actual_score = evaluate_move_v2(board, mv, &weights, &matrix);
            let diff = (actual_score - expected_score).abs();
            assert!(
                diff < 1e-5,
                "Parity failure on {}: actual={}, expected={}, diff={}",
                name,
                actual_score,
                expected_score,
                diff
            );
        }
    }

    fn fill_rect(b: &mut Board, r0: usize, c0: usize, h: usize, w: usize, val: u8) {
        for r in r0..r0 + h {
            let offset = r * BOARD_WIDTH;
            for c in c0..c0 + w {
                b.cells[offset + c] = val;
            }
        }
    }

    fn setup_state_1() -> Board {
        let mut b = Board::new();
        fill_rect(&mut b, 0, 0, 3, 4, PLAYER_1);
        b
    }

    fn setup_state_2() -> Board {
        let mut b = Board::new();
        fill_rect(&mut b, 0, 0, 3, 5, PLAYER_1);
        fill_rect(&mut b, 3, 0, 2, 4, PLAYER_1);
        fill_rect(&mut b, 12, 35, 3, 5, PLAYER_2);
        b
    }

    fn setup_state_3() -> Board {
        let mut b = Board::new();
        fill_rect(&mut b, 0, 0, 5, 12, PLAYER_1);
        fill_rect(&mut b, 5, 0, 4, 15, PLAYER_1);
        fill_rect(&mut b, 10, 25, 5, 15, PLAYER_2);
        fill_rect(&mut b, 7, 20, 3, 10, PLAYER_2);
        b
    }

    fn setup_state_4() -> Board {
        let mut b = Board::new();
        fill_rect(&mut b, 0, 0, 5, 8, PLAYER_1);
        fill_rect(&mut b, 10, 30, 2, 10, PLAYER_2);
        fill_rect(&mut b, 12, 30, 3, 2, PLAYER_2);
        fill_rect(&mut b, 12, 38, 3, 2, PLAYER_2);
        b
    }

    fn setup_state_5() -> Board {
        let mut b = Board::new();
        fill_rect(&mut b, 0, 8, 5, 2, PLAYER_1);
        fill_rect(&mut b, 4, 0, 2, 8, PLAYER_1);
        fill_rect(&mut b, 10, 25, 5, 15, PLAYER_2);
        fill_rect(&mut b, 0, 10, 3, 5, PLAYER_2);
        b
    }

    fn setup_state_6() -> Board {
        let mut b = Board::new();
        fill_rect(&mut b, 0, 0, 4, 10, PLAYER_1);
        fill_rect(&mut b, 11, 28, 4, 12, PLAYER_2);
        fill_rect(&mut b, 0, 26, 4, 6, PLAYER_2);
        b
    }

    fn setup_state_7() -> Board {
        let mut b = Board::new();
        fill_rect(&mut b, 0, 0, 5, 6, PLAYER_1);
        fill_rect(&mut b, 5, 3, 4, 5, PLAYER_1);
        fill_rect(&mut b, 9, 6, 3, 4, PLAYER_1);
        fill_rect(&mut b, 12, 20, 3, 20, PLAYER_2);
        fill_rect(&mut b, 12, 12, 3, 8, PLAYER_2);
        b
    }

    fn setup_state_8() -> Board {
        let mut b = Board::new();
        fill_rect(&mut b, 0, 0, 6, 8, PLAYER_1);
        fill_rect(&mut b, 6, 4, 4, 8, PLAYER_1);
        fill_rect(&mut b, 10, 8, 4, 4, PLAYER_1);
        fill_rect(&mut b, 10, 16, 5, 24, PLAYER_2);
        b
    }

    fn setup_state_9() -> Board {
        let mut b = Board::new();
        fill_rect(&mut b, 0, 0, 15, 14, PLAYER_1);
        fill_rect(&mut b, 0, 20, 15, 20, PLAYER_2);
        fill_rect(&mut b, 4, 16, 7, 4, PLAYER_2);
        b
    }

    fn setup_state_10() -> Board {
        let mut b = Board::new();
        for r in 0..15 {
            for c in 0..40 {
                if c < 18 {
                    b.set(r, c, PLAYER_1);
                } else if c > 22 {
                    b.set(r, c, PLAYER_2);
                }
            }
        }
        b
    }
}

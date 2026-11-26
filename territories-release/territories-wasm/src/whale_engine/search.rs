use super::eval::{
    can_close_loop, default_spatial_matrix, evaluate_board_state, generate_legal_moves_for_board,
    has_sufficient_pieces_and_moderate_contact,
};
use super::tt::{TTFlag, TranspositionTable};
use super::zobrist::hash_board;
use super::{
    HeuristicWeights, WhaleBoard, WhaleMove, WhalePlayer, DEFAULT_WHALE_DEPTH, MAX_SEARCH_DEPTH,
};
use crate::nnue::NnueModel;
use crate::types::{GameState, Move, NUM_CELLS, WIDTH, WIN_SCORE};

const MAX_PLY_BRANCHES: usize = 8;
const ASPIRATION_DELTA: f64 = 40.0;
const SINGULAR_MARGIN: f64 = 40.0;

#[derive(Debug, Clone, Copy, Default)]
pub struct WhaleStats {
    pub nodes_searched: usize,
    pub tt_hits: usize,
    pub q_nodes: usize,
    pub depth_reached: u8,
}

pub struct WhaleBot {
    pub max_depth: u8,
    pub tt: TranspositionTable,
    pub killer_moves: [[Option<WhaleMove>; 2]; MAX_SEARCH_DEPTH],
    pub countermoves: [[Option<WhaleMove>; NUM_CELLS]; 2],
    pub history: [[u32; NUM_CELLS]; 2],
    pub stats: WhaleStats,
    pub weights: HeuristicWeights,
    pub matrix: [f64; NUM_CELLS],
}

impl Default for WhaleBot {
    fn default() -> Self {
        Self::new(HeuristicWeights::default(), default_spatial_matrix())
    }
}

impl WhaleBot {
    pub fn new(weights: HeuristicWeights, matrix: [f64; NUM_CELLS]) -> Self {
        Self {
            max_depth: DEFAULT_WHALE_DEPTH,
            tt: TranspositionTable::default(),
            killer_moves: [[None; 2]; MAX_SEARCH_DEPTH],
            countermoves: [[None; NUM_CELLS]; 2],
            history: [[0u32; NUM_CELLS]; 2],
            stats: WhaleStats::default(),
            weights,
            matrix,
        }
    }

    pub fn reset_stats(&mut self) {
        self.stats = WhaleStats::default();
    }

    pub fn select_move_with_score(
        &mut self,
        state: &GameState,
        nnue: &NnueModel,
        d1: u8,
        d2: u8,
    ) -> Option<(Move, f64)> {
        let (board, player) = WhaleBoard::from_game_state(state);
        self.select_move_internal(&board, player, d1, d2, nnue)
            .map(|(mv, score)| (mv.to_move(), score))
    }

    pub fn evaluate_candidates(
        &mut self,
        state: &GameState,
        nnue: &NnueModel,
        candidates: &[Move],
        best_known: Option<(Move, f64)>,
    ) -> Vec<(Move, f64)> {
        let (board, player) = WhaleBoard::from_game_state(state);
        let search_depth = self.max_depth.saturating_sub(1);
        let mut results = Vec::with_capacity(candidates.len());

        for &mv in candidates {
            if let Some((best_mv, best_score)) = best_known {
                if best_mv == mv {
                    results.push((mv, best_score));
                    continue;
                }
            }
            let w_mv = WhaleMove::from_move(mv);
            let mut next_board = board.clone();
            next_board.apply_move_and_loops(player, w_mv);

            let score = if next_board.score(player) >= WIN_SCORE {
                50_000.0
            } else if next_board.score(player.opponent()) >= WIN_SCORE {
                -50_000.0
            } else {
                -self.negamax(
                    &next_board,
                    search_depth,
                    -f64::INFINITY,
                    f64::INFINITY,
                    player.opponent(),
                    1,
                    Some(w_mv),
                    true,
                    nnue,
                )
            };
            results.push((mv, score));
        }

        results
    }

    fn select_move_internal(
        &mut self,
        board: &WhaleBoard,
        player: WhalePlayer,
        d1: u8,
        d2: u8,
        nnue: &NnueModel,
    ) -> Option<(WhaleMove, f64)> {
        self.reset_stats();
        self.tt.new_search();

        let legal_moves = generate_legal_moves_for_board(board, player, d1, d2);
        if legal_moves.is_empty() {
            return None;
        }
        if legal_moves.len() == 1 {
            let mut next_board = board.clone();
            next_board.apply_move_and_loops(player, legal_moves[0]);
            let score = evaluate_board_state(&next_board, player, &self.weights, &self.matrix, nnue);
            return Some((legal_moves[0], score));
        }

        let mut best_move = legal_moves[0];
        let mut best_score = -f64::INFINITY;
        let root_hash = hash_board(board, player);

        for depth in 1..=self.max_depth {
            let (mut alpha, mut beta) =
                if depth >= 2 && best_score.is_finite() && best_score.abs() < 40_000.0 {
                    (best_score - ASPIRATION_DELTA, best_score + ASPIRATION_DELTA)
                } else {
                    (-f64::INFINITY, f64::INFINITY)
                };

            let mut iter_best_move;
            let mut iter_best_score;

            loop {
                let search_alpha = alpha;
                let search_beta = beta;

                let tt_hint = self.tt.probe(root_hash).and_then(|e| e.best_move);
                let mut ordered_moves = legal_moves.clone();
                self.order_moves_fast(&mut ordered_moves, board, player, tt_hint, 0, None);

                let mut current_best_mv = ordered_moves[0];
                let mut current_best_score = -f64::INFINITY;
                let mut current_alpha = search_alpha;

                for (idx, &mv) in ordered_moves.iter().enumerate() {
                    self.stats.nodes_searched += 1;
                    let mut next_board = board.clone();
                    next_board.apply_move_and_loops(player, mv);

                    let score = if idx == 0 {
                        -self.negamax(
                            &next_board,
                            depth - 1,
                            -search_beta,
                            -current_alpha,
                            player.opponent(),
                            1,
                            Some(mv),
                            true,
                            nnue,
                        )
                    } else {
                        let mut s = -self.negamax(
                            &next_board,
                            depth - 1,
                            -current_alpha - 1.0,
                            -current_alpha,
                            player.opponent(),
                            1,
                            Some(mv),
                            true,
                            nnue,
                        );
                        if s > current_alpha && s < search_beta {
                            s = -self.negamax(
                                &next_board,
                                depth - 1,
                                -search_beta,
                                -current_alpha,
                                player.opponent(),
                                1,
                                Some(mv),
                                true,
                                nnue,
                            );
                        }
                        s
                    };

                    if score > current_best_score {
                        current_best_score = score;
                        current_best_mv = mv;
                    }
                    if score > current_alpha {
                        current_alpha = score;
                    }
                    if current_alpha >= search_beta {
                        break;
                    }
                }

                iter_best_move = current_best_mv;
                iter_best_score = current_best_score;

                if depth >= 2 && search_alpha.is_finite() && iter_best_score <= search_alpha {
                    alpha = -f64::INFINITY;
                    beta = search_beta;
                    continue;
                }
                if depth >= 2 && search_beta.is_finite() && iter_best_score >= search_beta {
                    alpha = search_alpha;
                    beta = f64::INFINITY;
                    continue;
                }
                break;
            }

            best_move = iter_best_move;
            best_score = iter_best_score;
            self.stats.depth_reached = depth;

            self.tt.store(
                root_hash,
                depth,
                best_score,
                TTFlag::Exact,
                Some(best_move),
            );
        }

        Some((best_move, best_score))
    }

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    fn try_null_move(
        &mut self,
        board: &WhaleBoard,
        depth: u8,
        beta: f64,
        player: WhalePlayer,
        ply: usize,
        allow_nmp: bool,
        nnue: &NnueModel,
    ) -> Option<f64> {
        if !allow_nmp
            || ply == 0
            || depth < 3
            || board.passes != 0
            || !has_sufficient_pieces_and_moderate_contact(board)
        {
            return None;
        }
        let r_depth = depth.saturating_sub(3);
        let null_score = -self.negamax(
            board,
            r_depth,
            -beta,
            -beta + 1.0,
            player.opponent(),
            ply + 1,
            None,
            false,
            nnue,
        );
        if null_score >= beta {
            Some(beta)
        } else {
            None
        }
    }

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    fn compute_lmr_reduction(
        &self,
        board: &WhaleBoard,
        mv: WhaleMove,
        player: WhalePlayer,
        ply: usize,
        depth: u8,
        idx: usize,
        can_capture: bool,
    ) -> Option<u8> {
        if idx < 3 || depth < 2 {
            return None;
        }
        let is_capture = can_capture && can_close_loop(board, mv, player);
        let is_denial = can_capture && can_close_loop(board, mv, player.opponent());
        let is_killer = ply < MAX_SEARCH_DEPTH
            && (Some(mv) == self.killer_moves[ply][0]
                || Some(mv) == self.killer_moves[ply][1]);
        if !is_capture && !is_denial && !is_killer {
            Some(depth.saturating_sub(2))
        } else {
            None
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn negamax(
        &mut self,
        board: &WhaleBoard,
        depth: u8,
        mut alpha: f64,
        mut beta: f64,
        player: WhalePlayer,
        ply: usize,
        prev_move: Option<WhaleMove>,
        allow_nmp: bool,
        nnue: &NnueModel,
    ) -> f64 {
        self.stats.nodes_searched += 1;

        if board.p1_score >= WIN_SCORE {
            return if player == WhalePlayer::Player1 {
                50_000.0 - (ply as f64)
            } else {
                -50_000.0 + (ply as f64)
            };
        }
        if board.p2_score >= WIN_SCORE {
            return if player == WhalePlayer::Player2 {
                50_000.0 - (ply as f64)
            } else {
                -50_000.0 + (ply as f64)
            };
        }
        if board.passes >= 2 {
            let p_score = board.score(player) as f64;
            let opp_score = board.score(player.opponent()) as f64;
            return (p_score - opp_score) * 1000.0;
        }

        let original_alpha = alpha;
        let hash = hash_board(board, player);
        let mut tt_move = None;
        let mut tt_score = None;
        let mut tt_depth = 0;
        let mut tt_flag = TTFlag::UpperBound;

        if let Some(entry) = self.tt.probe(hash) {
            self.stats.tt_hits += 1;
            tt_move = entry.best_move;
            tt_score = Some(entry.score);
            tt_depth = entry.depth;
            tt_flag = entry.flag;

            if entry.depth >= depth {
                match entry.flag {
                    TTFlag::Exact => return entry.score,
                    TTFlag::LowerBound => {
                        alpha = alpha.max(entry.score);
                        if alpha >= beta {
                            return entry.score;
                        }
                    }
                    TTFlag::UpperBound => {
                        beta = beta.min(entry.score);
                        if alpha >= beta {
                            return entry.score;
                        }
                    }
                }
            }
        }

        if depth == 0 {
            return self.quiescence_search(board, alpha, beta, player, 1, nnue);
        }

        if let Some(nmp_score) = self.try_null_move(board, depth, beta, player, ply, allow_nmp, nnue)
        {
            return nmp_score;
        }

        const REP_D1: u8 = 3;
        const REP_D2: u8 = 4;
        let mut legal_moves = generate_legal_moves_for_board(board, player, REP_D1, REP_D2);

        if legal_moves.is_empty() {
            let mut pass_board = board.clone();
            pass_board.passes += 1;
            return -self.negamax(
                &pass_board,
                depth - 1,
                -beta,
                -alpha,
                player.opponent(),
                ply + 1,
                None,
                true,
                nnue,
            );
        }

        self.order_moves_fast(&mut legal_moves, board, player, tt_move, ply, prev_move);

        let mut singular_extension = 0;
        if depth >= 4
            && ply > 0
            && tt_move.is_some()
            && tt_move == Some(legal_moves[0])
            && tt_depth >= depth - 3
            && tt_flag != TTFlag::UpperBound
        {
            if let Some(s_score) = tt_score {
                if s_score > 50.0 {
                    let singular_beta = s_score - SINGULAR_MARGIN;
                    let alt_depth = depth.saturating_sub(3);
                    let mut is_singular = true;

                    for alt_mv in legal_moves.iter().skip(1).take(2) {
                        let mut alt_board = board.clone();
                        alt_board.apply_move_and_loops(player, *alt_mv);
                        let alt_score = -self.negamax(
                            &alt_board,
                            alt_depth,
                            -singular_beta - 1.0,
                            -singular_beta,
                            player.opponent(),
                            ply + 1,
                            Some(*alt_mv),
                            false,
                            nnue,
                        );
                        if alt_score >= singular_beta {
                            is_singular = false;
                            break;
                        }
                    }

                    if is_singular {
                        singular_extension = 1;
                    }
                }
            }
        }

        let branch_limit = legal_moves.len().min(MAX_PLY_BRANCHES);
        let can_capture = !board.p1_bb.is_empty() && !board.p2_bb.is_empty();
        let mut best_score = -f64::INFINITY;
        let mut best_move = None;

        for (idx, &mv) in legal_moves.iter().take(branch_limit).enumerate() {
            let mut next_board = board.clone();
            next_board.apply_move_and_loops(player, mv);

            let score = if idx == 0 {
                let search_depth = (depth - 1) + singular_extension;
                -self.negamax(
                    &next_board,
                    search_depth,
                    -beta,
                    -alpha,
                    player.opponent(),
                    ply + 1,
                    Some(mv),
                    true,
                    nnue,
                )
            } else {
                let mut s;
                if let Some(reduced_depth) =
                    self.compute_lmr_reduction(board, mv, player, ply, depth, idx, can_capture)
                {
                    s = -self.negamax(
                        &next_board,
                        reduced_depth,
                        -alpha - 1.0,
                        -alpha,
                        player.opponent(),
                        ply + 1,
                        Some(mv),
                        true,
                        nnue,
                    );
                    if s > alpha {
                        s = -self.negamax(
                            &next_board,
                            depth - 1,
                            -alpha - 1.0,
                            -alpha,
                            player.opponent(),
                            ply + 1,
                            Some(mv),
                            true,
                            nnue,
                        );
                        if s > alpha && s < beta {
                            s = -self.negamax(
                                &next_board,
                                depth - 1,
                                -beta,
                                -alpha,
                                player.opponent(),
                                ply + 1,
                                Some(mv),
                                true,
                                nnue,
                            );
                        }
                    }
                } else {
                    s = -self.negamax(
                        &next_board,
                        depth - 1,
                        -alpha - 1.0,
                        -alpha,
                        player.opponent(),
                        ply + 1,
                        Some(mv),
                        true,
                        nnue,
                    );
                    if s > alpha && s < beta {
                        s = -self.negamax(
                            &next_board,
                            depth - 1,
                            -beta,
                            -alpha,
                            player.opponent(),
                            ply + 1,
                            Some(mv),
                            true,
                            nnue,
                        );
                    }
                }
                s
            };

            if score > best_score {
                best_score = score;
                best_move = Some(mv);
            }
            if score > alpha {
                alpha = score;
            }
            if alpha >= beta {
                let p_idx = player.index();
                if ply < MAX_SEARCH_DEPTH && self.killer_moves[ply][0] != Some(mv) {
                    self.killer_moves[ply][1] = self.killer_moves[ply][0];
                    self.killer_moves[ply][0] = Some(mv);
                }
                if let Some(prev) = prev_move {
                    let prev_cell = (prev.r as usize) * WIDTH + (prev.c as usize);
                    if prev_cell < NUM_CELLS {
                        self.countermoves[p_idx][prev_cell] = Some(mv);
                    }
                }
                let cell_idx = (mv.r as usize) * WIDTH + (mv.c as usize);
                if cell_idx < NUM_CELLS {
                    self.history[p_idx][cell_idx] = self.history[p_idx][cell_idx]
                        .saturating_add((depth as u32) * (depth as u32));
                }
                break;
            }
        }

        let flag = if best_score <= original_alpha {
            TTFlag::UpperBound
        } else if best_score >= beta {
            TTFlag::LowerBound
        } else {
            TTFlag::Exact
        };
        self.tt.store(hash, depth, best_score, flag, best_move);
        best_score
    }

    fn quiescence_search(
        &mut self,
        board: &WhaleBoard,
        mut alpha: f64,
        beta: f64,
        player: WhalePlayer,
        q_depth: u8,
        nnue: &NnueModel,
    ) -> f64 {
        self.stats.q_nodes += 1;

        let stand_pat = evaluate_board_state(board, player, &self.weights, &self.matrix, nnue);
        if stand_pat >= beta {
            return beta;
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }
        if q_depth == 0 || board.p1_bb.is_empty() || board.p2_bb.is_empty() {
            return alpha;
        }

        let moves = generate_legal_moves_for_board(board, player, 3, 3);
        let mut tactical_moves = Vec::with_capacity(2);
        for mv in moves {
            if can_close_loop(board, mv, player) {
                tactical_moves.push(mv);
                if tactical_moves.len() >= 2 {
                    break;
                }
            }
        }
        if tactical_moves.is_empty() {
            return alpha;
        }

        for mv in tactical_moves {
            let mut next_board = board.clone();
            next_board.apply_move_and_loops(player, mv);
            let score = -self.quiescence_search(
                &next_board,
                -beta,
                -alpha,
                player.opponent(),
                q_depth - 1,
                nnue,
            );
            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }
        alpha
    }

    fn order_moves_fast(
        &self,
        moves: &mut [WhaleMove],
        board: &WhaleBoard,
        player: WhalePlayer,
        tt_move: Option<WhaleMove>,
        ply: usize,
        prev_move: Option<WhaleMove>,
    ) {
        let p_idx = player.index();
        let km0 = if ply < MAX_SEARCH_DEPTH {
            self.killer_moves[ply][0]
        } else {
            None
        };
        let km1 = if ply < MAX_SEARCH_DEPTH {
            self.killer_moves[ply][1]
        } else {
            None
        };
        let countermove = prev_move.and_then(|prev| {
            let prev_cell = (prev.r as usize) * WIDTH + (prev.c as usize);
            if prev_cell < NUM_CELLS {
                self.countermoves[p_idx][prev_cell]
            } else {
                None
            }
        });
        let can_capture_check = !board.p1_bb.is_empty() && !board.p2_bb.is_empty();

        let mut scored: Vec<(WhaleMove, f64)> = moves
            .iter()
            .map(|&mv| {
                let mut score = 0.0;
                if Some(mv) == tt_move {
                    score += 10_000_000.0;
                }
                if Some(mv) == km0 {
                    score += 900_000.0;
                } else if Some(mv) == km1 {
                    score += 800_000.0;
                }
                if can_capture_check {
                    if can_close_loop(board, mv, player) {
                        score += 500_000.0;
                    } else if can_close_loop(board, mv, player.opponent()) {
                        score += 350_000.0;
                    }
                }
                if Some(mv) == countermove {
                    score += 200_000.0;
                }
                let cell_idx = (mv.r as usize) * WIDTH + (mv.c as usize);
                if cell_idx < NUM_CELLS {
                    score += (self.history[p_idx][cell_idx] as f64).min(100_000.0);
                }

                let area = (mv.h as f64) * (mv.w as f64);
                let min_dim = mv.h.min(mv.w);
                let compact_bonus = if min_dim >= 2 { 20.0 } else { 0.0 };
                let advance = match player {
                    WhalePlayer::Player1 => mv.c as f64,
                    WhalePlayer::Player2 => {
                        (39usize.saturating_sub((mv.c + mv.w - 1) as usize)) as f64
                    }
                };
                score += area * 100.0 + compact_bonus + advance * 15.0;
                (mv, score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        for (i, (mv, _)) in scored.into_iter().enumerate() {
            moves[i] = mv;
        }
    }
}

use super::*;
use crate::bitboard::BOARD_MASK;

fn reference_extract_features(state: &Afterstate) -> TopoFeatures {
    let occupied = state.my_bb | state.opp_bb;
    let empty = (!occupied) & BOARD_MASK;

    let mut border_connected = empty & BORDER_MASK;
    loop {
        let dilated = border_connected.shift_left()
            | border_connected.shift_right()
            | border_connected.shift_up()
            | border_connected.shift_down();
        let next = (border_connected | dilated) & empty;
        if next == border_connected {
            break;
        }
        border_connected = next;
    }

    let cauldrons = empty & (!border_connected);
    let mut cauldron_size_map = [0.0f32; NUM_CELLS];
    let mut max_my_cauldron = 0u16;
    let mut max_opp_cauldron = 0u16;

    let my_dilated = state.my_bb.shift_left()
        | state.my_bb.shift_right()
        | state.my_bb.shift_up()
        | state.my_bb.shift_down();
    let opp_dilated = state.opp_bb.shift_left()
        | state.opp_bb.shift_right()
        | state.opp_bb.shift_up()
        | state.opp_bb.shift_down();

    let mut remaining = cauldrons;
    while !remaining.is_empty() {
        let mut seed_idx = None;
        for i in 0..10 {
            if remaining.0[i] != 0 {
                let bit = remaining.0[i].trailing_zeros() as usize;
                let idx = i * 64 + bit;
                if idx < NUM_CELLS {
                    seed_idx = Some(idx);
                }
                break;
            }
        }

        let idx = match seed_idx {
            Some(i) => i,
            None => break,
        };

        let mut comp = Bitboard::new();
        comp.set(idx % WIDTH, idx / WIDTH);
        loop {
            let dilated = comp.shift_left() | comp.shift_right() | comp.shift_up() | comp.shift_down();
            let next = (comp | dilated) & cauldrons;
            if next == comp {
                break;
            }
            comp = next;
        }

        let size = comp.count_ones();
        remaining = remaining & (!comp);

        let touches_my = !(comp & my_dilated).is_empty();
        let touches_opp = !(comp & opp_dilated).is_empty();

        if touches_my && size > max_my_cauldron {
            max_my_cauldron = size;
        }
        if touches_opp && size > max_opp_cauldron {
            max_opp_cauldron = size;
        }

        let norm_size = (size as f32) / 300.0;
        for i in 0..10 {
            let mut w = comp.0[i];
            while w != 0 {
                let bit = w.trailing_zeros() as usize;
                let c_idx = i * 64 + bit;
                if c_idx < NUM_CELLS {
                    cauldron_size_map[c_idx] = norm_size;
                }
                w &= w - 1;
            }
        }
    }

    let mut p_seal_my = [0.0f32; NUM_CELLS];
    let mut p_seal_opp = [0.0f32; NUM_CELLS];
    let cauldron_border_my = (cauldrons.shift_left()
        | cauldrons.shift_right()
        | cauldrons.shift_up()
        | cauldrons.shift_down())
        & my_dilated
        & empty;
    let cauldron_border_opp = (cauldrons.shift_left()
        | cauldrons.shift_right()
        | cauldrons.shift_up()
        | cauldrons.shift_down())
        & opp_dilated
        & empty;

    for i in 0..10 {
        let mut w = cauldron_border_my.0[i];
        while w != 0 {
            let bit = w.trailing_zeros() as usize;
            let c_idx = i * 64 + bit;
            if c_idx < NUM_CELLS {
                p_seal_my[c_idx] = 0.55;
            }
            w &= w - 1;
        }
    }
    for i in 0..10 {
        let mut w = cauldron_border_opp.0[i];
        while w != 0 {
            let bit = w.trailing_zeros() as usize;
            let c_idx = i * 64 + bit;
            if c_idx < NUM_CELLS {
                p_seal_opp[c_idx] = 0.55;
            }
            w &= w - 1;
        }
    }

    let my_state = GameState {
        my_bb: state.my_bb,
        opp_bb: state.opp_bb,
        my_score: state.my_score,
        opp_score: state.opp_score,
        turn: state.turn,
    };
    let opp_state = GameState {
        my_bb: state.opp_bb,
        opp_bb: state.my_bb,
        my_score: state.opp_score,
        opp_score: state.my_score,
        turn: state.turn + 1,
    };

    let mut my_legal_count = 0;
    let mut opp_legal_count = 0;
    for &(d1, d2, _) in &DICE_OUTCOMES {
        let roll = DiceRoll(d1, d2);
        if crate::bitboard::has_legal_move(&my_state, roll) {
            my_legal_count += 1;
        }
        if crate::bitboard::has_legal_move(&opp_state, roll) {
            opp_legal_count += 1;
        }
    }

    let my_elasticity = (my_legal_count as f32) / 21.0;
    let opp_elasticity = (opp_legal_count as f32) / 21.0;

    TopoFeatures {
        cauldron_size_map,
        p_seal_my,
        p_seal_opp,
        my_elasticity,
        opp_elasticity,
        max_my_cauldron,
        max_opp_cauldron,
    }
}

fn reference_encode_tensor(state: &Afterstate, feat: &TopoFeatures) -> [f32; TENSOR_SIZE] {
    let mut tensor = [0.0f32; TENSOR_SIZE];
    let plane_size = HEIGHT * WIDTH;

    let occupied = state.my_bb | state.opp_bb;
    let empty = (!occupied) & BOARD_MASK;
    let my_dilated = state.my_bb.shift_left()
        | state.my_bb.shift_right()
        | state.my_bb.shift_up()
        | state.my_bb.shift_down();
    let legal_perimeter = my_dilated & empty;

    let score_plane = (state.my_score as f32) / 300.0;
    let elasticity_plane = feat.my_elasticity;

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let idx = y * WIDTH + x;
            if state.my_bb.get(x, y) {
                tensor[0 * plane_size + idx] = 1.0;
            }
            if state.opp_bb.get(x, y) {
                tensor[1 * plane_size + idx] = 1.0;
            }
            if legal_perimeter.get(x, y) {
                tensor[2 * plane_size + idx] = 1.0;
            }
            tensor[3 * plane_size + idx] = feat.cauldron_size_map[idx];
            tensor[4 * plane_size + idx] = feat.p_seal_my[idx];
            tensor[5 * plane_size + idx] = feat.p_seal_opp[idx];
            tensor[6 * plane_size + idx] = score_plane;
            tensor[7 * plane_size + idx] = elasticity_plane;
        }
    }
    tensor
}

#[test]
fn test_equivalence_on_diverse_boards() {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    let mut rng = StdRng::seed_from_u64(424242);

    for board_idx in 0..25 {
        let mut my_bb = Bitboard::new();
        let mut opp_bb = Bitboard::new();

        match board_idx {
            0 => {}
            1 => {
                for x in 10..=15 {
                    my_bb.set(x, 8);
                    my_bb.set(x, 12);
                }
                for y in 8..=12 {
                    my_bb.set(10, y);
                    my_bb.set(15, y);
                }
            }
            2 => {
                for x in 5..=8 {
                    my_bb.set(x, 3);
                    my_bb.set(x, 6);
                }
                for y in 3..=6 {
                    my_bb.set(5, y);
                    my_bb.set(8, y);
                }
                for x in 20..=24 {
                    opp_bb.set(x, 10);
                    opp_bb.set(x, 13);
                }
                for y in 10..=13 {
                    opp_bb.set(20, y);
                    opp_bb.set(24, y);
                }
            }
            3 => {
                for y in 0..15 {
                    my_bb.set(15, y);
                }
            }
            _ => {
                let num_rects = rng.gen_range(2..8);
                for _ in 0..num_rects {
                    let is_my = rng.gen_bool(0.5);
                    let w = rng.gen_range(2..6);
                    let h = rng.gen_range(2..6);
                    let rx = rng.gen_range(0..WIDTH - w);
                    let ry = rng.gen_range(0..HEIGHT - h);
                    for y in ry..ry + h {
                        for x in rx..rx + w {
                            if is_my {
                                if !opp_bb.get(x, y) {
                                    my_bb.set(x, y);
                                }
                            } else {
                                if !my_bb.get(x, y) {
                                    opp_bb.set(x, y);
                                }
                            }
                        }
                    }
                }
            }
        }

        let state = Afterstate {
            my_bb,
            opp_bb,
            my_score: (my_bb.count_ones() * 5) as u16,
            opp_score: (opp_bb.count_ones() * 5) as u16,
            turn: board_idx as u16,
        };

        let fast_feat = extract_features(&state);
        let ref_feat = reference_extract_features(&state);

        assert_eq!(
            fast_feat.max_my_cauldron, ref_feat.max_my_cauldron,
            "Board {}: max_my_cauldron mismatch (fast: {}, ref: {})",
            board_idx, fast_feat.max_my_cauldron, ref_feat.max_my_cauldron
        );
        assert_eq!(
            fast_feat.max_opp_cauldron, ref_feat.max_opp_cauldron,
            "Board {}: max_opp_cauldron mismatch (fast: {}, ref: {})",
            board_idx, fast_feat.max_opp_cauldron, ref_feat.max_opp_cauldron
        );
        assert!(
            (fast_feat.my_elasticity - ref_feat.my_elasticity).abs() < 1e-6,
            "Board {}: my_elasticity mismatch",
            board_idx
        );
        assert!(
            (fast_feat.opp_elasticity - ref_feat.opp_elasticity).abs() < 1e-6,
            "Board {}: opp_elasticity mismatch",
            board_idx
        );

        for i in 0..NUM_CELLS {
            assert_eq!(
                fast_feat.cauldron_size_map[i], ref_feat.cauldron_size_map[i],
                "Board {} cell {}: cauldron_size_map mismatch",
                board_idx, i
            );
            assert_eq!(
                fast_feat.p_seal_my[i], ref_feat.p_seal_my[i],
                "Board {} cell {}: p_seal_my mismatch",
                board_idx, i
            );
            assert_eq!(
                fast_feat.p_seal_opp[i], ref_feat.p_seal_opp[i],
                "Board {} cell {}: p_seal_opp mismatch",
                board_idx, i
            );
        }

        let fast_tensor = encode_tensor(&state, &fast_feat);
        let ref_tensor = reference_encode_tensor(&state, &ref_feat);

        for i in 0..TENSOR_SIZE {
            assert_eq!(
                fast_tensor[i], ref_tensor[i],
                "Board {} tensor index {}: encode_tensor mismatch (fast: {}, ref: {})",
                board_idx, i, fast_tensor[i], ref_tensor[i]
            );
        }
    }
}

#[test]
fn test_multi_pocket_corridor_and_enemy_island_edges() {
    let mut p1_bb = Bitboard::new();
    p1_bb.set(7, 0);
    p1_bb.set(7, 1);
    p1_bb.set(8, 1);
    p1_bb.set(9, 1);
    p1_bb.set(6, 3);
    p1_bb.set(6, 4);
    p1_bb.set(6, 5);
    for x in 7..=9 {
        p1_bb.set(x, 3);
        p1_bb.set(x, 5);
    }
    let mut opp_bb = Bitboard::new();
    opp_bb.set(39, 14);
    for x in 23..=27 {
        p1_bb.set(x, 8);
        p1_bb.set(x, 12);
    }
    for y in 8..=12 {
        p1_bb.set(23, y);
        p1_bb.set(27, y);
    }
    opp_bb.set(25, 10);

    let state = GameState {
        my_bb: p1_bb,
        opp_bb,
        my_score: p1_bb.count_ones(),
        opp_score: 2,
        turn: 4,
    };

    let after = crate::bitboard::apply_move(&state, crate::types::Move { x: 10, y: 0, w: 1, h: 5 });
    assert_eq!(after.my_score, state.my_score + 5 + 2 + 3);
    assert!(after.my_bb.get(8, 0) && after.my_bb.get(9, 0));
    assert!(after.my_bb.get(7, 4) && after.my_bb.get(8, 4) && after.my_bb.get(9, 4));
    assert!(after.opp_bb.get(25, 10));
    assert!(!after.my_bb.get(24, 9) && !after.opp_bb.get(24, 9));
}

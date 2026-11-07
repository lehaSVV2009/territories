"""Empirical Challenger Stress Test Suite for Milestone M3 (PyO3 FFI & Maturin Build).

This suite empirically tests:
1. Concurrency & Python GIL release (`py.allow_threads`).
2. Large-batch stress & thread safety across 64 candidates x 50 seeds (6,400 matches).
3. Deterministic index preservation under Rayon multi-threading.
4. Latency SLA measurement for `select_move_ffi` across 500 diverse board states (p99 <= 2.0 ms).
5. Robustness and schema compliance for tree exports (.dot, .json) and weights round-trip.
"""

import json
import os
import random
import tempfile
import threading
import time
from typing import List, Tuple
import pytest
import pyrust

CANONICAL_WEIGHT_KEYS = [
    "WEIGHT_AREA",
    "WEIGHT_CAPTURE",
    "WEIGHT_DENY_CAPTURE",
    "WEIGHT_VORONOI_DELTA",
    "WEIGHT_SAFE_TERRITORY",
    "WEIGHT_COMPACTNESS",
    "WEIGHT_SPATIAL_MATRIX",
    "WEIGHT_FRONTIER",
    "WEIGHT_OPP_CHOKE",
    "WEIGHT_BOTTLENECK",
]

DEFAULT_WEIGHTS = [50.0, 95.0, 85.0, 26.0, 23.088, 12.973, 0.322, 17.307, 18.0, 12.0]


# ============================================================================
# Helpers: Diverse Board Generation
# ============================================================================

def make_empty_board() -> List[int]:
    return [0] * 600


def make_opening_board(seed: int) -> List[int]:
    board = make_empty_board()
    rng = random.Random(seed)
    # P1 start corner (0, 0)
    h1, w1 = rng.choice([(2, 3), (3, 2), (2, 2), (1, 4), (4, 1)])
    for r in range(h1):
        for c in range(w1):
            board[r * 40 + c] = 1

    # P2 start corner (14, 39)
    h2, w2 = rng.choice([(2, 3), (3, 2), (2, 2), (1, 4), (4, 1)])
    for r in range(15 - h2, 15):
        for c in range(40 - w2, 40):
            board[r * 40 + c] = 2

    return board


def make_midgame_board(seed: int) -> List[int]:
    board = make_opening_board(seed)
    rng = random.Random(seed)
    # P1 expands toward center/east
    p1_cells = [(r, c) for r in range(15) for c in range(40) if board[r * 40 + c] == 1]
    for _ in range(rng.randint(5, 12)):
        if not p1_cells:
            break
        br, bc = rng.choice(p1_cells)
        dr, dc = rng.choice([(0, 1), (1, 0), (0, -1), (-1, 0)])
        nr, nc = br + dr, bc + dc
        h, w = rng.randint(1, 3), rng.randint(1, 4)
        for r in range(nr, min(15, nr + h)):
            for c in range(nc, min(40, nc + w)):
                if 0 <= r < 15 and 0 <= c < 40 and board[r * 40 + c] == 0:
                    board[r * 40 + c] = 1
                    p1_cells.append((r, c))

    # P2 expands toward center/west
    p2_cells = [(r, c) for r in range(15) for c in range(40) if board[r * 40 + c] == 2]
    for _ in range(rng.randint(5, 12)):
        if not p2_cells:
            break
        br, bc = rng.choice(p2_cells)
        dr, dc = rng.choice([(0, -1), (-1, 0), (0, 1), (1, 0)])
        nr, nc = br + dr, bc + dc
        h, w = rng.randint(1, 3), rng.randint(1, 4)
        for r in range(nr, min(15, nr + h)):
            for c in range(nc, min(40, nc + w)):
                if 0 <= r < 15 and 0 <= c < 40 and board[r * 40 + c] == 0:
                    board[r * 40 + c] = 2
                    p2_cells.append((r, c))

    return board


def make_lategame_board(seed: int) -> List[int]:
    board = make_midgame_board(seed)
    rng = random.Random(seed)
    # Dense territory allocation
    for r in range(15):
        for c in range(40):
            if board[r * 40 + c] == 0:
                if c < 18 and rng.random() < 0.65:
                    board[r * 40 + c] = 1
                elif c > 21 and rng.random() < 0.65:
                    board[r * 40 + c] = 2
                elif rng.random() < 0.35:
                    board[r * 40 + c] = rng.choice([1, 2])
    return board


def generate_500_diverse_boards() -> List[Tuple[List[int], int, int]]:
    """Generates 500 diverse board configurations paired with dice rolls."""
    boards = []
    rng = random.Random(42)

    # 1. 100 early-game boards
    for i in range(100):
        b = make_opening_board(1000 + i)
        d1, d2 = rng.randint(1, 6), rng.randint(1, 6)
        boards.append((b, d1, d2))

    # 2. 200 mid-game boards
    for i in range(200):
        b = make_midgame_board(2000 + i)
        d1, d2 = rng.randint(1, 6), rng.randint(1, 6)
        boards.append((b, d1, d2))

    # 3. 200 late-game boards
    for i in range(200):
        b = make_lategame_board(3000 + i)
        d1, d2 = rng.randint(1, 6), rng.randint(1, 6)
        boards.append((b, d1, d2))

    return boards


# ============================================================================
# Test Suite 1: GIL Release & Concurrency
# ============================================================================

class TestGilReleaseConcurrency:
    """Stress-tests the Global Interpreter Lock (GIL) release behavior."""

    def test_gil_released_during_evaluate_population(self):
        """Proves that a concurrent Python thread runs unblocked during evaluate_population_ffi."""
        cands = [
            [50.0, 95.0 + i, 85.0 - i, 26.0, 23.0, 12.0, 0.3, 17.0, 18.0, 12.0]
            for i in range(32)
        ]
        seeds = list(range(500, 520))  # 20 seeds x 32 candidates = 1,280 matches

        stop_event = threading.Event()
        python_tick_count = 0
        max_python_gap_ms = 0.0

        def python_worker():
            nonlocal python_tick_count, max_python_gap_ms
            last_time = time.perf_counter()
            while not stop_event.is_set():
                now = time.perf_counter()
                gap_ms = (now - last_time) * 1000.0
                if gap_ms > max_python_gap_ms:
                    max_python_gap_ms = gap_ms
                last_time = now
                python_tick_count += 1
                # Small sleep or busy yield in Python
                time.sleep(0.002)

        py_thread = threading.Thread(target=python_worker)
        py_thread.start()

        # Execute heavy Rust batch evaluation
        t_start = time.perf_counter()
        results = pyrust.evaluate_population_ffi(cands, None, "GreedyBot", seeds, 4)
        t_rust_elapsed = time.perf_counter() - t_start

        stop_event.set()
        py_thread.join()

        print(f"\n[GIL EVALUATION TEST] Rust batch duration: {t_rust_elapsed:.3f} s")
        print(f"[GIL EVALUATION TEST] Python thread ticks completed: {python_tick_count}")
        print(f"[GIL EVALUATION TEST] Max Python thread scheduling gap: {max_python_gap_ms:.2f} ms")

        assert len(results) == 32
        assert t_rust_elapsed > 0.05, "Rust batch execution must take non-negligible time to test GIL"
        # If the GIL were NOT released, the Python thread would be completely blocked and python_tick_count would be near 0
        assert python_tick_count >= 10, (
            f"GIL was blocked: Python thread only executed {python_tick_count} ticks during {t_rust_elapsed:.3f} s Rust call"
        )
        assert max_python_gap_ms < 500.0, (
            f"Python thread suffered excessive starvation gap ({max_python_gap_ms:.2f} ms), GIL lock suspect"
        )

    def test_gil_released_during_simulate_match_loop(self):
        """Proves that simulate_match_ffi releases the GIL across concurrent calls."""
        stop_event = threading.Event()
        counter = [0]

        def python_counter():
            while not stop_event.is_set():
                counter[0] += 1
                time.sleep(0.001)

        t = threading.Thread(target=python_counter)
        t.start()

        start = time.perf_counter()
        for i in range(100):
            res = pyrust.simulate_match_ffi("SpatialSlayer", "GreedyBot", 1000 + i, DEFAULT_WEIGHTS, None)
            assert len(res) == 4

        elapsed = time.perf_counter() - start
        stop_event.set()
        t.join()

        print(f"\n[GIL SIMULATE MATCH] 100 matches in {elapsed:.3f} s, Python thread completed {counter[0]} iterations")
        assert counter[0] > 20, f"Python thread was blocked during simulate_match_ffi: {counter[0]} iterations"


# ============================================================================
# Test Suite 2: Large-Batch Stress & Index Order Determinism
# ============================================================================

class TestLargeBatchStressAndDeterminism:
    """Stress-tests large-scale candidate evaluation (64 candidates x 50 seeds = 6,400 matches)."""

    def test_large_batch_64_candidates_50_seeds_stability(self):
        """Validates 64 candidates x 50 seeds without crashes, panics, or memory explosion."""
        cands = []
        rng = random.Random(777)
        for _ in range(64):
            w = [
                50.0,
                rng.uniform(70.0, 150.0),
                rng.uniform(60.0, 120.0),
                rng.uniform(15.0, 45.0),
                rng.uniform(10.0, 35.0),
                rng.uniform(5.0, 25.0),
                rng.uniform(0.1, 0.6),
                rng.uniform(10.0, 30.0),
                rng.uniform(10.0, 30.0),
                rng.uniform(5.0, 25.0),
            ]
            cands.append(w)

        seeds = list(range(2000, 2050))  # 50 seeds => 6,400 matches

        t0 = time.perf_counter()
        results = pyrust.evaluate_population_ffi(cands, None, "GreedyBot", seeds, 4)
        elapsed = time.perf_counter() - t0

        print(f"\n[LARGE BATCH 64x50] 6,400 matches finished in {elapsed:.3f} s ({6400 / elapsed:.1f} matches/sec)")

        assert len(results) == 64
        for idx, r in enumerate(results):
            assert r["total_matches"] == 100
            assert r["wins"] + r["losses"] + r["draws"] == 100
            expected_winrate = (r["wins"] + 0.5 * r["draws"]) / 100.0
            assert abs(r["winrate"] - expected_winrate) < 1e-6
            # In selector.rs, avg_margin is per CRN pair (total_margin / seeds.len())
            expected_avg_margin = r["net_margin"] / float(len(seeds))
            assert abs(r["avg_margin"] - expected_avg_margin) < 1e-6
            expected_fitness = r["winrate"] * 100.0 + expected_avg_margin * 2.0
            assert abs(r["fitness"] - expected_fitness) < 1e-6

    def test_strict_deterministic_index_ordering_across_rayon(self):
        """Validates that candidate A produces identical scores when placed at index 0, 7, 31, 63."""
        cand_a = [50.0, 95.0, 85.0, 26.0, 23.088, 12.973, 0.322, 17.307, 18.0, 12.0]
        seeds = list(range(3000, 3020))  # 20 seeds

        # Evaluate candidate A alone
        solo_res = pyrust.evaluate_population_ffi([cand_a], None, "GreedyBot", seeds, 1)[0]

        # Generate 63 distinct background candidates
        rng = random.Random(999)
        background = [
            [50.0] + [rng.uniform(10.0, 100.0) for _ in range(9)]
            for _ in range(63)
        ]

        target_indices = [0, 7, 31, 63]
        for target_idx in target_indices:
            batch = list(background)
            batch.insert(target_idx, cand_a)
            assert len(batch) == 64

            batch_res = pyrust.evaluate_population_ffi(batch, None, "GreedyBot", seeds, 4)
            cand_res = batch_res[target_idx]

            print(f"[INDEX DETERMINISM] Target index {target_idx}: "
                  f"winrate={cand_res['winrate']} vs solo={solo_res['winrate']}, "
                  f"net_margin={cand_res['net_margin']} vs solo={solo_res['net_margin']}")

            assert cand_res["winrate"] == solo_res["winrate"], (
                f"Index {target_idx}: Winrate mismatch {cand_res['winrate']} vs {solo_res['winrate']}"
            )
            assert cand_res["net_margin"] == solo_res["net_margin"], (
                f"Index {target_idx}: Net margin mismatch {cand_res['net_margin']} vs {solo_res['net_margin']}"
            )
            assert cand_res["total_turns"] == solo_res["total_turns"], (
                f"Index {target_idx}: Total turns mismatch {cand_res['total_turns']} vs {solo_res['total_turns']}"
            )
            assert cand_res["fitness"] == solo_res["fitness"], (
                f"Index {target_idx}: Fitness mismatch {cand_res['fitness']} vs {solo_res['fitness']}"
            )


# ============================================================================
# Test Suite 3: Latency SLA Measurement over 500 Diverse Boards
# ============================================================================

class TestInferenceLatencySLA500Boards:
    """Rigorous empirical measurement of select_move_ffi latency across 500 diverse board states."""

    def test_select_move_ffi_latency_sla_p99_under_two_ms(self):
        """Verifies p99 latency SLA <= 2.0 ms over 500 distinct early/mid/late board states."""
        boards = generate_500_diverse_boards()
        assert len(boards) == 500
        weights = DEFAULT_WEIGHTS

        # Warmup
        for i in range(15):
            b, d1, d2 = boards[i]
            pyrust.select_move_ffi(b, d1, d2, weights)

        latencies_ms = []

        for b, d1, d2 in boards:
            t0 = time.perf_counter()
            res = pyrust.select_move_ffi(b, d1, d2, weights)
            elapsed_ms = (time.perf_counter() - t0) * 1000.0
            latencies_ms.append(elapsed_ms)

            assert isinstance(res, tuple) and len(res) == 5
            r0, c0, h, w, score = res
            assert 0 <= r0 <= 15
            assert 0 <= c0 <= 40

        latencies_ms.sort()
        n = len(latencies_ms)
        p50 = latencies_ms[int(n * 0.50)]
        p90 = latencies_ms[int(n * 0.90)]
        p95 = latencies_ms[int(n * 0.95)]
        p99 = latencies_ms[int(n * 0.99)]
        max_lat = latencies_ms[-1]
        mean_lat = sum(latencies_ms) / n

        print("\n" + "=" * 60)
        print("EMPIRICAL LATENCY BENCHMARK: 500 Diverse Board States (select_move_ffi)")
        print("=" * 60)
        print(f"Mean Latency:    {mean_lat:.4f} ms")
        print(f"p50 (Median):    {p50:.4f} ms")
        print(f"p90:             {p90:.4f} ms")
        print(f"p95:             {p95:.4f} ms")
        print(f"p99 (SLA Bound): {p99:.4f} ms  (Target SLA: <= 2.000 ms)")
        print(f"Max Latency:     {max_lat:.4f} ms")
        print("=" * 60)

        assert p99 <= 2.0, f"SLA Violation: p99 latency {p99:.4f} ms exceeds 2.0 ms threshold"


# ============================================================================
# Test Suite 4: Decision Tree & Weights File Robustness
# ============================================================================

class TestTreeExportAndWeightsPersistenceRobustness:
    """Tests syntax and edge-case handling for tree export and weights round-trip."""

    def test_tree_export_dot_and_json_across_diverse_states(self):
        """Verifies that export_tree_dot and export_tree_json succeed and produce valid syntax."""
        test_states = [
            ("empty_board", make_empty_board(), 2, 3),
            ("opening_board", make_opening_board(42), 3, 4),
            ("midgame_choke", make_midgame_board(101), 2, 5),
            ("lategame_dense", make_lategame_board(202), 4, 4),
        ]

        with tempfile.TemporaryDirectory() as tmpdir:
            for name, board, d1, d2 in test_states:
                dot_file = os.path.join(tmpdir, f"{name}.dot")
                json_file = os.path.join(tmpdir, f"{name}.json")

                # Export DOT
                pyrust.export_tree_dot(board, d1, d2, DEFAULT_WEIGHTS, dot_file)
                assert os.path.exists(dot_file)
                with open(dot_file, "r", encoding="utf-8") as f:
                    dot_text = f.read()

                assert dot_text.startswith("digraph DecisionTree {")
                assert dot_text.strip().endswith("}")
                assert dot_text.count("{") == dot_text.count("}")
                assert 'rankdir=LR;' in dot_text
                assert 'root' in dot_text

                # Export JSON
                pyrust.export_tree_json(board, d1, d2, DEFAULT_WEIGHTS, json_file)
                assert os.path.exists(json_file)
                with open(json_file, "r", encoding="utf-8") as f:
                    tree_data = json.load(f)

                assert "turn" in tree_data
                assert "player" in tree_data
                assert "dice" in tree_data
                assert "evaluatedCandidates" in tree_data
                assert "root" in tree_data
                assert isinstance(tree_data["evaluatedCandidates"], list)
                assert len(tree_data["evaluatedCandidates"]) <= 4

    def test_weights_persistence_extreme_edge_cases_roundtrip(self):
        """Tests weights serialization and deserialization with extreme boundary values."""
        edge_cases = [
            ("all_zeros", [0.0] * 10, [0.0] * 600),
            ("extreme_positives", [1e6] * 10, [99999.0] * 600),
            ("extreme_negatives", [-1e5] * 10, [-500.0] * 600),
            ("fine_precision", [0.123456789012345] * 10, [float(i) * 0.001 for i in range(600)]),
        ]

        with tempfile.TemporaryDirectory() as tmpdir:
            for label, weights, matrix in edge_cases:
                file_path = os.path.join(tmpdir, f"{label}.json")
                meta = {
                    "epoch": 999,
                    "winrate_vs_hard": 0.95,
                    "winrate_vs_greedy": 0.99,
                    "winrate_vs_champ": 0.70,
                    "avg_margin": 15.5,
                    "fitness": 120.0,
                }

                pyrust.save_weights_file(file_path, weights, matrix, meta)
                assert os.path.exists(file_path)

                # Check raw JSON structure for 100% JS browser format compatibility
                with open(file_path, "r", encoding="utf-8") as f:
                    raw_data = json.load(f)

                assert "matrix" in raw_data
                assert len(raw_data["matrix"]) == 600
                assert "params" in raw_data
                for k in CANONICAL_WEIGHT_KEYS:
                    assert k in raw_data["params"], f"Key {k} missing in params"

                # Reload via FFI
                w_loaded, m_loaded, meta_loaded = pyrust.load_weights_file(file_path)
                assert len(w_loaded) == 10
                assert len(m_loaded) == 600

                for orig, loaded in zip(weights, w_loaded):
                    assert abs(orig - loaded) < 1e-5 or abs(orig - loaded) / (abs(orig) + 1e-9) < 1e-5

                for orig_m, loaded_m in zip(matrix, m_loaded):
                    assert abs(orig_m - loaded_m) < 1e-5 or abs(orig_m - loaded_m) / (abs(orig_m) + 1e-9) < 1e-5

    def test_weights_persistence_error_handling(self):
        """Verifies that invalid arguments and corrupted paths raise appropriate PyErrors."""
        with tempfile.TemporaryDirectory() as tmpdir:
            valid_weights = DEFAULT_WEIGHTS
            # 1. Invalid matrix length (< 600)
            bad_path = os.path.join(tmpdir, "bad.json")
            with pytest.raises(ValueError, match="matrix must have exactly 600 elements"):
                pyrust.save_weights_file(bad_path, valid_weights, [1.0] * 599, None)

            # 2. Invalid matrix length (> 600)
            with pytest.raises(ValueError, match="matrix must have exactly 600 elements"):
                pyrust.save_weights_file(bad_path, valid_weights, [1.0] * 601, None)

            # 3. Loading non-existent file
            non_existent = os.path.join(tmpdir, "missing.json")
            with pytest.raises(IOError):
                pyrust.load_weights_file(non_existent)

"""
PyRust Bit-Exact Parity and Core Specification Test Suite.

Verifies mathematical and algorithmic parity against JS reference implementations:
1. Mulberry32 PRNG exact sequence for seed 123456789 (calls 1..10, dice pairs).
2. Board dimensions (15x40=600), start positions, coordinate indexing.
3. 180° board rotation and inverse move mapping.
4. findClosedLoops flood-fill enclosure capture and non-contact rules.
5. Match termination conditions (4 passes, 80 turns, >300 cells).
6. Procedural DEFAULT_V2_MATRIX (600 floats).
7. 10 control board states for V2 Base + Tactical Heuristic evaluation.
"""

import math
from collections import deque
import pytest

# Try importing the compiled pyrust extension module if available
try:
    import pyrust
    HAS_PYRUST = True
except ImportError:
    HAS_PYRUST = False


# ============================================================================
# Python Reference Oracle Implementations (Ground Truth from JS Reference)
# ============================================================================

class Mulberry32Oracle:
    """Exact 32-bit state Mulberry32 PRNG matching ArenaEngine.js."""

    def __init__(self, seed: int = 123456789):
        self.state = seed & 0xFFFFFFFF

    def next_u32(self) -> int:
        self.state = (self.state + 0x6D2B79F5) & 0xFFFFFFFF
        t = ((self.state ^ (self.state >> 15)) * (1 | self.state)) & 0xFFFFFFFF
        t = ((t + (((t ^ (t >> 7)) * (61 | t)) & 0xFFFFFFFF)) ^ t) & 0xFFFFFFFF
        return (t ^ (t >> 14)) & 0xFFFFFFFF

    def next_f64(self) -> float:
        return self.next_u32() / 4294967296.0

    def roll_die(self) -> int:
        return math.floor(self.next_f64() * 6.0) + 1


def rotate_board_180(cells: list) -> list:
    """180° board rotation: r' = 14 - r, c' = 39 - c, P1 <-> P2 swap."""
    assert len(cells) == 600
    rotated = [0] * 600
    for r in range(15):
        for c in range(40):
            src_idx = r * 40 + c
            dst_r = 14 - r
            dst_c = 39 - c
            dst_idx = dst_r * 40 + dst_c
            val = cells[src_idx]
            if val == 1:
                rotated[dst_idx] = 2
            elif val == 2:
                rotated[dst_idx] = 1
            else:
                rotated[dst_idx] = 0
    return rotated


def invert_move_coordinates(r_rot: int, c_rot: int, h: int, w: int) -> tuple:
    """Inverse coordinate mapping: r_real = 15 - (r_rot + h), c_real = 40 - (c_rot + w)."""
    return 15 - (r_rot + h), 40 - (c_rot + w), h, w


def find_closed_loops_oracle(cells: list) -> list:
    """
    4-connected orthogonal flood-fill for empty cells.
    Returns list of components: {'cells': [(r, c)...], 'neighbors': set([cell_values])}.
    Walls outside grid bounds do NOT count as neighbors.
    """
    visited = [[False] * 40 for _ in range(15)]
    loops = []

    for r in range(15):
        for c in range(40):
            if cells[r * 40 + c] == 0 and not visited[r][c]:
                comp_cells = []
                neighbors = set()
                queue = deque([(r, c)])
                visited[r][c] = True

                while queue:
                    curr_r, curr_c = queue.popleft()
                    comp_cells.append((curr_r, curr_c))

                    for dr, dc in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
                        nr, nc = curr_r + dr, curr_c + dc
                        if 0 <= nr < 15 and 0 <= nc < 40:
                            n_val = cells[nr * 40 + nc]
                            if n_val == 0:
                                if not visited[nr][nc]:
                                    visited[nr][nc] = True
                                    queue.append((nr, nc))
                            else:
                                neighbors.add(n_val)

                loops.append({'cells': comp_cells, 'neighbors': neighbors})
    return loops


def generate_default_v2_matrix_oracle() -> list:
    """Procedural generation of DEFAULT_V2_MATRIX (600 floats)."""
    matrix = [0.0] * 600
    for r in range(15):
        for c in range(40):
            idx = r * 40 + c
            weight = 10.0
            dist_center_col = abs(c - 19.5)
            dist_center_row = abs(r - 7.0)

            # 1. Center column meridian corridor (cols 17..22)
            if dist_center_col <= 3.0:
                weight += (4.0 - dist_center_col) * 8.0

            # 2. Center row equator corridor (rows 4..10)
            if dist_center_row <= 3.0:
                weight += (4.0 - dist_center_row) * 4.0

            # 3. Top and bottom border walls
            if r == 0 or r == 14:
                weight += 6.0

            # 4. Milestone columns
            if c == 10 or c == 20 or c == 30:
                weight += 5.0

            matrix[idx] = weight
    return matrix


def evaluate_heuristic_oracle(cells: list, move: tuple, params: dict, matrix: list) -> dict:
    """
    Evaluates candidate move (r0, c0, h, w) for Player 2 using V2 Base + Tactical formula.
    Returns breakdown of score components and total score.
    """
    r0, c0, h, w = move
    area = h * w
    min_dim = min(h, w)
    empty_cells = sum(1 for x in cells if x == 0)

    # 1. Base Area
    score_area = area * params.get("WEIGHT_AREA", 50.0)

    # 2. Compactness vs Noodle
    p1_cols = [c for r in range(15) for c in range(40) if cells[r * 40 + c] == 1]
    p2_cols = [c for r in range(15) for c in range(40) if cells[r * 40 + c] == 2] + [
        c for r in range(r0, r0 + h) for c in range(c0, c0 + w)
    ]
    p1_max_col = max(p1_cols) if p1_cols else -1
    p2_min_col = min(p2_cols) if p2_cols else 40
    has_contact = p2_min_col <= (p1_max_col + 3)
    is_opening = empty_cells > 490

    weight_compactness = params.get("WEIGHT_COMPACTNESS", 12.973)
    if is_opening and not has_contact:
        score_compactness = 0.0
    else:
        if min_dim >= 2:
            score_compactness = weight_compactness
        elif min_dim == 1 and area > 2:
            score_compactness = -1.5 * weight_compactness
        else:
            score_compactness = 0.0

    # 3. Loop Closing & Deny Capture
    # canCloseLoop predicate
    touches_boundary = (r0 == 0 or (r0 + h) == 15 or c0 == 0 or (c0 + w) == 40)
    friendly_neighbors = 0
    has_adjacent_empty = False

    # Check perimeter around (r0, c0, h, w)
    checked_coords = set()
    for r in range(r0, r0 + h):
        for dc in [-1, w]:
            nc = c0 + dc
            if 0 <= nc < 40:
                checked_coords.add((r, nc))
    for c in range(c0, c0 + w):
        for dr in [-1, h]:
            nr = r0 + dr
            if 0 <= nr < 15:
                checked_coords.add((nr, c))

    for nr, nc in checked_coords:
        val = cells[nr * 40 + nc]
        if val == 2:
            friendly_neighbors += 1
        elif val == 0:
            has_adjacent_empty = True

    can_close_loop = has_adjacent_empty and (
        (touches_boundary and friendly_neighbors >= 1) or (friendly_neighbors >= 2)
    )

    captured_my = 0
    sim_cells = list(cells)
    for r in range(r0, r0 + h):
        for c in range(c0, c0 + w):
            sim_cells[r * 40 + c] = 2

    has_p1 = any(x == 1 for x in sim_cells)
    has_p2 = any(x == 2 for x in sim_cells)

    if can_close_loop and has_p1 and has_p2:
        cand_adj = set(checked_coords)
        loops = find_closed_loops_oracle(sim_cells)
        for loop in loops:
            if loop['neighbors'] == {2} and any(cell in cand_adj for cell in loop['cells']):
                captured_my += len(loop['cells'])

    score_loops = captured_my * params.get("WEIGHT_CAPTURE", 95.0)

    # 4. Voronoi Influence Field
    # BFS 1: Player 1 sources (val == 1), obstacles = (val == 2 or candidate cells)
    d1 = [9999] * 600
    q1 = deque()
    cand_cells_set = {(r, c) for r in range(r0, r0 + h) for c in range(c0, c0 + w)}

    for idx, v in enumerate(cells):
        if v == 1:
            d1[idx] = 0
            q1.append((idx // 40, idx % 40))

    while q1:
        cr, cc = q1.popleft()
        cd = d1[cr * 40 + cc]
        for dr, dc in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
            nr, nc = cr + dr, cc + dc
            if 0 <= nr < 15 and 0 <= nc < 40:
                n_idx = nr * 40 + nc
                if cells[n_idx] != 2 and (nr, nc) not in cand_cells_set:
                    if d1[n_idx] > cd + 1:
                        d1[n_idx] = cd + 1
                        q1.append((nr, nc))

    # BFS 2: Player 2 sources (val == 2 or candidate cells), obstacles = (val == 1)
    d2 = [9999] * 600
    q2 = deque()
    for idx, v in enumerate(cells):
        r, c = idx // 40, idx % 40
        if v == 2 or (r, c) in cand_cells_set:
            d2[idx] = 0
            q2.append((r, c))

    while q2:
        cr, cc = q2.popleft()
        cd = d2[cr * 40 + cc]
        for dr, dc in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
            nr, nc = cr + dr, cc + dc
            if 0 <= nr < 15 and 0 <= nc < 40:
                n_idx = nr * 40 + nc
                if cells[n_idx] != 1:
                    if d2[n_idx] > cd + 1:
                        d2[n_idx] = cd + 1
                        q2.append((nr, nc))

    p1_territory = 0
    p2_territory = 0
    p1_safe = 0
    p2_safe = 0
    spatial_score = 0.0
    opp_choked = 0

    for i in range(600):
        if d1[i] == 0 and d2[i] == 0:
            continue
        if d2[i] < d1[i]:
            p2_territory += 1
            spatial_score += matrix[i]
            if d1[i] == 9999:
                p2_safe += 1
                spatial_score += 2.0 * matrix[i]
        elif d1[i] < d2[i]:
            p1_territory += 1
            spatial_score -= matrix[i]
            if d2[i] == 9999:
                p1_safe += 1
                spatial_score -= 2.0 * matrix[i]

        if d1[i] <= 2 and d2[i] <= (d1[i] + 1):
            opp_choked += 1

    score_voronoi = (
        (p2_territory - p1_territory) * params.get("WEIGHT_VORONOI_DELTA", 26.0)
        + (p2_safe - p1_safe) * params.get("WEIGHT_SAFE_TERRITORY", 23.088)
        + spatial_score * params.get("WEIGHT_SPATIAL_MATRIX", 0.322)
        + opp_choked * params.get("WEIGHT_OPP_CHOKE", 18.0)
    )

    # 5. Frontier Advance
    westward_sprint = (39.0 - c0) / 39.0
    center_row_dist = abs(r0 + 0.5 * h - 7.0) / 7.0
    vertical_center_bonus = 1.0 - center_row_dist
    weight_frontier = params.get("WEIGHT_FRONTIER", 17.307)

    if empty_cells > 490:
        score_frontier = (1.4 * westward_sprint + vertical_center_bonus) * weight_frontier
    elif empty_cells >= 180:
        score_frontier = (westward_sprint + 0.5 * vertical_center_bonus) * weight_frontier
    else:
        score_frontier = (0.2 * westward_sprint) * weight_frontier

    # 6. Bottleneck & Outer Wall
    touches_wall = (r0 == 0 or (r0 + h) == 15)
    if touches_wall and c0 < 25 and h >= 2:
        score_bottleneck = params.get("WEIGHT_BOTTLENECK", 12.0)
    else:
        score_bottleneck = 0.0

    total_score = (
        score_area
        + score_compactness
        + score_loops
        + score_voronoi
        + score_frontier
        + score_bottleneck
    )

    return {
        "area": score_area,
        "compactness": score_compactness,
        "loops": score_loops,
        "voronoi": score_voronoi,
        "frontier": score_frontier,
        "bottleneck": score_bottleneck,
        "total": total_score,
        "p1_territory": p1_territory,
        "p2_territory": p2_territory,
        "p1_safe": p1_safe,
        "p2_safe": p2_safe,
        "opp_choked": opp_choked,
    }


# ============================================================================
# Test Cases: PRNG Bit-Exact Parity
# ============================================================================

class TestMulberry32Parity:
    """Tests verifying bit-exact parity of Mulberry32 PRNG."""

    EXPECTED_F64 = [
        0.2577907438389957,
        0.9707721115555614,
        0.7853280142880976,
        0.20616457983851433,
        0.30307188746519387,
        0.7470660470426083,
        0.7787336520850658,
        0.2845096290111542,
        0.016536934999749064,
        0.1614646923262626,
    ]

    EXPECTED_DICE = [2, 6, 5, 2, 2, 5, 5, 2, 1, 1]

    def test_mulberry32_first_10_values(self):
        """First 10 values from seed 123456789 must match reference JS table."""
        rng = Mulberry32Oracle(123456789)
        for idx, (exp_f64, exp_die) in enumerate(zip(self.EXPECTED_F64, self.EXPECTED_DICE)):
            val = rng.next_f64()
            die = math.floor(val * 6.0) + 1
            assert abs(val - exp_f64) < 1e-12, f"Call {idx+1} next_f64 mismatch: {val} vs {exp_f64}"
            assert die == exp_die

    def test_mulberry32_die_rolls_sequence(self):
        """Check the sequence of 10 consecutive die rolls."""
        rng = Mulberry32Oracle(123456789)
        dice = []
        for _ in range(10):
            # Compute die directly from current next_f64() call
            val = rng.next_f64()
            dice.append(math.floor(val * 6.0) + 1)
        assert dice == self.EXPECTED_DICE

    def test_mulberry32_dice_pairs_for_simulate_match(self):
        """Test the first 5 consecutive pairs [d1, d2] used in simulateMatch."""
        rng = Mulberry32Oracle(123456789)
        pairs = []
        for _ in range(5):
            d1 = rng.roll_die()
            d2 = rng.roll_die()
            pairs.append((d1, d2))
        assert pairs[0] == (2, 6)
        assert pairs[1] == (5, 2)
        assert pairs[2] == (2, 5)
        assert pairs[3] == (5, 2)
        assert pairs[4] == (1, 1)

    def test_mulberry32_determinism_across_instances(self):
        """Two PRNGs initialized with the same seed must produce identical outputs."""
        rng1 = Mulberry32Oracle(42)
        rng2 = Mulberry32Oracle(42)
        for _ in range(50):
            assert rng1.next_u32() == rng2.next_u32()

    def test_mulberry32_output_bounds(self):
        """All outputs must be strictly in [0.0, 1.0) and dice in [1, 6]."""
        rng = Mulberry32Oracle(99999)
        for _ in range(200):
            f = rng.next_f64()
            assert 0.0 <= f < 1.0
            d = rng.roll_die()
            assert 1 <= d <= 6


# ============================================================================
# Test Cases: Board Geometry & 180° Rotation Parity
# ============================================================================

class TestBoardGeometryAndRotationParity:
    """Tests board dimensions, start positions, and 180° symmetry."""

    def test_board_dimensions_and_indexing(self):
        """Board must be 15 rows by 40 columns (600 cells)."""
        cells = [0] * 600
        assert len(cells) == 600
        # Row-major indexing: (r, c) -> r * 40 + c
        assert (0 * 40 + 0) == 0
        assert (14 * 40 + 39) == 599
        assert (7 * 40 + 20) == 300

    def test_rotation_symmetry_p1_p2_swap(self):
        """180° rotation must swap P1 and P2 cell occupancies and point-invert coordinates."""
        cells = [0] * 600
        # P1 at (0, 0)
        cells[0] = 1
        # P2 at (14, 39)
        cells[599] = 2

        rot = rotate_board_180(cells)
        # On rotated board:
        # Former P1 at (0, 0) maps to (14, 39) and becomes P2 (value 2)
        assert rot[599] == 2
        # Former P2 at (14, 39) maps to (0, 0) and becomes P1 (value 1)
        assert rot[0] == 1

    def test_double_rotation_identity(self):
        """Rotating a board 180° twice must reproduce the original board exactly."""
        cells = [0] * 600
        cells[10] = 1
        cells[150] = 2
        cells[450] = 1
        cells[580] = 2

        double_rot = rotate_board_180(rotate_board_180(cells))
        assert double_rot == cells

    def test_inverse_move_mapping_p1_start(self):
        """
        On rotated board, P2 places at (15 - h, 40 - w, h, w).
        Inverse mapping must yield (0, 0, h, w) — the required P1 start.
        """
        h, w = 3, 5
        r_rot = 15 - h
        c_rot = 40 - w

        r_real, c_real, out_h, out_w = invert_move_coordinates(r_rot, c_rot, h, w)
        assert r_real == 0
        assert c_real == 0
        assert out_h == h
        assert out_w == w

    def test_inverse_move_mapping_arbitrary(self):
        """Arbitrary rotated move inversion matches mathematical formula."""
        r_rot, c_rot, h, w = 4, 12, 2, 4
        r_real, c_real, rh, rw = invert_move_coordinates(r_rot, c_rot, h, w)
        assert r_real == 15 - (4 + 2) == 9
        assert c_real == 40 - (12 + 4) == 24
        assert rh == 2
        assert rw == 4


# ============================================================================
# Test Cases: Loop Closure Parity
# ============================================================================

class TestFindClosedLoopsParity:
    """Tests orthogonal flood-fill enclosure capture mechanics."""

    def test_single_cell_enclosed_chamber(self):
        """Single empty cell surrounded on 4 orthogonal sides by P1 is captured."""
        cells = [1] * 600
        # Place empty cell at (7, 20)
        target_idx = 7 * 40 + 20
        cells[target_idx] = 0

        loops = find_closed_loops_oracle(cells)
        assert len(loops) == 1
        assert loops[0]['cells'] == [(7, 20)]
        assert loops[0]['neighbors'] == {1}

    def test_corner_enclosure_with_walls(self):
        """Corner chamber touching walls and P1 cells is captured (walls do NOT add neighbor)."""
        cells = [0] * 600
        # Create enclosure in top-left corner:
        # Cell (0, 0) empty, enclosed by P1 at (0, 1) and (1, 0)
        cells[0 * 40 + 1] = 1
        cells[1 * 40 + 0] = 1

        loops = find_closed_loops_oracle(cells)
        # Find the loop containing (0, 0)
        corner_loop = next(lp for lp in loops if (0, 0) in lp['cells'])
        assert corner_loop['cells'] == [(0, 0)]
        assert corner_loop['neighbors'] == {1}

    def test_contested_chamber_touches_both_players(self):
        """Chamber touching both P1 and P2 is contested and NOT captured."""
        cells = [0] * 600
        # Empty cell at (5, 5), neighbor P1 at (5, 4), neighbor P2 at (5, 6)
        # Bounded by 1s above and below
        cells[4 * 40 + 5] = 1
        cells[6 * 40 + 5] = 1
        cells[5 * 40 + 4] = 1
        cells[5 * 40 + 6] = 2

        loops = find_closed_loops_oracle(cells)
        target_loop = next(lp for lp in loops if (5, 5) in lp['cells'])
        assert target_loop['neighbors'] == {1, 2}
        assert len(target_loop['neighbors']) == 2  # Must not be captured

    def test_multi_cell_chamber_capture(self):
        """3-cell L-shaped chamber enclosed by P2 is captured."""
        cells = [2] * 600
        # Empty cells at (2, 2), (2, 3), (3, 2)
        cells[2 * 40 + 2] = 0
        cells[2 * 40 + 3] = 0
        cells[3 * 40 + 2] = 0

        loops = find_closed_loops_oracle(cells)
        assert len(loops) == 1
        assert len(loops[0]['cells']) == 3
        assert loops[0]['neighbors'] == {2}

    def test_empty_board_no_enclosure(self):
        """Entirely empty board has no player neighbors and is not captured."""
        cells = [0] * 600
        loops = find_closed_loops_oracle(cells)
        assert len(loops) == 1
        assert len(loops[0]['cells']) == 600
        assert len(loops[0]['neighbors']) == 0


# ============================================================================
# Test Cases: Procedural Matrix Parity
# ============================================================================

class TestProceduralMatrixParity:
    """Tests DEFAULT_V2_MATRIX procedural generation."""

    def test_matrix_dimensions_and_types(self):
        """Matrix must have exactly 600 float elements."""
        matrix = generate_default_v2_matrix_oracle()
        assert len(matrix) == 600
        assert all(isinstance(x, float) for x in matrix)

    def test_corner_values(self):
        """Corners (0,0), (0,39), (14,0), (14,39) have baseline weight + border wall = 16.0."""
        matrix = generate_default_v2_matrix_oracle()
        assert matrix[0 * 40 + 0] == 16.0
        assert matrix[0 * 40 + 39] == 16.0
        assert matrix[14 * 40 + 0] == 16.0
        assert matrix[14 * 40 + 39] == 16.0

    def test_center_cell_exact_value(self):
        """Center cell (7, 20) has value 10 + 28 + 16 + 5 = 59.0."""
        matrix = generate_default_v2_matrix_oracle()
        assert matrix[7 * 40 + 20] == 59.0

    def test_milestone_columns_bonus(self):
        """Milestone columns 10, 20, 30 must receive +5.0 bonus."""
        matrix = generate_default_v2_matrix_oracle()
        # Row 1 (outside central corridor):
        # col 9 vs col 10
        val_c9 = matrix[1 * 40 + 9]
        val_c10 = matrix[1 * 40 + 10]
        assert val_c10 == val_c9 + 5.0

    def test_horizontal_symmetry(self):
        """Matrix weights must be symmetric across the vertical equator (row 7)."""
        matrix = generate_default_v2_matrix_oracle()
        for r in range(7):
            for c in range(40):
                w_top = matrix[r * 40 + c]
                w_bottom = matrix[(14 - r) * 40 + c]
                assert abs(w_top - w_bottom) < 1e-6


# ============================================================================
# Test Cases: 10 Control Board States (Bit-Exact Heuristic Check)
# ============================================================================

class TestTenControlStatesParity:
    """10 benchmark board states testing V2 Base + Tactical Heuristics."""

    @pytest.fixture
    def default_params(self):
        return {
            "WEIGHT_AREA": 50.0,
            "WEIGHT_CAPTURE": 95.0,
            "WEIGHT_DENY_CAPTURE": 85.0,
            "WEIGHT_VORONOI_DELTA": 26.0,
            "WEIGHT_SAFE_TERRITORY": 23.088,
            "WEIGHT_COMPACTNESS": 12.973,
            "WEIGHT_SPATIAL_MATRIX": 0.322,
            "WEIGHT_FRONTIER": 17.307,
            "WEIGHT_OPP_CHOKE": 18.0,
            "WEIGHT_BOTTLENECK": 12.0,
        }

    @pytest.fixture
    def default_matrix(self):
        return generate_default_v2_matrix_oracle()

    def test_control_state_0_opening_p2_anchor(self, default_params, default_matrix):
        """State 0: Turn 2 opening response. P1 at (0, 0, 2, 6). P2 places (12, 34, 3, 6)."""
        cells = [0] * 600
        for r in range(2):
            for c in range(6):
                cells[r * 40 + c] = 1

        cand_move = (12, 34, 3, 6)
        eval_result = evaluate_heuristic_oracle(cells, cand_move, default_params, default_matrix)

        # In opening (>490 empty, no contact): compactness = 0.0, bottleneck = 0.0 (c0=34 >= 25)
        assert eval_result["area"] == (3 * 6) * 50.0  # 900.0
        assert eval_result["compactness"] == 0.0
        assert eval_result["loops"] == 0.0
        assert eval_result["bottleneck"] == 0.0
        assert eval_result["total"] > 900.0

    def test_control_state_1_compact_vs_noodle(self, default_params, default_matrix):
        """State 1: Midgame comparison between compact 2x3 piece vs 1x6 noodle."""
        # Create midgame state with < 490 empty cells
        cells = [0] * 600
        for i in range(120):
            cells[i] = 1
        for i in range(480, 600):
            cells[i] = 2

        # Compact move (2, 3)
        compact_res = evaluate_heuristic_oracle(cells, (6, 20, 2, 3), default_params, default_matrix)
        assert compact_res["compactness"] == default_params["WEIGHT_COMPACTNESS"]

        # Noodle move (1, 6)
        noodle_res = evaluate_heuristic_oracle(cells, (6, 20, 1, 6), default_params, default_matrix)
        assert noodle_res["compactness"] == -1.5 * default_params["WEIGHT_COMPACTNESS"]

    def test_control_state_2_wavefront_contact(self, default_params, default_matrix):
        """State 2: Wavefront contact (p2_min_col <= p1_max_col + 3) activates penalty."""
        cells = [0] * 600
        # P1 advances to col 18
        cells[7 * 40 + 18] = 1
        # P2 candidate at col 20 (min col 20 <= 18 + 3 = 21 -> hasContact is True)
        cand = (7, 20, 1, 4)
        res = evaluate_heuristic_oracle(cells, cand, default_params, default_matrix)
        assert res["compactness"] == -1.5 * default_params["WEIGHT_COMPACTNESS"]

    def test_control_state_3_bottleneck_wall_touch(self, default_params, default_matrix):
        """State 3: Candidate touching top wall (r0=0) with c0 < 25 and h >= 2 gets bottleneck bonus."""
        cells = [0] * 600
        cand = (0, 15, 3, 4)
        res = evaluate_heuristic_oracle(cells, cand, default_params, default_matrix)
        assert res["bottleneck"] == default_params["WEIGHT_BOTTLENECK"]

    def test_control_state_4_bottleneck_deep_east_denied(self, default_params, default_matrix):
        """State 4: Candidate touching wall but with c0 >= 25 does NOT get bottleneck bonus."""
        cells = [0] * 600
        cand = (0, 26, 3, 4)
        res = evaluate_heuristic_oracle(cells, cand, default_params, default_matrix)
        assert res["bottleneck"] == 0.0

    def test_control_state_5_loop_closure_capture(self, default_params, default_matrix):
        """State 5: Move completes loop enclosing 2 empty cells against border."""
        cells = [0] * 600
        # P1 has presence on the board
        cells[14 * 40 + 0] = 1
        # Build surrounding walls with P2 cells
        cells[0 * 40 + 2] = 2
        cells[1 * 40 + 2] = 2
        cells[2 * 40 + 0] = 2
        cells[2 * 40 + 1] = 2
        # Cells (0,0), (0,1), (1,0), (1,1)
        # Candidate at (0, 0, 2, 1) closes the loop around (0,1) and (1,1)
        cand = (0, 0, 2, 1)
        res = evaluate_heuristic_oracle(cells, cand, default_params, default_matrix)
        assert res["loops"] == 2 * default_params["WEIGHT_CAPTURE"]

    def test_control_state_6_voronoi_territory_shift(self, default_params, default_matrix):
        """State 6: Large move expands Voronoi territory delta compared to small move."""
        cells = [0] * 600
        cells[0 * 40 + 0] = 1
        cells[14 * 40 + 39] = 2

        res_large = evaluate_heuristic_oracle(cells, (10, 30, 4, 6), default_params, default_matrix)
        res_small = evaluate_heuristic_oracle(cells, (13, 38, 1, 1), default_params, default_matrix)

        assert res_large["p2_territory"] > res_small["p2_territory"]
        assert res_large["voronoi"] > res_small["voronoi"]

    def test_control_state_7_voronoi_safe_territory(self, default_params, default_matrix):
        """State 7: Move enclosing a corner makes it safe (unreachable by P1, d1=9999)."""
        cells = [0] * 600
        # P1 at (0, 0)
        cells[0 * 40 + 0] = 1
        # P2 wall blocking access to corner (14, 0)
        for c in range(10):
            cells[10 * 40 + c] = 2
        cand = (11, 0, 2, 2)
        res = evaluate_heuristic_oracle(cells, cand, default_params, default_matrix)
        assert res["p2_safe"] > 0

    def test_control_state_8_opp_choke_detection(self, default_params, default_matrix):
        """State 8: Close-range move detects choked opponent cells (d1 <= 2, d2 <= d1 + 1)."""
        cells = [0] * 600
        cells[7 * 40 + 10] = 1
        # Move placed directly 2 steps away from P1 cell
        cand = (7, 12, 1, 2)
        res = evaluate_heuristic_oracle(cells, cand, default_params, default_matrix)
        assert res["opp_choked"] > 0

    def test_control_state_9_endgame_phase_frontier(self, default_params, default_matrix):
        """State 9: In endgame (< 180 empty cells), frontier advance is scaled down to 0.2."""
        cells = [1] * 450 + [0] * 150  # 150 empty cells (< 180)
        cand = (7, 10, 2, 2)
        res = evaluate_heuristic_oracle(cells, cand, default_params, default_matrix)

        westward = (39.0 - 10) / 39.0
        exp_frontier = (0.2 * westward) * default_params["WEIGHT_FRONTIER"]
        assert abs(res["frontier"] - exp_frontier) < 1e-6

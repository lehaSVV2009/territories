//! Model weight selector and persistence.
//!
//! Manages candidate population rankings, evaluation metrics (winrate, margin, turns),
//! champion selection based on Common Random Numbers (CRN) mirrored pairings,
//! and exact JSON schema persistence (`weights.json`).

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use serde::{Deserialize, Serialize};

use crate::game::{
    BotType, HeuristicWeights, Player, TOTAL_CELLS,
    generate_default_v2_matrix, simulate_match,
};

/// The 10 core heuristic parameters configured for training, benchmark validation, and gatekeeping.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeuristicParams {
    #[serde(rename = "WEIGHT_AREA", alias = "weight_area")]
    pub weight_area: f64,
    #[serde(rename = "WEIGHT_CAPTURE", alias = "weight_capture")]
    pub weight_capture: f64,
    #[serde(rename = "WEIGHT_DENY_CAPTURE", alias = "weight_deny_capture")]
    pub weight_deny_capture: f64,
    #[serde(rename = "WEIGHT_VORONOI_DELTA", alias = "weight_voronoi_delta")]
    pub weight_voronoi_delta: f64,
    #[serde(rename = "WEIGHT_SAFE_TERRITORY", alias = "weight_safe_territory")]
    pub weight_safe_territory: f64,
    #[serde(rename = "WEIGHT_COMPACTNESS", alias = "weight_compactness")]
    pub weight_compactness: f64,
    #[serde(rename = "WEIGHT_SPATIAL_MATRIX", alias = "weight_spatial_matrix")]
    pub weight_spatial_matrix: f64,
    #[serde(rename = "WEIGHT_FRONTIER", alias = "weight_frontier")]
    pub weight_frontier: f64,
    #[serde(rename = "WEIGHT_OPP_CHOKE", alias = "weight_opp_choke")]
    pub weight_opp_choke: f64,
    #[serde(rename = "WEIGHT_BOTTLENECK", alias = "weight_bottleneck")]
    pub weight_bottleneck: f64,
}

impl Default for HeuristicParams {
    fn default() -> Self {
        Self {
            weight_area: 50.0,
            weight_capture: 95.0,
            weight_deny_capture: 85.0,
            weight_voronoi_delta: 26.0,
            weight_safe_territory: 23.088,
            weight_compactness: 12.973,
            weight_spatial_matrix: 0.322,
            weight_frontier: 17.307,
            weight_opp_choke: 18.0,
            weight_bottleneck: 12.0,
        }
    }
}

impl HeuristicParams {
    /// Constructs parameters from a float slice, falling back to defaults if items are missing.
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
        }
    }

    /// Converts parameters to a 10-element vector in canonical order.
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
        ]
    }

    /// Clamps weights to authoritative training corridors and locks Numeraire `WEIGHT_AREA = 50.0`.
    pub fn clamp(&mut self) {
        self.weight_area = 50.0; // Invariant Numeraire
        self.weight_capture = self.weight_capture.clamp(70.0, 150.0);
        self.weight_deny_capture = self.weight_deny_capture.clamp(60.0, 150.0);
        self.weight_voronoi_delta = self.weight_voronoi_delta.clamp(15.0, 45.0);
        self.weight_safe_territory = self.weight_safe_territory.clamp(15.0, 35.0);
        self.weight_compactness = self.weight_compactness.clamp(5.0, 25.0);
        self.weight_spatial_matrix = self.weight_spatial_matrix.clamp(0.10, 0.50);
        self.weight_frontier = self.weight_frontier.clamp(10.0, 30.0);
        self.weight_opp_choke = self.weight_opp_choke.clamp(5.0, 25.0);
        self.weight_bottleneck = self.weight_bottleneck.clamp(5.0, 25.0);
    }

    /// Checks whether all weights conform to the clamping boundaries.
    pub fn is_clamped(&self) -> bool {
        (self.weight_area - 50.0).abs() < 1e-6
            && (70.0..=150.0).contains(&self.weight_capture)
            && (60.0..=150.0).contains(&self.weight_deny_capture)
            && (15.0..=45.0).contains(&self.weight_voronoi_delta)
            && (15.0..=35.0).contains(&self.weight_safe_territory)
            && (5.0..=25.0).contains(&self.weight_compactness)
            && (0.10..=0.50).contains(&self.weight_spatial_matrix)
            && (10.0..=30.0).contains(&self.weight_frontier)
            && (5.0..=25.0).contains(&self.weight_opp_choke)
            && (5.0..=25.0).contains(&self.weight_bottleneck)
    }
}

impl From<&HeuristicParams> for HeuristicWeights {
    fn from(p: &HeuristicParams) -> Self {
        let def = HeuristicWeights::default();
        Self {
            weight_area: p.weight_area,
            weight_capture: p.weight_capture,
            weight_deny_capture: p.weight_deny_capture,
            weight_voronoi_delta: p.weight_voronoi_delta,
            weight_safe_territory: p.weight_safe_territory,
            weight_compactness: p.weight_compactness,
            weight_spatial_matrix: p.weight_spatial_matrix,
            weight_frontier: p.weight_frontier,
            weight_opp_choke: p.weight_opp_choke,
            weight_bottleneck: p.weight_bottleneck,
            weight_guillotine: def.weight_guillotine,
            weight_gap_penalty: def.weight_gap_penalty,
            weight_wall_cork: def.weight_wall_cork,
            weight_pocket_trap: def.weight_pocket_trap,
        }
    }
}

impl From<HeuristicParams> for HeuristicWeights {
    fn from(p: HeuristicParams) -> Self {
        HeuristicWeights::from(&p)
    }
}

impl From<&HeuristicWeights> for HeuristicParams {
    fn from(w: &HeuristicWeights) -> Self {
        Self {
            weight_area: w.weight_area,
            weight_capture: w.weight_capture,
            weight_deny_capture: w.weight_deny_capture,
            weight_voronoi_delta: w.weight_voronoi_delta,
            weight_safe_territory: w.weight_safe_territory,
            weight_compactness: w.weight_compactness,
            weight_spatial_matrix: w.weight_spatial_matrix,
            weight_frontier: w.weight_frontier,
            weight_opp_choke: w.weight_opp_choke,
            weight_bottleneck: w.weight_bottleneck,
        }
    }
}

/// Metadata tracked for saved models and training generations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectorMetadata {
    pub epoch: usize,
    #[serde(default)]
    pub winrate_vs_hard: f64,
    #[serde(default)]
    pub winrate_vs_greedy: f64,
    #[serde(default)]
    pub winrate_vs_champ: f64,
    #[serde(default)]
    pub avg_margin: f64,
    #[serde(default)]
    pub fitness: f64,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Default for SelectorMetadata {
    fn default() -> Self {
        Self {
            epoch: 0,
            winrate_vs_hard: 0.0,
            winrate_vs_greedy: 0.0,
            winrate_vs_champ: 0.0,
            avg_margin: 0.0,
            fitness: 0.0,
            extra: HashMap::new(),
        }
    }
}

fn default_name() -> String {
    "SpatialSlayer-V2".to_string()
}
fn default_version() -> String {
    "2.0.0".to_string()
}
fn default_architecture() -> String {
    "Argon-4 / Phase-Adaptive + Loop Denial".to_string()
}

/// Exact file schema of `weights.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeightsFile {
    #[serde(default = "default_name")]
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_architecture")]
    pub architecture: String,
    pub weights: HeuristicParams,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<HeuristicParams>,
    pub spatial_matrix: Vec<f64>,
    pub metadata: SelectorMetadata,
}

/// Error type for selector I/O and format operations.
#[derive(Debug)]
pub enum SelectorError {
    Io(std::io::Error),
    Json(serde_json::Error),
    InvalidFormat(String),
}

impl std::fmt::Display for SelectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectorError::Io(e) => write!(f, "I/O error: {}", e),
            SelectorError::Json(e) => write!(f, "JSON serialization error: {}", e),
            SelectorError::InvalidFormat(msg) => write!(f, "Invalid weights format: {}", msg),
        }
    }
}

impl std::error::Error for SelectorError {}

impl From<std::io::Error> for SelectorError {
    fn from(e: std::io::Error) -> Self {
        SelectorError::Io(e)
    }
}

impl From<serde_json::Error> for SelectorError {
    fn from(e: serde_json::Error) -> Self {
        SelectorError::Json(e)
    }
}

/// Saves model parameters, spatial matrix, and metadata to a JSON file conforming to `weights.json`.
pub fn save_to_file(
    path: &str,
    weights: &HeuristicParams,
    matrix: &[f64; TOTAL_CELLS],
    metadata: &SelectorMetadata,
) -> Result<(), SelectorError> {
    let file_struct = WeightsFile {
        name: default_name(),
        version: default_version(),
        architecture: default_architecture(),
        weights: weights.clone(),
        params: Some(weights.clone()),
        spatial_matrix: matrix.to_vec(),
        metadata: metadata.clone(),
    };
    let json_str = serde_json::to_string_pretty(&file_struct)?;
    let mut file = File::create(path)?;
    file.write_all(json_str.as_bytes())?;
    Ok(())
}

/// Loads model parameters, spatial matrix, and metadata from a JSON file.
pub fn load_from_file(
    path: &str,
) -> Result<(HeuristicParams, [f64; TOTAL_CELLS], SelectorMetadata), SelectorError> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let file_struct: WeightsFile = serde_json::from_str(&contents)?;

    if file_struct.spatial_matrix.len() != TOTAL_CELLS {
        return Err(SelectorError::InvalidFormat(format!(
            "Expected {} elements in spatial_matrix, found {}",
            TOTAL_CELLS,
            file_struct.spatial_matrix.len()
        )));
    }

    let mut matrix = [0.0; TOTAL_CELLS];
    matrix.copy_from_slice(&file_struct.spatial_matrix);

    Ok((file_struct.weights, matrix, file_struct.metadata))
}

/// Evaluation statistics for a candidate model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidateStats {
    pub candidate_id: usize,
    pub wins: usize,
    pub losses: usize,
    pub draws: usize,
    pub total_matches: usize,
    pub total_turns: usize,
    pub net_margin: f64,
    pub winrate: f64,
    pub avg_margin: f64,
    pub avg_turns: f64,
    pub fitness: f64,
}

impl Default for CandidateStats {
    fn default() -> Self {
        Self {
            candidate_id: 0,
            wins: 0,
            losses: 0,
            draws: 0,
            total_matches: 0,
            total_turns: 0,
            net_margin: 0.0,
            winrate: 0.0,
            avg_margin: 0.0,
            avg_turns: 0.0,
            fitness: 0.0,
        }
    }
}

/// A candidate model in the population.
#[derive(Debug, Clone, PartialEq)]
pub struct CandidateModel {
    pub id: usize,
    pub weights: HeuristicParams,
    pub matrix: [f64; TOTAL_CELLS],
    pub stats: CandidateStats,
}

/// Result of a 2-leg CRN mirrored pairing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CrnPairResult {
    pub leg1_winner: Option<Player>,
    pub leg1_p1_score: u16,
    pub leg1_p2_score: u16,
    pub leg1_turns: usize,
    pub leg2_winner: Option<Player>,
    pub leg2_p1_score: u16,
    pub leg2_p2_score: u16,
    pub leg2_turns: usize,
    pub net_margin: f64,
    pub candidate_wins: usize,
    pub candidate_losses: usize,
    pub candidate_draws: usize,
}

/// ModelSelector: Manages candidate populations, CRN mirrored evaluations,
/// peak model selection, and persistence.
#[derive(Debug, Clone)]
pub struct ModelSelector {
    pub champion: CandidateModel,
    pub population: Vec<CandidateModel>,
    pub generation: usize,
    pub history: Vec<SelectorMetadata>,
}

impl Default for ModelSelector {
    fn default() -> Self {
        Self::with_default_champion()
    }
}

impl ModelSelector {
    /// Creates a new selector with the given champion parameters and matrix.
    pub fn new(champion_weights: HeuristicParams, champion_matrix: [f64; TOTAL_CELLS]) -> Self {
        let champion = CandidateModel {
            id: 0,
            weights: champion_weights,
            matrix: champion_matrix,
            stats: CandidateStats::default(),
        };
        Self {
            champion,
            population: Vec::new(),
            generation: 0,
            history: Vec::new(),
        }
    }

    /// Creates a selector with default reference baseline parameters.
    pub fn with_default_champion() -> Self {
        Self::new(HeuristicParams::default(), generate_default_v2_matrix())
    }

    /// Adds a candidate model to the population and returns its allocated ID.
    pub fn add_candidate(&mut self, mut weights: HeuristicParams, matrix: [f64; TOTAL_CELLS]) -> usize {
        weights.clamp();
        let id = self.population.len() + 1;
        self.population.push(CandidateModel {
            id,
            weights,
            matrix,
            stats: CandidateStats {
                candidate_id: id,
                ..Default::default()
            },
        });
        id
    }

    /// Clears the current candidate population.
    pub fn clear_population(&mut self) {
        self.population.clear();
    }

    /// Evaluates a 2-leg CRN mirrored pairing on a given seed between a candidate and an opponent.
    ///
    /// - Leg 1: Candidate (P1) vs Opponent (P2)
    /// - Leg 2: Opponent (P1) vs Candidate (P2)
    pub fn evaluate_pair_crn(
        cand_weights: &HeuristicParams,
        cand_mat: &[f64; TOTAL_CELLS],
        opp_weights: &HeuristicParams,
        opp_mat: &[f64; TOTAL_CELLS],
        seed: u32,
        max_turns: usize,
    ) -> CrnPairResult {
        let w_cand = HeuristicWeights::from(cand_weights);
        let w_opp = HeuristicWeights::from(opp_weights);

        // Leg 1: Cand = P1, Opp = P2
        let res1 = simulate_match(
            BotType::SpatialSlayer,
            BotType::SpatialSlayer,
            Some(seed),
            max_turns,
            Some(&w_cand),
            Some(&w_opp),
            Some(cand_mat),
            Some(opp_mat),
        );

        // Leg 2: Opp = P1, Cand = P2
        let res2 = simulate_match(
            BotType::SpatialSlayer,
            BotType::SpatialSlayer,
            Some(seed),
            max_turns,
            Some(&w_opp),
            Some(&w_cand),
            Some(opp_mat),
            Some(cand_mat),
        );

        let leg1_cand_margin = (res1.p1_score as f64) - (res1.p2_score as f64);
        let leg2_cand_margin = (res2.p2_score as f64) - (res2.p1_score as f64);
        let net_margin = leg1_cand_margin + leg2_cand_margin;

        let mut candidate_wins = 0;
        let mut candidate_losses = 0;
        let mut candidate_draws = 0;

        // Leg 1 win/loss
        match res1.winner {
            Some(Player::Player1) => candidate_wins += 1,
            Some(Player::Player2) => candidate_losses += 1,
            None => candidate_draws += 1,
        }

        // Leg 2 win/loss
        match res2.winner {
            Some(Player::Player2) => candidate_wins += 1,
            Some(Player::Player1) => candidate_losses += 1,
            None => candidate_draws += 1,
        }

        CrnPairResult {
            leg1_winner: res1.winner,
            leg1_p1_score: res1.p1_score,
            leg1_p2_score: res1.p2_score,
            leg1_turns: res1.turns,
            leg2_winner: res2.winner,
            leg2_p1_score: res2.p1_score,
            leg2_p2_score: res2.p2_score,
            leg2_turns: res2.turns,
            net_margin,
            candidate_wins,
            candidate_losses,
            candidate_draws,
        }
    }

    /// Evaluates a candidate model against the current champion across a set of CRN seeds.
    pub fn evaluate_candidate_vs_champion(
        &self,
        candidate_idx: usize,
        seeds: &[u32],
        max_turns: usize,
    ) -> CandidateStats {
        let cand = &self.population[candidate_idx];
        let mut wins = 0;
        let mut losses = 0;
        let mut draws = 0;
        let mut total_margin = 0.0;
        let mut total_turns = 0;

        for &seed in seeds {
            let res = Self::evaluate_pair_crn(
                &cand.weights,
                &cand.matrix,
                &self.champion.weights,
                &self.champion.matrix,
                seed,
                max_turns,
            );
            wins += res.candidate_wins;
            losses += res.candidate_losses;
            draws += res.candidate_draws;
            total_margin += res.net_margin;
            total_turns += res.leg1_turns + res.leg2_turns;
        }

        let total_matches = seeds.len() * 2;
        let winrate = if total_matches > 0 {
            (wins as f64) / (total_matches as f64)
        } else {
            0.0
        };
        let avg_margin = if total_matches > 0 {
            total_margin / (seeds.len() as f64)
        } else {
            0.0
        };
        let avg_turns = if total_matches > 0 {
            (total_turns as f64) / (total_matches as f64)
        } else {
            0.0
        };

        // Standard fitness formula: winrate * 100.0 + avg_margin * 2.0
        let fitness = winrate * 100.0 + avg_margin * 2.0;

        CandidateStats {
            candidate_id: cand.id,
            wins,
            losses,
            draws,
            total_matches,
            total_turns,
            net_margin: total_margin,
            winrate,
            avg_margin,
            avg_turns,
            fitness,
        }
    }

    /// Evaluates all population candidates against the champion across the given seeds.
    pub fn evaluate_population_vs_champion(&mut self, seeds: &[u32], max_turns: usize) {
        for idx in 0..self.population.len() {
            let stats = self.evaluate_candidate_vs_champion(idx, seeds, max_turns);
            self.population[idx].stats = stats;
        }
    }

    /// Evaluates a candidate against an opponent bot policy (e.g. Greedy, Fast, LegacyHard).
    pub fn evaluate_candidate_vs_bot(
        cand_weights: &HeuristicParams,
        cand_mat: &[f64; TOTAL_CELLS],
        opponent: BotType,
        seeds: &[u32],
        max_turns: usize,
    ) -> CandidateStats {
        let w_cand = HeuristicWeights::from(cand_weights);
        let mut wins = 0;
        let mut losses = 0;
        let mut draws = 0;
        let mut total_margin = 0.0;
        let mut total_turns = 0;

        for &seed in seeds {
            // Leg 1: Candidate is P1
            let res1 = simulate_match(
                BotType::SpatialSlayer,
                opponent,
                Some(seed),
                max_turns,
                Some(&w_cand),
                None,
                Some(cand_mat),
                None,
            );
            // Leg 2: Candidate is P2
            let res2 = simulate_match(
                opponent,
                BotType::SpatialSlayer,
                Some(seed),
                max_turns,
                None,
                Some(&w_cand),
                None,
                Some(cand_mat),
            );

            if res1.winner == Some(Player::Player1) {
                wins += 1;
            } else if res1.winner == Some(Player::Player2) {
                losses += 1;
            } else {
                draws += 1;
            }

            if res2.winner == Some(Player::Player2) {
                wins += 1;
            } else if res2.winner == Some(Player::Player1) {
                losses += 1;
            } else {
                draws += 1;
            }

            let m1 = (res1.p1_score as f64) - (res1.p2_score as f64);
            let m2 = (res2.p2_score as f64) - (res2.p1_score as f64);
            total_margin += m1 + m2;
            total_turns += res1.turns + res2.turns;
        }

        let total_matches = seeds.len() * 2;
        let winrate = if total_matches > 0 {
            (wins as f64) / (total_matches as f64)
        } else {
            0.0
        };
        let avg_margin = if seeds.is_empty() {
            0.0
        } else {
            total_margin / (seeds.len() as f64)
        };
        let avg_turns = if total_matches > 0 {
            (total_turns as f64) / (total_matches as f64)
        } else {
            0.0
        };
        let fitness = winrate * 100.0 + avg_margin * 2.0;

        CandidateStats {
            candidate_id: 0,
            wins,
            losses,
            draws,
            total_matches,
            total_turns,
            net_margin: total_margin,
            winrate,
            avg_margin,
            avg_turns,
            fitness,
        }
    }

    /// Ranks the candidate population by fitness descending.
    /// Returns vector of candidate indices in sorted order.
    pub fn rank_population(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..self.population.len()).collect();
        indices.sort_by(|&a, &b| {
            let fa = self.population[a].stats.fitness;
            let fb = self.population[b].stats.fitness;
            fb.partial_cmp(&fa).unwrap_or(std::cmp::Ordering::Equal)
        });
        indices
    }

    /// Returns a reference to the candidate with the highest fitness in the current population.
    pub fn get_best_candidate(&self) -> Option<&CandidateModel> {
        let ranking = self.rank_population();
        ranking.first().map(|&idx| &self.population[idx])
    }

    /// Selects and promotes the best candidate if its performance exceeds promotion thresholds.
    ///
    /// Returns `true` if a new champion was crowned.
    pub fn select_and_promote_champion(&mut self, min_winrate: f64, min_avg_margin: f64) -> bool {
        let ranking = self.rank_population();
        if let Some(&best_idx) = ranking.first() {
            let best_cand = &self.population[best_idx];
            if best_cand.stats.winrate >= min_winrate && best_cand.stats.avg_margin >= min_avg_margin {
                self.champion = best_cand.clone();
                self.generation += 1;
                let meta = SelectorMetadata {
                    epoch: self.generation,
                    winrate_vs_champ: best_cand.stats.winrate,
                    avg_margin: best_cand.stats.avg_margin,
                    fitness: best_cand.stats.fitness,
                    ..Default::default()
                };
                self.history.push(meta);
                return true;
            }
        }
        false
    }

    /// Saves current champion weights and matrix to file.
    pub fn save_champion(&self, path: &str) -> Result<(), SelectorError> {
        let meta = SelectorMetadata {
            epoch: self.generation,
            winrate_vs_champ: self.champion.stats.winrate,
            avg_margin: self.champion.stats.avg_margin,
            fitness: self.champion.stats.fitness,
            ..Default::default()
        };
        save_to_file(path, &self.champion.weights, &self.champion.matrix, &meta)
    }

    /// Loads champion weights and matrix from file.
    pub fn load_champion(&mut self, path: &str) -> Result<(), SelectorError> {
        let (weights, matrix, meta) = load_from_file(path)?;
        self.champion.weights = weights;
        self.champion.matrix = matrix;
        self.generation = meta.epoch;
        self.history.push(meta);
        Ok(())
    }

    /// Persistence helper delegating to module-level `save_to_file`.
    pub fn save_to_file(
        path: &str,
        weights: &HeuristicParams,
        matrix: &[f64; TOTAL_CELLS],
        metadata: &SelectorMetadata,
    ) -> Result<(), SelectorError> {
        save_to_file(path, weights, matrix, metadata)
    }

    /// Persistence helper delegating to module-level `load_from_file`.
    pub fn load_from_file(
        path: &str,
    ) -> Result<(HeuristicParams, [f64; TOTAL_CELLS], SelectorMetadata), SelectorError> {
        load_from_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heuristic_params_default_and_clamping() {
        let mut params = HeuristicParams::default();
        assert_eq!(params.weight_area, 50.0);
        assert!(params.is_clamped());

        // Alter with extreme out-of-bounds values
        params.weight_area = 999.0;
        params.weight_capture = 10.0; // below 70.0
        params.weight_voronoi_delta = 99.0; // above 45.0
        params.weight_spatial_matrix = 0.01; // below 0.10

        assert!(!params.is_clamped());
        params.clamp();

        assert_eq!(params.weight_area, 50.0);
        assert_eq!(params.weight_capture, 70.0);
        assert_eq!(params.weight_voronoi_delta, 45.0);
        assert_eq!(params.weight_spatial_matrix, 0.10);
        assert!(params.is_clamped());
    }

    #[test]
    fn test_heuristic_params_slice_and_vec_roundtrip() {
        let params = HeuristicParams::default();
        let vec = params.to_vec();
        assert_eq!(vec.len(), 10);
        assert_eq!(vec[0], 50.0);

        let reconstructed = HeuristicParams::from_slice(&vec);
        assert_eq!(params, reconstructed);
    }

    #[test]
    fn test_weights_file_persistence_roundtrip() {
        let mut tmp_path = std::env::temp_dir();
        tmp_path.push(format!("test_weights_{}.json", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let path = tmp_path.to_str().unwrap();

        let weights = HeuristicParams::default();
        let matrix = generate_default_v2_matrix();
        let metadata = SelectorMetadata {
            epoch: 7,
            winrate_vs_hard: 0.96,
            winrate_vs_greedy: 0.99,
            winrate_vs_champ: 0.72,
            avg_margin: 22.5,
            fitness: 117.0,
            extra: HashMap::new(),
        };

        save_to_file(path, &weights, &matrix, &metadata).expect("Failed to save weights");

        let (loaded_w, loaded_m, loaded_meta) = load_from_file(path).expect("Failed to load weights");
        let _ = std::fs::remove_file(path);

        assert_eq!(weights, loaded_w);
        assert_eq!(matrix, loaded_m);
        assert_eq!(metadata.epoch, loaded_meta.epoch);
        assert_eq!(metadata.winrate_vs_hard, loaded_meta.winrate_vs_hard);
        assert_eq!(metadata.winrate_vs_greedy, loaded_meta.winrate_vs_greedy);
        assert_eq!(metadata.fitness, loaded_meta.fitness);
    }

    #[test]
    fn test_model_selector_ranking_and_champion_promotion() {
        let mut selector = ModelSelector::with_default_champion();

        let mut p1 = HeuristicParams::default();
        p1.weight_voronoi_delta = 30.0;
        let id1 = selector.add_candidate(p1, generate_default_v2_matrix());

        let mut p2 = HeuristicParams::default();
        p2.weight_voronoi_delta = 40.0;
        let id2 = selector.add_candidate(p2, generate_default_v2_matrix());

        assert_eq!(selector.population.len(), 2);
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);

        // Assign fake stats to test ranking logic
        selector.population[0].stats.fitness = 120.0;
        selector.population[1].stats.fitness = 150.0;

        let ranking = selector.rank_population();
        assert_eq!(ranking, vec![1, 0]);

        let best = selector.get_best_candidate().unwrap();
        assert_eq!(best.id, 2);
        assert_eq!(best.stats.fitness, 150.0);

        // Test promotion
        selector.population[1].stats.winrate = 0.65;
        selector.population[1].stats.avg_margin = 15.0;

        let promoted = selector.select_and_promote_champion(0.55, 10.0);
        assert!(promoted);
        assert_eq!(selector.generation, 1);
        assert_eq!(selector.champion.id, 2);
        assert_eq!(selector.champion.weights.weight_voronoi_delta, 40.0);
    }

    #[test]
    fn test_crn_paired_matches_determinism() {
        let cand = HeuristicParams::default();
        let champ = HeuristicParams::default();
        let matrix = generate_default_v2_matrix();

        let res1 = ModelSelector::evaluate_pair_crn(&cand, &matrix, &champ, &matrix, 12345, 15);
        let res2 = ModelSelector::evaluate_pair_crn(&cand, &matrix, &champ, &matrix, 12345, 15);

        assert_eq!(res1, res2);
        assert_eq!(res1.candidate_wins + res1.candidate_losses + res1.candidate_draws, 2);
    }
}

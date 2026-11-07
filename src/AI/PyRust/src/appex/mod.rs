//! Appex adaptive AI module.
//!
//! Contains:
//! - `zoo`: 10 archetypes catalog and PhaseShiftBot combinator (Milestone 1).
//! - `profiler`: Opponent telemetry tracking and macro-cluster classification (Milestone 2).
//! - `model`: Adaptive weight blending and inference (Milestone 3).

pub mod zoo;
pub mod profiler;
pub mod model;

pub use zoo::*;
pub use profiler::*;
pub use model::*;

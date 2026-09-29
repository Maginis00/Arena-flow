//! The wave machine's data: the phase, the wave in play and its running stats.
//! [`super::transitions`] moves it between phases, [`super::measure`] fills the
//! stats.

use super::api::{WaveIndex, WaveSpec};
use crate::combat::api::ShotId;
use crate::flow_director::api::{Difficulty, WaveLevers};
use bevy::platform::collections::HashSet;
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Outcome {
    Cleared,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum WavePhase {
    /// Waiting for the director's first difficulty.
    Idle,
    Spawning,
    Active,
    /// Transient: report the wave, then enter intermission.
    Cleared(Outcome),
    Intermission {
        remaining_secs: f32,
    },
}

/// Facts counted while a wave is in play.
#[derive(Debug, Clone, Default)]
pub(super) struct WaveStats {
    pub(super) duration_secs: f32,
    pub(super) enemies_spawned: u32,
    pub(super) enemies_killed: u32,
    pub(super) damage_taken: u32,
    pub(super) player_died: bool,
    pub(super) shots_fired: u32,
    /// Distinct shots that hit something; a melee swing hitting three counts once.
    pub(super) shots_hit: HashSet<ShotId>,
    pub(super) pickups_collected: u32,
}

#[derive(Resource, Debug)]
pub(super) struct WaveMachine {
    pub(super) phase: WavePhase,
    pub(super) current: Option<WaveSpec>,
    pub(super) last_index: WaveIndex,
    pub(super) last_attempt: u32,
    /// The previous wave failed, so the next one repeats its index.
    pub(super) retry: bool,
    pub(super) stats: WaveStats,
    /// Latest director output; applied when the next wave starts.
    pub(super) next: Option<(Difficulty, WaveLevers)>,
    pub(super) player_max_hp: u32,
}

impl Default for WaveMachine {
    fn default() -> Self {
        Self {
            phase: WavePhase::Idle,
            current: None,
            last_index: WaveIndex(0),
            last_attempt: 0,
            retry: false,
            stats: WaveStats::default(),
            next: None,
            player_max_hp: 0,
        }
    }
}

impl WaveMachine {
    pub(super) fn in_play(&self) -> bool {
        matches!(self.phase, WavePhase::Spawning | WavePhase::Active)
    }
}

/// Saturating `usize` to `u32`, for counting messages and hit shots.
pub(super) fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

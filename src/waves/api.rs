use crate::flow_director::api::Difficulty;
use bevy::prelude::*;
use std::fmt;

/// 1-based wave number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct WaveIndex(pub u32);

impl WaveIndex {
    pub const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

impl fmt::Display for WaveIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Everything the enemies plugin needs to run one wave.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaveSpec {
    pub index: WaveIndex,
    pub difficulty: Difficulty,
    pub enemy_count: u32,
    pub enemy_speed: f32,
    pub contact_damage: u32,
}

/// A wave began spawning. The player respawns on this if dead.
#[derive(Message, Debug, Clone, Copy)]
pub struct WaveStarted {
    pub spec: WaveSpec,
}

/// Every spawned enemy of the wave was killed.
#[derive(Message, Debug, Clone, Copy)]
pub struct WaveCleared {
    pub index: WaveIndex,
}

/// The player died before the wave was cleared.
#[derive(Message, Debug, Clone, Copy)]
pub struct WaveFailed {
    pub index: WaveIndex,
}

/// Measured facts about one finished wave (cleared or failed).
/// Written once per wave, right after `WaveCleared` / `WaveFailed`.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct WaveReport {
    pub index: WaveIndex,
    pub difficulty: Difficulty,
    pub duration_secs: f32,
    pub enemies_spawned: u32,
    pub enemies_killed: u32,
    pub damage_taken: u32,
    pub player_max_hp: u32,
    pub player_died: bool,
    pub shots_fired: u32,
    pub shots_hit: u32,
}

impl WaveReport {
    pub fn cleared(&self) -> bool {
        !self.player_died && self.enemies_killed >= self.enemies_spawned
    }

    /// Fraction of max hp lost this wave, in `[0, 1]`. Zero if max hp is unknown.
    pub fn damage_fraction(&self) -> f32 {
        if self.player_max_hp == 0 {
            return 0.0;
        }
        (self.damage_taken as f32 / self.player_max_hp as f32).clamp(0.0, 1.0)
    }
}

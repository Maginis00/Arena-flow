//! Wave state machine: Idle -> Spawning -> Active -> Cleared -> Intermission -> Spawning ...
//!
//! A failed wave (player died) is retried: the next wave keeps the same index
//! with `attempt + 1`, at whatever difficulty the director picked.
//!
//! Runs entirely in `FixedUpdate` as a plain resource rather than Bevy `States`,
//! so transitions happen on simulation ticks instead of at frame boundaries.
//! `Cleared` covers both outcomes: the wave has left play, cleared or failed.
//!
//! The machine's data lives in `machine`, the counting in `measure`, how near
//! enemies came in `danger`, and the phase changes in `transitions`.

mod danger;
mod machine;
mod measure;
mod transitions;

use crate::app_setup::SimSet;
use crate::flow_director::Difficulty;
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
    /// 1 on the first try; a failed wave is retried with the same index.
    pub attempt: u32,
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
    pub attempt: u32,
    pub difficulty: Difficulty,
    pub duration_secs: f32,
    pub enemies_spawned: u32,
    pub enemies_killed: u32,
    pub damage_taken: u32,
    /// Contact hits the player took.
    pub hits_taken: u32,
    pub player_max_hp: u32,
    /// Player hp when the wave came into play: full after a cleared wave or a
    /// respawn, less if a pickup or the intermission changed it.
    pub start_hp: u32,
    /// Lowest player hp while the wave was in play; 0 if the player died.
    pub lowest_hp: u32,
    pub player_died: bool,
    pub shots_fired: u32,
    /// Shots that hit at least one enemy.
    pub shots_hit: u32,
    pub pickups_collected: u32,
    /// How near enemies came, whether or not they hit.
    pub danger: Danger,
}

/// How near enemies came to the player while the wave was in play, measured
/// every tick from positions. Damage only shows the approaches that landed;
/// this also counts the ones the player got away from.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Danger {
    /// Enemies that came within touching distance and left (killed or
    /// outrun) without a contact hit on that approach.
    pub close_calls: u32,
    /// Close calls weighted by how near each came: 1 for a graze, 0 at the
    /// edge of the close-call ring.
    pub close_call_weight: f32,
    /// Approaches that touched the player (a contact hit, even one the
    /// player's grace period absorbed).
    pub hit_approaches: u32,
    /// Seconds some enemy was less than a short reaction time from contact.
    pub threat_secs: f32,
    /// Seconds no enemy was within a few seconds of contact.
    pub calm_secs: f32,
    /// Most enemies at once within about a second of contact.
    pub peak_crowd: u32,
    /// Fewest seconds any enemy was from contact; `None` if none ever came
    /// within a few seconds.
    pub closest_secs: Option<f32>,
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

    /// Lowest hp in the wave as a fraction of max hp, in `[0, 1]`: how close
    /// the player came to dying. One if max hp is unknown.
    pub fn lowest_hp_fraction(&self) -> f32 {
        if self.player_max_hp == 0 {
            return 1.0;
        }
        (self.lowest_hp as f32 / self.player_max_hp as f32).clamp(0.0, 1.0)
    }
}

pub struct WavesPlugin;

impl Plugin for WavesPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<WaveStarted>()
            .add_message::<WaveCleared>()
            .add_message::<WaveFailed>()
            .add_message::<WaveReport>()
            .init_resource::<machine::WaveMachine>()
            .add_systems(
                FixedUpdate,
                (
                    measure::record_director,
                    measure::track_hp,
                    measure::measure,
                    danger::measure_danger,
                    transitions::advance,
                )
                    .chain()
                    .in_set(SimSet::Progress),
            );
    }
}

//! Wave state machine: Idle -> Spawning -> Active -> Cleared -> Intermission -> Spawning ...
//!
//! A failed wave (player died) is retried: the next wave keeps the same index
//! with `attempt + 1`, at whatever difficulty the director picked.
//!
//! Runs entirely in `FixedUpdate` as a plain resource rather than Bevy `States`,
//! so transitions happen on simulation ticks instead of at frame boundaries.
//! `Cleared` covers both outcomes: the wave has left play, cleared or failed.

pub mod api;

use crate::app_setup::api::SimSet;
use crate::combat::api::{EnemyKilled, Hit, HitSource, PlayerDamaged, PlayerDied, ShotId};
use crate::enemies::api::EnemySpawned;
use crate::flow_director::api::{Difficulty, DifficultyAdjusted, WaveLevers};
use crate::pickups::api::PickupCollected;
use crate::player::api::PlayerSpawned;
use crate::weapons::api::ShotFired;
use api::{WaveCleared, WaveFailed, WaveIndex, WaveReport, WaveSpec, WaveStarted};
use bevy::platform::collections::HashSet;
use bevy::prelude::*;

/// PLACEHOLDER: seconds between the end of a wave and the start of the next.
const INTERMISSION_SECS: f32 = 2.5;

pub struct WavesPlugin;

impl Plugin for WavesPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<WaveStarted>()
            .add_message::<WaveCleared>()
            .add_message::<WaveFailed>()
            .add_message::<WaveReport>()
            .init_resource::<WaveMachine>()
            .add_systems(
                FixedUpdate,
                (record_director, measure, advance)
                    .chain()
                    .in_set(SimSet::Progress),
            );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Cleared,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum WavePhase {
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
struct WaveStats {
    duration_secs: f32,
    enemies_spawned: u32,
    enemies_killed: u32,
    damage_taken: u32,
    player_died: bool,
    shots_fired: u32,
    /// Distinct shots that hit something; a melee swing hitting three counts once.
    shots_hit: HashSet<ShotId>,
    pickups_collected: u32,
}

#[derive(Resource, Debug)]
struct WaveMachine {
    phase: WavePhase,
    current: Option<WaveSpec>,
    last_index: WaveIndex,
    last_attempt: u32,
    /// The previous wave failed, so the next one repeats its index.
    retry: bool,
    stats: WaveStats,
    /// Latest director output; applied when the next wave starts.
    next: Option<(Difficulty, WaveLevers)>,
    player_max_hp: u32,
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
    fn in_play(&self) -> bool {
        matches!(self.phase, WavePhase::Spawning | WavePhase::Active)
    }
}

fn record_director(
    mut adjusted: MessageReader<DifficultyAdjusted>,
    mut spawned: MessageReader<PlayerSpawned>,
    mut machine: ResMut<WaveMachine>,
) {
    if let Some(latest) = adjusted.read().last() {
        machine.next = Some((latest.difficulty, latest.levers));
    }
    if let Some(latest) = spawned.read().last() {
        machine.player_max_hp = latest.max_hp;
    }
}

/// Count facts for the wave in play. Facts arriving outside play are dropped.
#[allow(clippy::too_many_arguments)] // one reader per measured fact
fn measure(
    time: Res<Time>,
    mut machine: ResMut<WaveMachine>,
    mut enemy_spawned: MessageReader<EnemySpawned>,
    mut enemy_killed: MessageReader<EnemyKilled>,
    mut player_damaged: MessageReader<PlayerDamaged>,
    mut player_died: MessageReader<PlayerDied>,
    mut shots: MessageReader<ShotFired>,
    mut hits: MessageReader<Hit>,
    mut pickups: MessageReader<PickupCollected>,
) {
    let spawned = count(enemy_spawned.read().count());
    let killed = count(enemy_killed.read().count());
    let damage: u32 = player_damaged.read().map(|d| d.amount).sum();
    let died = player_died.read().count() > 0;
    let fired = count(shots.read().count());
    let collected = count(pickups.read().count());
    let landed: Vec<ShotId> = hits
        .read()
        .filter_map(|hit| match hit.source {
            HitSource::Shot { shot, .. } => Some(shot),
            HitSource::Contact(_) => None,
        })
        .collect();
    if !machine.in_play() {
        return;
    }
    let stats = &mut machine.stats;
    stats.shots_hit.extend(landed);
    stats.duration_secs += time.delta_secs();
    stats.enemies_spawned = stats.enemies_spawned.saturating_add(spawned);
    stats.enemies_killed = stats.enemies_killed.saturating_add(killed);
    stats.damage_taken = stats.damage_taken.saturating_add(damage);
    stats.player_died |= died;
    stats.shots_fired = stats.shots_fired.saturating_add(fired);
    stats.pickups_collected = stats.pickups_collected.saturating_add(collected);
}

fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

fn advance(
    time: Res<Time>,
    mut machine: ResMut<WaveMachine>,
    mut started: MessageWriter<WaveStarted>,
    mut cleared: MessageWriter<WaveCleared>,
    mut failed: MessageWriter<WaveFailed>,
    mut reports: MessageWriter<WaveReport>,
) {
    match machine.phase {
        WavePhase::Idle => {
            if machine.next.is_some() {
                start_next_wave(&mut machine, &mut started);
            }
        }
        WavePhase::Spawning | WavePhase::Active => {
            let Some(spec) = machine.current else {
                // Unreachable by construction; recover by going back to Idle.
                machine.phase = WavePhase::Idle;
                return;
            };
            let stats = &machine.stats;
            machine.phase = if stats.player_died {
                WavePhase::Cleared(Outcome::Failed)
            } else if stats.enemies_spawned < spec.enemy_count {
                WavePhase::Spawning
            } else if stats.enemies_killed >= stats.enemies_spawned {
                WavePhase::Cleared(Outcome::Cleared)
            } else {
                WavePhase::Active
            };
        }
        WavePhase::Cleared(outcome) => {
            if let Some(spec) = machine.current.take() {
                match outcome {
                    Outcome::Cleared => {
                        cleared.write(WaveCleared { index: spec.index });
                    }
                    Outcome::Failed => {
                        failed.write(WaveFailed { index: spec.index });
                    }
                }
                machine.retry = outcome == Outcome::Failed;
                let s = &machine.stats;
                reports.write(WaveReport {
                    index: spec.index,
                    attempt: spec.attempt,
                    difficulty: spec.difficulty,
                    duration_secs: s.duration_secs,
                    enemies_spawned: s.enemies_spawned,
                    enemies_killed: s.enemies_killed,
                    damage_taken: s.damage_taken,
                    player_max_hp: machine.player_max_hp,
                    player_died: s.player_died,
                    shots_fired: s.shots_fired,
                    shots_hit: count(s.shots_hit.len()),
                    pickups_collected: s.pickups_collected,
                });
            }
            machine.phase = WavePhase::Intermission {
                remaining_secs: INTERMISSION_SECS,
            };
        }
        WavePhase::Intermission { remaining_secs } => {
            let remaining_secs = remaining_secs - time.delta_secs();
            if remaining_secs > 0.0 {
                machine.phase = WavePhase::Intermission { remaining_secs };
            } else {
                start_next_wave(&mut machine, &mut started);
            }
        }
    }
}

fn start_next_wave(machine: &mut WaveMachine, started: &mut MessageWriter<WaveStarted>) {
    let Some((difficulty, levers)) = machine.next else {
        machine.phase = WavePhase::Idle;
        return;
    };
    let (index, attempt) = if machine.retry {
        (machine.last_index, machine.last_attempt.saturating_add(1))
    } else {
        (machine.last_index.next(), 1)
    };
    let spec = WaveSpec {
        index,
        attempt,
        difficulty,
        enemy_count: levers.enemy_count,
        enemy_speed: levers.enemy_speed,
        contact_damage: levers.contact_damage,
    };
    machine.last_index = index;
    machine.last_attempt = attempt;
    machine.retry = false;
    machine.current = Some(spec);
    machine.stats = WaveStats::default();
    machine.phase = WavePhase::Spawning;
    started.write(WaveStarted { spec });
}

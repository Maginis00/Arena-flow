//! Moving the wave machine between phases and announcing every change.

use super::api::{WaveCleared, WaveFailed, WaveReport, WaveSpec, WaveStarted};
use super::machine::{Outcome, WaveMachine, WavePhase, WaveStats, count};
use bevy::prelude::*;

/// PLACEHOLDER: seconds between the end of a wave and the start of the next.
const INTERMISSION_SECS: f32 = 2.5;

pub(super) fn advance(
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
            } else if stats.queued_spawned < spec.enemy_count {
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
                reports.write(report(&machine, spec));
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

/// The finished wave's spec plus everything counted during it.
fn report(machine: &WaveMachine, spec: WaveSpec) -> WaveReport {
    let s = &machine.stats;
    WaveReport {
        index: spec.index,
        attempt: spec.attempt,
        difficulty: spec.difficulty,
        duration_secs: s.duration_secs,
        enemies_spawned: s.enemies_spawned,
        enemies_killed: s.enemies_killed,
        damage_taken: s.damage_taken,
        hits_taken: s.hits_taken,
        player_max_hp: machine.player_max_hp,
        start_hp: s.start_hp.unwrap_or(machine.player_hp),
        lowest_hp: if s.player_died {
            0
        } else {
            s.lowest_hp.unwrap_or(machine.player_hp)
        },
        player_died: s.player_died,
        shots_fired: s.shots_fired,
        shots_hit: count(s.shots_hit.len()),
        pickups_collected: s.pickups_collected,
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

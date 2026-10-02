//! Counting what happens during the wave in play, and remembering the latest
//! director output for the wave after it.

use super::machine::{WaveMachine, count};
use crate::combat::api::{
    EnemyKilled, Hit, HitSource, PlayerDamaged, PlayerDied, PlayerHealed, ShotId,
};
use crate::enemies::api::EnemySpawned;
use crate::flow_director::api::DifficultyAdjusted;
use crate::pickups::api::PickupCollected;
use crate::player::api::PlayerSpawned;
use crate::weapons::api::ShotFired;
use bevy::prelude::*;

pub(super) fn record_director(
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

/// Follow the player's hp in and out of play, and while a wave is in play
/// remember the hp it started at and the lowest it reached.
pub(super) fn track_hp(
    mut spawned: MessageReader<PlayerSpawned>,
    mut damaged: MessageReader<PlayerDamaged>,
    mut healed: MessageReader<PlayerHealed>,
    mut machine: ResMut<WaveMachine>,
) {
    if let Some(latest) = spawned.read().last() {
        machine.player_hp = latest.max_hp;
    }
    let before_damage = machine.player_hp;
    let mut lowest = before_damage;
    let mut hits = 0;
    for damage in damaged.read() {
        hits += 1;
        lowest = lowest.min(damage.remaining);
        machine.player_hp = damage.remaining;
    }
    if let Some(latest) = healed.read().last() {
        machine.player_hp = latest.remaining;
    }
    if !machine.in_play() {
        return;
    }
    let stats = &mut machine.stats;
    stats.hits_taken = stats.hits_taken.saturating_add(hits);
    let start = *stats.start_hp.get_or_insert(before_damage);
    stats.lowest_hp = Some(stats.lowest_hp.unwrap_or(start).min(lowest));
}

/// Count facts for the wave in play. Facts arriving outside play are dropped.
#[allow(clippy::too_many_arguments)] // one reader per measured fact
pub(super) fn measure(
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
            HitSource::Contact(_) | HitSource::Blast => None,
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

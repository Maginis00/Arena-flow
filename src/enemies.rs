//! Enemy spawning from a wave spec, chasing the player, and contact damage.

pub mod api;

use crate::app_setup::api::SimSet;
use crate::arena::api::ArenaBounds;
use crate::combat::api::{Health, Hit, HitSource, Hitbox, Team};
use crate::player::api::Player;
use crate::waves::api::{WaveCleared, WaveFailed, WaveSpec, WaveStarted};
use api::{Enemy, EnemySpawned};
use bevy::prelude::*;

/// PLACEHOLDER: seconds between enemy spawns within a wave.
const SPAWN_INTERVAL_SECS: f32 = 0.35;
/// PLACEHOLDER: enemy hit points (not a difficulty lever in v1).
const ENEMY_MAX_HP: u32 = 3;
/// PLACEHOLDER: enemy box half-size.
const ENEMY_HALF_SIZE: f32 = 12.0;
/// PLACEHOLDER: seconds between contact hits from one enemy.
const CONTACT_COOLDOWN_SECS: f32 = 0.75;
/// Spawn positions walk the perimeter by the golden ratio so consecutive
/// enemies are spread out deterministically (no RNG in the harness).
const SPAWN_STEP: f32 = 0.618_034;

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<EnemySpawned>()
            .init_resource::<SpawnQueue>()
            .add_systems(
                FixedUpdate,
                (queue_wave, spawn_from_queue).chain().in_set(SimSet::Spawn),
            )
            .add_systems(FixedUpdate, chase_player.in_set(SimSet::Movement))
            .add_systems(FixedUpdate, contact_damage.in_set(SimSet::Detect))
            .add_systems(FixedUpdate, clear_on_wave_end.in_set(SimSet::Cleanup));
    }
}

/// Enemies still to spawn for the current wave.
#[derive(Resource, Debug, Default)]
struct SpawnQueue {
    spec: Option<WaveSpec>,
    remaining: u32,
    until_next_secs: f32,
    /// Monotonic across waves so spawn points keep rotating.
    spawn_cursor: u32,
}

/// Per-enemy tuning copied from the wave spec at spawn time.
#[derive(Component, Debug, Clone, Copy)]
struct Chaser {
    speed: f32,
    contact_damage: u32,
}

/// Seconds until this enemy may deal contact damage again.
#[derive(Component, Debug, Clone, Copy, Default)]
struct ContactCooldown {
    remaining_secs: f32,
}

fn queue_wave(mut started: MessageReader<WaveStarted>, mut queue: ResMut<SpawnQueue>) {
    for started in started.read() {
        queue.spec = Some(started.spec);
        queue.remaining = started.spec.enemy_count;
        queue.until_next_secs = 0.0;
    }
}

fn spawn_from_queue(
    mut commands: Commands,
    time: Res<Time>,
    bounds: Res<ArenaBounds>,
    mut queue: ResMut<SpawnQueue>,
    mut spawned: MessageWriter<EnemySpawned>,
) {
    let Some(spec) = queue.spec else {
        return;
    };
    if queue.remaining == 0 {
        return;
    }
    queue.until_next_secs -= time.delta_secs();
    if queue.until_next_secs > 0.0 {
        return;
    }
    queue.until_next_secs += SPAWN_INTERVAL_SECS;
    queue.remaining -= 1;
    queue.spawn_cursor = queue.spawn_cursor.wrapping_add(1);

    // u32 -> f32 loses precision only past 2^24 spawns; fine for a spread pattern.
    let t = queue.spawn_cursor as f32 * SPAWN_STEP;
    let at = bounds.perimeter_point(t, ENEMY_HALF_SIZE * 2.0);
    let enemy = commands
        .spawn((
            Enemy,
            Team::Enemy,
            Health::full(ENEMY_MAX_HP),
            Hitbox::square(ENEMY_HALF_SIZE),
            Chaser {
                speed: spec.enemy_speed,
                contact_damage: spec.contact_damage,
            },
            ContactCooldown::default(),
            Transform::from_translation(at.extend(0.5)),
        ))
        .id();
    spawned.write(EnemySpawned { enemy });
}

fn chase_player(
    time: Res<Time>,
    player: Option<Single<&Transform, (With<Player>, Without<Enemy>)>>,
    mut enemies: Query<(&mut Transform, &Chaser), With<Enemy>>,
) {
    // No player (dead, awaiting respawn): enemies hold position.
    let Some(player) = player else {
        return;
    };
    let target = player.translation.truncate();
    let dt = time.delta_secs();
    for (mut transform, chaser) in &mut enemies {
        let here = transform.translation.truncate();
        let step = (target - here).normalize_or_zero() * chaser.speed * dt;
        transform.translation += step.extend(0.0);
    }
}

fn contact_damage(
    time: Res<Time>,
    player: Option<Single<(Entity, &Transform, &Hitbox), With<Player>>>,
    mut enemies: Query<(Entity, &Transform, &Hitbox, &Chaser, &mut ContactCooldown), With<Enemy>>,
    mut hits: MessageWriter<Hit>,
) {
    let dt = time.delta_secs();
    for (.., mut cooldown) in &mut enemies {
        cooldown.remaining_secs = (cooldown.remaining_secs - dt).max(0.0);
    }
    let Some(player) = player else {
        return;
    };
    let (player_entity, player_transform, player_box) = *player;
    let player_at = player_transform.translation.truncate();
    for (enemy, transform, hitbox, chaser, mut cooldown) in &mut enemies {
        if cooldown.remaining_secs > 0.0 {
            continue;
        }
        if hitbox.overlaps(transform.translation.truncate(), *player_box, player_at) {
            cooldown.remaining_secs = CONTACT_COOLDOWN_SECS;
            hits.write(Hit {
                target: player_entity,
                damage: chaser.contact_damage,
                source: HitSource::Contact(enemy),
            });
        }
    }
}

/// When a wave ends either way, remaining enemies and queued spawns go away.
fn clear_on_wave_end(
    mut commands: Commands,
    mut cleared: MessageReader<WaveCleared>,
    mut failed: MessageReader<WaveFailed>,
    mut queue: ResMut<SpawnQueue>,
    enemies: Query<Entity, With<Enemy>>,
) {
    let ended = cleared.read().count() + failed.read().count();
    if ended == 0 {
        return;
    }
    queue.spec = None;
    queue.remaining = 0;
    for enemy in &enemies {
        commands.entity(enemy).try_despawn();
    }
}

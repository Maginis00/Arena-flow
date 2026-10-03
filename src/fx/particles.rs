//! Cheap sprite particles: shards in the enemy's colour when it dies, sparks
//! on weapon hits, and a red screen flash when the player is hurt. No
//! randomness: directions are spread evenly and turned a little each burst.

use crate::combat::{EnemyKilled, Hit, HitSource, PlayerDamaged};
use crate::enemies::EnemyKind;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

/// PLACEHOLDER: shards per enemy death.
const KILL_SHARDS: u32 = 10;
/// PLACEHOLDER: shard speed range in world units per second.
const KILL_SPEED: (f32, f32) = (90.0, 220.0);
const KILL_SIZE: f32 = 6.0;
const KILL_SECS: f32 = 0.45;
/// PLACEHOLDER: sparks per weapon hit.
const HIT_SPARKS: u32 = 4;
const HIT_SPEED: f32 = 160.0;
const HIT_SIZE: f32 = 3.0;
const HIT_SECS: f32 = 0.15;
const HIT_COLOR: Color = Color::srgb(1.0, 0.95, 0.7);
/// Fallback when an enemy died before its colour was seen.
const ENEMY_FALLBACK_COLOR: Color = Color::srgb(0.9, 0.15, 0.15);
/// Particles slow down by this factor per second.
const DRAG_PER_SEC: f32 = 0.02;
/// Particles draw above the boxes.
const PARTICLE_Z: f32 = 5.0;
/// PLACEHOLDER: peak opacity and length of the red flash when the player is hurt.
const HURT_FLASH_ALPHA: f32 = 0.22;
const HURT_FLASH_SECS: f32 = 0.18;
/// Turns each burst's pattern so consecutive bursts don't look stamped.
const BURST_TURN: f32 = 0.7;

#[derive(Component, Debug)]
pub(super) struct Particle {
    velocity: Vec2,
    remaining_secs: f32,
    lifetime_secs: f32,
    size: f32,
}

/// The last colour seen on each living enemy, so its death burst matches it
/// even though the enemy is gone by the time the fact is read.
#[derive(Resource, Debug, Default)]
pub(super) struct EnemyColors(HashMap<Entity, Color>);

#[derive(Component, Debug)]
pub(super) struct HurtFlash {
    remaining_secs: f32,
}

pub(super) fn remember_enemy_colors(
    enemies: Query<(Entity, &Sprite), With<EnemyKind>>,
    mut colors: ResMut<EnemyColors>,
) {
    for (entity, sprite) in &enemies {
        colors.0.insert(entity, sprite.color);
    }
}

pub(super) fn burst_on_kill(
    mut commands: Commands,
    mut kills: MessageReader<EnemyKilled>,
    mut colors: ResMut<EnemyColors>,
    mut turn: Local<f32>,
) {
    for kill in kills.read() {
        let color = colors.0.remove(&kill.enemy).unwrap_or(ENEMY_FALLBACK_COLOR);
        *turn += BURST_TURN;
        for i in 0..KILL_SHARDS {
            let t = i as f32 / KILL_SHARDS as f32;
            // Alternate fast and slow shards so the burst has some depth.
            let speed = if i % 2 == 0 {
                KILL_SPEED.1
            } else {
                KILL_SPEED.0
            };
            let direction = Vec2::from_angle(*turn + t * std::f32::consts::TAU);
            spawn(
                &mut commands,
                kill.at,
                direction * speed,
                color,
                KILL_SIZE,
                KILL_SECS,
            );
        }
    }
    // Enemies that left without dying (wave reset) would otherwise linger.
    if colors.0.len() > 512 {
        colors.0.clear();
    }
}

pub(super) fn spark_on_hit(
    mut commands: Commands,
    mut hits: MessageReader<Hit>,
    targets: Query<&GlobalTransform>,
    mut turn: Local<f32>,
) {
    for hit in hits.read() {
        if !matches!(hit.source, HitSource::Shot { .. } | HitSource::Blast) {
            continue;
        }
        let Ok(target) = targets.get(hit.target) else {
            continue;
        };
        let at = target.translation().truncate();
        *turn += BURST_TURN;
        for i in 0..HIT_SPARKS {
            let angle = *turn + i as f32 / HIT_SPARKS as f32 * std::f32::consts::TAU;
            let velocity = Vec2::from_angle(angle) * HIT_SPEED;
            spawn(&mut commands, at, velocity, HIT_COLOR, HIT_SIZE, HIT_SECS);
        }
    }
}

fn spawn(commands: &mut Commands, at: Vec2, velocity: Vec2, color: Color, size: f32, secs: f32) {
    commands.spawn((
        Particle {
            velocity,
            remaining_secs: secs,
            lifetime_secs: secs,
            size,
        },
        Sprite::from_color(color, Vec2::splat(size)),
        Transform::from_translation(at.extend(PARTICLE_Z)),
    ));
}

pub(super) fn move_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Particle, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    let drag = DRAG_PER_SEC.powf(dt);
    for (entity, mut particle, mut transform, mut sprite) in &mut particles {
        particle.remaining_secs -= dt;
        if particle.remaining_secs <= 0.0 {
            commands.entity(entity).try_despawn();
            continue;
        }
        transform.translation += (particle.velocity * dt).extend(0.0);
        particle.velocity *= drag;
        let life = particle.remaining_secs / particle.lifetime_secs;
        sprite.custom_size = Some(Vec2::splat(particle.size * (0.4 + 0.6 * life)));
        sprite.color.set_alpha(life);
    }
}

pub(super) fn spawn_hurt_flash(mut commands: Commands) {
    commands.spawn((
        HurtFlash {
            remaining_secs: 0.0,
        },
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 0.0, 0.0, 0.0)),
        // Clicks go through to the game.
        Pickable::IGNORE,
    ));
}

pub(super) fn flash_on_player_hit(
    mut damaged: MessageReader<PlayerDamaged>,
    mut flash: Query<&mut HurtFlash>,
) {
    if damaged.read().count() == 0 {
        return;
    }
    for mut flash in &mut flash {
        flash.remaining_secs = HURT_FLASH_SECS;
    }
}

pub(super) fn fade_hurt_flash(
    time: Res<Time>,
    mut flash: Query<(&mut HurtFlash, &mut BackgroundColor)>,
) {
    for (mut flash, mut background) in &mut flash {
        flash.remaining_secs = (flash.remaining_secs - time.delta_secs()).max(0.0);
        let alpha = HURT_FLASH_ALPHA * flash.remaining_secs / HURT_FLASH_SECS;
        background.0.set_alpha(alpha);
    }
}

//! The two attacks beyond touching the player: shooters fire bolts, and
//! summoners call in grunts while they live.

use super::api::{Enemy, EnemyBolt, EnemyKind, EnemySpawned};
use super::kinds::scaled;
use super::spawning::{SpawnQueue, spawn_enemy};
use crate::arena::api::{ArenaBounds, DespawnOutsideArena};
use crate::combat::api::{Hit, HitSource, Hitbox};
use crate::player::api::Player;
use crate::waves::api::WaveSpec;
use bevy::prelude::*;

/// PLACEHOLDER: seconds between a shooter's bolts.
const SHOT_EVERY_SECS: f32 = 2.4;
/// PLACEHOLDER: seconds before a new shooter's first bolt.
const FIRST_SHOT_SECS: f32 = 1.5;
/// PLACEHOLDER: shooters only fire at a player this close.
const SHOT_RANGE: f32 = 480.0;
/// PLACEHOLDER: bolt speed in world units per second (the player moves 280).
const BOLT_SPEED: f32 = 300.0;
/// PLACEHOLDER: bolt box half-size.
const BOLT_HALF_SIZE: f32 = 6.0;
/// PLACEHOLDER: bolt damage as a multiple of the wave's contact damage.
const BOLT_DAMAGE_SCALE: f32 = 0.6;

/// PLACEHOLDER: seconds between a summoner's calls.
const SUMMON_EVERY_SECS: f32 = 3.0;
/// PLACEHOLDER: seconds before a new summoner's first call.
const FIRST_SUMMON_SECS: f32 = 1.5;
/// PLACEHOLDER: grunts one summoner keeps alive at most.
const MAX_MINIONS: usize = 3;
/// PLACEHOLDER: grunts one summoner calls in over its life. Keeps every wave
/// finite even for a player who never reaches the summoner.
const MAX_CALLS: u32 = 6;

/// Fires bolts at the player on a fixed rhythm.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct Gun {
    until_secs: f32,
    damage: u32,
}

/// Calls in a grunt on a fixed rhythm, up to [`MAX_MINIONS`] alive and
/// [`MAX_CALLS`] in all.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct Caller {
    until_secs: f32,
    calls: u32,
}

/// A grunt called in by the summoner `of`.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct Minion {
    of: Entity,
}

#[derive(Component, Debug, Clone, Copy)]
pub(super) struct BoltVelocity(Vec2);

/// Adds the attack components of `kind`, if it has any.
pub(super) fn arm(enemy: &mut EntityCommands, kind: EnemyKind, spec: &WaveSpec) {
    match kind {
        EnemyKind::Shooter => {
            enemy.insert(Gun {
                until_secs: FIRST_SHOT_SECS,
                damage: scaled(spec.contact_damage, BOLT_DAMAGE_SCALE),
            });
        }
        EnemyKind::Summoner => {
            enemy.insert(Caller {
                until_secs: FIRST_SUMMON_SECS,
                calls: 0,
            });
        }
        EnemyKind::Grunt | EnemyKind::Brute | EnemyKind::Charger => {}
    }
}

pub(super) fn shoot(
    mut commands: Commands,
    time: Res<Time>,
    player: Option<Single<&Transform, (With<Player>, Without<Enemy>)>>,
    mut shooters: Query<(&Transform, &mut Gun), With<Enemy>>,
) {
    let dt = time.delta_secs();
    let target = player.map(|p| p.translation.truncate());
    for (transform, mut gun) in &mut shooters {
        gun.until_secs = (gun.until_secs - dt).max(0.0);
        let Some(target) = target else {
            continue;
        };
        let here = transform.translation.truncate();
        if gun.until_secs > 0.0 || here.distance(target) > SHOT_RANGE {
            continue;
        }
        gun.until_secs = SHOT_EVERY_SECS;
        commands.spawn((
            EnemyBolt { damage: gun.damage },
            Hitbox::square(BOLT_HALF_SIZE),
            BoltVelocity((target - here).normalize_or(Vec2::X) * BOLT_SPEED),
            DespawnOutsideArena,
            Transform::from_translation(here.extend(1.0)),
        ));
    }
}

pub(super) fn move_bolts(time: Res<Time>, mut bolts: Query<(&mut Transform, &BoltVelocity)>) {
    let dt = time.delta_secs();
    for (mut transform, velocity) in &mut bolts {
        transform.translation += (velocity.0 * dt).extend(0.0);
    }
}

/// A bolt touching the player hits once and is gone.
pub(super) fn bolt_hits(
    mut commands: Commands,
    player: Option<Single<(Entity, &Transform, &Hitbox), With<Player>>>,
    bolts: Query<(Entity, &Transform, &Hitbox, &EnemyBolt)>,
    mut hits: MessageWriter<Hit>,
) {
    let Some(player) = player else {
        return;
    };
    let (player_entity, player_transform, player_box) = *player;
    let player_at = player_transform.translation.truncate();
    for (bolt, transform, hitbox, shot) in &bolts {
        if hitbox.overlaps(transform.translation.truncate(), *player_box, player_at) {
            hits.write(Hit {
                target: player_entity,
                damage: shot.damage,
                source: HitSource::EnemyShot(bolt),
            });
            commands.entity(bolt).try_despawn();
        }
    }
}

#[allow(clippy::too_many_arguments)] // one param per thing a summon reads or writes
pub(super) fn summon(
    mut commands: Commands,
    time: Res<Time>,
    bounds: Res<ArenaBounds>,
    queue: Res<SpawnQueue>,
    player: Option<Single<&Transform, (With<Player>, Without<Enemy>)>>,
    mut summoners: Query<(Entity, &Transform, &Hitbox, &mut Caller), With<Enemy>>,
    minions: Query<&Minion>,
    mut spawned: MessageWriter<EnemySpawned>,
) {
    let dt = time.delta_secs();
    // Summoners call only while a wave runs and the player is there to chase.
    let (Some(spec), Some(player)) = (queue.spec, player) else {
        return;
    };
    let target = player.translation.truncate();
    for (summoner, transform, hitbox, mut caller) in &mut summoners {
        caller.until_secs = (caller.until_secs - dt).max(0.0);
        if caller.until_secs > 0.0 || caller.calls >= MAX_CALLS {
            continue;
        }
        let alive = minions.iter().filter(|m| m.of == summoner).count();
        if alive >= MAX_MINIONS {
            continue;
        }
        caller.until_secs = SUMMON_EVERY_SECS;
        caller.calls += 1;
        let here = transform.translation.truncate();
        let ahead = (target - here).normalize_or(Vec2::X) * (hitbox.half_extents.x * 2.0);
        let at = bounds.clamp(here + ahead, hitbox.half_extents);
        let enemy = spawn_enemy(&mut commands, EnemyKind::Grunt, &spec, at, 1.0)
            .insert(Minion { of: summoner })
            .id();
        spawned.write(EnemySpawned {
            enemy,
            kind: EnemyKind::Grunt,
            summoned: true,
        });
    }
}

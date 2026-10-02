//! Enemy spawning from a wave spec (outside a safe radius around the player),
//! movement per kind, contact damage, and the ranged and summoning attacks.
//!
//! Which kinds a wave holds comes from [`EnemyMix`](api::EnemyMix); the grunt
//! mix is the original game.

pub mod api;
mod attacks;
mod kinds;
mod movement;
mod placement;
mod spawning;

use crate::app_setup::api::SimSet;
use crate::arena::api::ArenaBounds;
use crate::combat::api::{Hit, HitSource, Hitbox};
use crate::player::api::Player;
use crate::waves::api::{WaveCleared, WaveFailed};
use api::{ChargeTell, Enemy, EnemyBolt, EnemyMix, EnemySpawned};
use bevy::prelude::*;
use movement::Charge;

/// PLACEHOLDER: seconds between contact hits from one enemy.
const CONTACT_COOLDOWN_SECS: f32 = 0.75;

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        let mix = std::env::var(EnemyMix::ENV_VAR)
            .ok()
            .and_then(|name| name.parse().ok())
            .unwrap_or_default();
        app.add_message::<EnemySpawned>()
            .insert_resource::<EnemyMix>(mix)
            .init_resource::<spawning::SpawnQueue>()
            .add_systems(
                FixedUpdate,
                (
                    spawning::queue_wave,
                    spawning::spawn_from_queue,
                    attacks::summon,
                    attacks::shoot,
                )
                    .chain()
                    .in_set(SimSet::Spawn),
            )
            .add_systems(
                FixedUpdate,
                (move_enemies, attacks::move_bolts).in_set(SimSet::Movement),
            )
            .add_systems(
                FixedUpdate,
                (contact_damage, attacks::bolt_hits)
                    .chain()
                    .in_set(SimSet::Detect),
            )
            .add_systems(FixedUpdate, clear_on_wave_end.in_set(SimSet::Cleanup));
    }
}

/// Per-enemy tuning copied from the wave spec at spawn time.
#[derive(Component, Debug, Clone, Copy)]
struct Chaser {
    speed: f32,
    contact_damage: u32,
}

/// How an enemy moves; see [`movement`].
#[derive(Component, Debug, Clone, Copy)]
enum Gait {
    Chase,
    HoldRange { range: f32, orbit: f32 },
    Charge(Charge),
}

/// Seconds until this enemy may deal contact damage again.
#[derive(Component, Debug, Clone, Copy, Default)]
struct ContactCooldown {
    remaining_secs: f32,
}

fn move_enemies(
    time: Res<Time>,
    bounds: Res<ArenaBounds>,
    player: Option<Single<&Transform, (With<Player>, Without<Enemy>)>>,
    mut enemies: Query<
        (
            &mut Transform,
            &Hitbox,
            &Chaser,
            &mut Gait,
            Option<&mut ChargeTell>,
        ),
        With<Enemy>,
    >,
) {
    // No player (dead, awaiting respawn): enemies hold position.
    let Some(player) = player else {
        return;
    };
    let target = player.translation.truncate();
    let dt = time.delta_secs();
    for (mut transform, hitbox, chaser, mut gait, tell) in &mut enemies {
        let here = transform.translation.truncate();
        let to_player = target - here;
        let heading = match *gait {
            Gait::Chase => movement::chase(to_player),
            Gait::HoldRange { range, orbit } => movement::hold_range(to_player, range, orbit),
            Gait::Charge(charge) => {
                let (next, heading) = charge.step(to_player, dt);
                *gait = Gait::Charge(next);
                if let Some(mut tell) = tell {
                    // Only touch it on a change, so readers can use `Changed`.
                    tell.set_if_neq(ChargeTell { aim: next.tell() });
                }
                heading
            }
        };
        let next = here + heading * chaser.speed * dt;
        // Chasers head inward anyway; this keeps ranged kinds and dashes in.
        let next = match *gait {
            Gait::Chase => next,
            Gait::HoldRange { .. } | Gait::Charge(_) => bounds.clamp(next, hitbox.half_extents),
        };
        transform.translation = next.extend(transform.translation.z);
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

/// When a wave ends either way, remaining enemies, their bolts and queued
/// spawns go away.
fn clear_on_wave_end(
    mut commands: Commands,
    mut cleared: MessageReader<WaveCleared>,
    mut failed: MessageReader<WaveFailed>,
    mut queue: ResMut<spawning::SpawnQueue>,
    leftovers: Query<Entity, Or<(With<Enemy>, With<EnemyBolt>)>>,
) {
    let ended = cleared.read().count() + failed.read().count();
    if ended == 0 {
        return;
    }
    queue.spec = None;
    queue.remaining = 0;
    for entity in &leftovers {
        commands.entity(entity).try_despawn();
    }
}

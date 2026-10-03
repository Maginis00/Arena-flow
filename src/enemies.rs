//! Enemy spawning from a wave spec (outside a safe radius around the player),
//! movement per kind, contact damage, and the ranged and summoning attacks.
//!
//! Which kinds a wave holds comes from [`EnemyMix`]; the grunt
//! mix is the original game.

mod attacks;
mod kinds;
mod movement;
mod placement;
mod spawning;

pub use kinds::{EnemyKind, EnemyMix};

use crate::app_setup::SimSet;
use crate::arena::ArenaBounds;
use crate::combat::{Hit, HitSource, Hitbox};
use crate::player::Player;
use crate::waves::{WaveCleared, WaveFailed};
use bevy::prelude::*;
use movement::{Charge, Flock};

/// Marker for enemy entities.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Enemy;

/// What a charger shows: the direction it will dash in while it winds up,
/// `None` otherwise. Every charger carries one.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct ChargeTell {
    pub aim: Option<Vec2>,
}

/// What a swarm shows: true while the pack holds still before it dives at
/// the player. Every swarm member carries one.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct DiveTell {
    pub winding_up: bool,
}

/// A bolt fired by an enemy. Hurts only the player; player shots pass through it.
#[derive(Component, Debug, Clone, Copy)]
pub struct EnemyBolt {
    pub damage: u32,
}

/// An enemy from the current wave entered the arena.
#[derive(Message, Debug, Clone, Copy)]
pub struct EnemySpawned {
    pub enemy: Entity,
    pub kind: EnemyKind,
    /// Not taken from the wave's own count: called in by a summoner, or the
    /// rest of a swarm after its first member.
    pub extra: bool,
}

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
    HoldRange {
        range: f32,
        orbit: f32,
    },
    /// `cap` is the fastest its dash may go (see [`movement::dash_cap`]).
    Charge {
        charge: Charge,
        cap: f32,
    },
    /// `orbit` (+1 or -1) is the way the pack circles the player.
    Flock {
        flock: Flock,
        orbit: f32,
    },
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
            Option<&mut DiveTell>,
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
    for (mut transform, hitbox, chaser, mut gait, tell, dive_tell) in &mut enemies {
        let here = transform.translation.truncate();
        let to_player = target - here;
        let heading = match *gait {
            Gait::Chase => movement::chase(to_player),
            Gait::HoldRange { range, orbit } => movement::hold_range(to_player, range, orbit),
            Gait::Charge { charge, cap } => {
                let (next, heading) = charge.step(to_player, dt);
                *gait = Gait::Charge { charge: next, cap };
                if let Some(mut tell) = tell {
                    // Only touch it on a change, so readers can use `Changed`.
                    tell.set_if_neq(ChargeTell { aim: next.tell() });
                }
                heading
            }
            Gait::Flock { flock, orbit } => {
                let (next, heading) = flock.step(to_player, orbit, dt);
                *gait = Gait::Flock { flock: next, orbit };
                if let Some(mut tell) = dive_tell {
                    tell.set_if_neq(DiveTell {
                        winding_up: next.winding_up(),
                    });
                }
                heading
            }
        };
        let velocity = match *gait {
            Gait::Charge { cap, .. } => (heading * chaser.speed).clamp_length_max(cap),
            Gait::Chase | Gait::HoldRange { .. } | Gait::Flock { .. } => heading * chaser.speed,
        };
        let next = here + velocity * dt;
        // Chasers head inward anyway; this keeps ranged kinds, dashes and swarms in.
        let next = match *gait {
            Gait::Chase => next,
            Gait::HoldRange { .. } | Gait::Charge { .. } | Gait::Flock { .. } => {
                bounds.clamp(next, hitbox.half_extents)
            }
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

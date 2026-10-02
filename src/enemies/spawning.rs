//! Turning a wave spec into enemies: the spawn queue, which kind each slot
//! gets, where it appears, and everything a new enemy carries.

use super::api::{ChargeTell, Enemy, EnemyKind, EnemyMix, EnemySpawned};
use super::movement::{self, Charge};
use super::{Chaser, ContactCooldown, Gait, attacks, kinds, placement};
use crate::arena::api::ArenaBounds;
use crate::combat::api::{Health, Hitbox, Team};
use crate::player::api::Player;
use crate::waves::api::{WaveSpec, WaveStarted};
use bevy::prelude::*;

/// PLACEHOLDER: seconds between enemy spawns within a wave.
const SPAWN_INTERVAL_SECS: f32 = 0.35;
/// Spawn positions walk the perimeter by the golden ratio so consecutive
/// enemies are spread out deterministically (no RNG in the harness).
const SPAWN_STEP: f32 = 0.618_034;
/// PLACEHOLDER: enemies never appear closer than this to the player.
const SPAWN_SAFE_RADIUS: f32 = 200.0;
/// PLACEHOLDER: distance shooters hold from the player.
const SHOOTER_RANGE: f32 = 320.0;
/// PLACEHOLDER: distance summoners hold from the player.
const SUMMONER_RANGE: f32 = 380.0;
/// PLACEHOLDER: enemies in one swarm. Only the first counts as a wave slot.
const SWARM_SIZE: u32 = 6;
/// PLACEHOLDER: radius of the ring a swarm appears in.
const SWARM_SPREAD: f32 = 22.0;

/// Enemies still to spawn for the current wave.
#[derive(Resource, Debug, Default)]
pub(super) struct SpawnQueue {
    pub(super) spec: Option<WaveSpec>,
    pub(super) remaining: u32,
    /// Slots of this wave handed out so far; picks each spawn's kind.
    slot: u32,
    until_next_secs: f32,
    /// Monotonic across waves so spawn points keep rotating.
    spawn_cursor: u32,
}

/// Spawns an enemy of `kind` with everything it needs, tuned by the wave's
/// levers. `orbit` (+1 or -1) is the way a ranged kind circles the player.
pub(super) fn spawn_enemy<'a>(
    commands: &'a mut Commands,
    kind: EnemyKind,
    spec: &WaveSpec,
    at: Vec2,
    orbit: f32,
) -> EntityCommands<'a> {
    let stats = kinds::stats(kind);
    let gait = match kind {
        EnemyKind::Grunt | EnemyKind::Brute | EnemyKind::Swarm => Gait::Chase,
        EnemyKind::Shooter => Gait::HoldRange {
            range: SHOOTER_RANGE,
            orbit,
        },
        EnemyKind::Summoner => Gait::HoldRange {
            range: SUMMONER_RANGE,
            orbit,
        },
        EnemyKind::Charger => Gait::Charge {
            charge: Charge::default(),
            cap: movement::dash_cap(spec.difficulty.level()),
        },
    };
    let mut enemy = commands.spawn((
        (Enemy, kind, Team::Enemy),
        Health::full(stats.max_hp),
        Hitbox::square(stats.half_size),
        Chaser {
            speed: spec.enemy_speed * stats.speed_scale,
            contact_damage: kinds::scaled(spec.contact_damage, stats.contact_scale),
        },
        gait,
        ContactCooldown::default(),
        Transform::from_translation(at.extend(0.5)),
    ));
    if kind == EnemyKind::Charger {
        enemy.insert(ChargeTell::default());
    }
    attacks::arm(&mut enemy, kind, spec);
    enemy
}

pub(super) fn queue_wave(mut started: MessageReader<WaveStarted>, mut queue: ResMut<SpawnQueue>) {
    for started in started.read() {
        queue.spec = Some(started.spec);
        queue.remaining = started.spec.enemy_count;
        queue.slot = 0;
        queue.until_next_secs = 0.0;
    }
}

pub(super) fn spawn_from_queue(
    mut commands: Commands,
    time: Res<Time>,
    bounds: Res<ArenaBounds>,
    mix: Res<EnemyMix>,
    player: Option<Single<&Transform, With<Player>>>,
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
    queue.slot += 1;
    queue.spawn_cursor = queue.spawn_cursor.wrapping_add(1);

    let kind = kinds::kind_for_slot(*mix, queue.slot);
    let half_size = kinds::stats(kind).half_size;
    // u32 -> f32 loses precision only past 2^24 spawns; fine for a spread pattern.
    let t = queue.spawn_cursor as f32 * SPAWN_STEP;
    let candidate = bounds.perimeter_point(t, half_size * 2.0);
    let player_at = player.map(|p| p.translation.truncate());
    let at = placement::spawn_point(candidate, player_at, SPAWN_SAFE_RADIUS);
    let orbit = if queue.slot.is_multiple_of(2) {
        1.0
    } else {
        -1.0
    };
    let pack = if kind == EnemyKind::Swarm {
        SWARM_SIZE
    } else {
        1
    };
    for member in 0..pack {
        let at = bounds.clamp(at + pack_offset(member, pack), Vec2::splat(half_size));
        let enemy = spawn_enemy(&mut commands, kind, &spec, at, orbit).id();
        spawned.write(EnemySpawned {
            enemy,
            kind,
            extra: member > 0,
        });
    }
}

/// Where member `member` of a pack of `size` stands, relative to the pack's
/// spawn point: an even ring, or the point itself for a pack of one.
fn pack_offset(member: u32, size: u32) -> Vec2 {
    if size <= 1 {
        return Vec2::ZERO;
    }
    let angle = std::f32::consts::TAU * member as f32 / size as f32;
    Vec2::from_angle(angle) * SWARM_SPREAD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_lone_enemy_stands_on_its_spawn_point() {
        assert_eq!(pack_offset(0, 1), Vec2::ZERO);
    }

    #[test]
    fn a_pack_spreads_evenly_around_its_spawn_point() {
        let ring: Vec<Vec2> = (0..SWARM_SIZE)
            .map(|m| pack_offset(m, SWARM_SIZE))
            .collect();
        for offset in &ring {
            assert!((offset.length() - SWARM_SPREAD).abs() < 1e-3);
        }
        let centre = ring.iter().sum::<Vec2>() / SWARM_SIZE as f32;
        assert!(centre.length() < 1e-3, "{centre:?}");
    }
}

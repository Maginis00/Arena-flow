//! A simulated player that plays through the real game plugins: it moves by
//! holding `W` `A` `S` `D`, switches weapons with `1` `2` `3`, and aims and
//! fires by writing the same `FireRequested` the mouse would (and
//! `SwingRequested` for a sword on the right mouse button).

use super::choices::{
    choose_weapon, primary, reach, swing_target, wants_pickup, wants_shard, wants_to_spend,
};
use super::perception::{DelayedView, Rng, Snapshot};
use super::steering::{dodge, eight_way, nearest, rotate, sidestep, wall_push};
#[cfg(doc)]
use super::tier::SkillTier;
use super::tier::{PickupPolicy, TierParams};
use crate::app_setup::{FIXED_HZ, SimSet};
use crate::arena::ArenaBounds;
use crate::combat::Health;
use crate::enemies::{ChargeTell, Enemy, EnemyBolt, EnemyKind};
use crate::pickups::{PickupKind, Shard, ShardsChanged};
use crate::player::{FireRequested, Player, SwingRequested};
use crate::weapons::{SwordBinding, WeaponKind};
use bevy::prelude::*;

/// PLACEHOLDER steering weights shared by every tier.
const WALL_MARGIN: f32 = 140.0;
const WALL_WEIGHT: f32 = 1.2;
const PICKUP_PULL: f32 = 0.6;
/// Weighed pickers sidestep unwanted pickups this close.
const PICKUP_SIDESTEP_RADIUS: f32 = 40.0;
const CENTRE_DRIFT: f32 = 0.3;
/// Holding melee: dodge only enemies this close, walk in when all are this far.
const MELEE_DODGE_RADIUS: f32 = 45.0;
const MELEE_STANDOFF: f32 = 200.0;
const MELEE_PULL: f32 = 0.8;
/// Enemy bolts closer than this (capped by the tier's dodge radius) push the bot away.
const BOLT_DODGE_RADIUS: f32 = 120.0;
const BOLT_DODGE_WEIGHT: f32 = 1.5;
/// Each decision comes up to this share of `decision_secs` early or late, so
/// no tier plays like a metronome and seeds differ even when aim never misses.
const DECISION_JITTER: f32 = 0.3;
/// Bots that read tells step out of a dash lane this wide on either side.
const TELL_LANE: f32 = 50.0;
const TELL_WEIGHT: f32 = 2.0;

/// Plays the game with one tier's limits (see [`SkillTier::params`]).
/// `seed` varies aim wobble between runs.
pub struct PlaytestBotPlugin {
    pub params: TierParams,
    pub seed: u32,
}

impl Plugin for PlaytestBotPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Brain::new(self.params, self.seed))
            .add_systems(FixedUpdate, (perceive, act).chain().in_set(SimSet::Intent));
    }
}

#[derive(Resource, Debug)]
struct Brain {
    params: TierParams,
    view: DelayedView,
    rng: Rng,
    until_decision_secs: f32,
    aim_offset_rad: f32,
    weapon: WeaponKind,
    /// Shards held; a player always knows their own count, so not delayed.
    held_shards: u32,
}

impl Brain {
    fn new(params: TierParams, seed: u32) -> Self {
        // Rounding a positive, small reaction time to whole ticks.
        let delay_ticks = (params.reaction_secs * FIXED_HZ as f32).round().max(0.0) as usize;
        Self {
            params,
            view: DelayedView::new(delay_ticks),
            rng: Rng::new(seed),
            until_decision_secs: 0.0,
            aim_offset_rad: 0.0,
            weapon: WeaponKind::default(),
            held_shards: 0,
        }
    }
}

fn perceive(
    mut brain: ResMut<Brain>,
    enemies: Query<(&Transform, &EnemyKind), With<Enemy>>,
    bolts: Query<&Transform, With<EnemyBolt>>,
    tells: Query<(&Transform, &ChargeTell)>,
    pickups: Query<(&Transform, &PickupKind)>,
    shards: Query<&Transform, With<Shard>>,
    mut held: MessageReader<ShardsChanged>,
) {
    if let Some(latest) = held.read().last() {
        brain.held_shards = latest.held;
    }
    brain.view.push(Snapshot {
        enemies: enemies
            .iter()
            .map(|(t, _)| t.translation.truncate())
            .collect(),
        priority: enemies
            .iter()
            .filter(|(_, kind)| **kind == EnemyKind::Summoner)
            .map(|(t, _)| t.translation.truncate())
            .collect(),
        bolts: bolts.iter().map(|t| t.translation.truncate()).collect(),
        tells: tells
            .iter()
            .filter_map(|(t, tell)| tell.aim.map(|aim| (t.translation.truncate(), aim)))
            .collect(),
        pickups: pickups
            .iter()
            .map(|(t, kind)| (t.translation.truncate(), *kind))
            .collect(),
        shards: shards.iter().map(|t| t.translation.truncate()).collect(),
    });
}

#[allow(clippy::too_many_arguments)] // one param per thing the bot reads or presses
fn act(
    time: Res<Time>,
    bounds: Res<ArenaBounds>,
    sword: Res<SwordBinding>,
    mut brain: ResMut<Brain>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    player: Query<(Entity, &Transform, &Health), With<Player>>,
    mut fire: MessageWriter<FireRequested>,
    mut swing: MessageWriter<SwingRequested>,
) {
    let alive = player.iter().find(|(.., health)| !health.is_dead());
    let Some((shooter, transform, health)) = alive else {
        hold_keys(&mut keys, (0, 0));
        return;
    };
    let own = transform.translation.truncate();
    let brain = &mut *brain;
    let Some(seen) = brain.view.seen() else {
        return;
    };
    let p = brain.params;
    // Space is tapped, not held: one press per blast.
    keys.release(KeyCode::Space);

    brain.until_decision_secs -= time.delta_secs();
    if brain.until_decision_secs <= 0.0 {
        brain.until_decision_secs +=
            p.decision_secs * (1.0 + DECISION_JITTER * brain.rng.signed_unit());
        let hp = health.current() as f32 / health.max().max(1) as f32;
        let weapon = primary(choose_weapon(p.weapon, own, &seen.enemies), *sword);
        let wish = movement_wish(&p, weapon, own, seen, hp, bounds.half_extents());
        hold_keys(&mut keys, eight_way(wish));
        if weapon != brain.weapon {
            brain.weapon = weapon;
            let key = weapon_key(weapon);
            keys.release(key);
            keys.press(key);
        }
        brain.aim_offset_rad = brain.rng.wobble() * p.aim_error_deg.to_radians();
        if wants_to_spend(
            p.spend,
            brain.held_shards,
            hp,
            own,
            &seen.enemies,
            &seen.bolts,
        ) {
            keys.press(KeyCode::Space);
        }
    }

    if let Some(close) = swing_target(*sword, own, &seen.enemies) {
        let direction = rotate((close - own).normalize_or_zero(), brain.aim_offset_rad);
        if direction != Vec2::ZERO {
            swing.write(SwingRequested {
                shooter,
                origin: own,
                direction,
            });
        }
    }

    let priority = nearest(own, seen.priority.iter().copied())
        .filter(|t| p.focuses_priority && t.distance(own) <= reach(brain.weapon));
    let Some(target) = priority.or_else(|| nearest(own, seen.enemies.iter().copied())) else {
        return;
    };
    if p.trigger_discipline && target.distance(own) > reach(brain.weapon) {
        return;
    }
    let direction = rotate((target - own).normalize_or_zero(), brain.aim_offset_rad);
    if direction != Vec2::ZERO {
        fire.write(FireRequested {
            shooter,
            origin: own,
            direction,
        });
    }
}

fn movement_wish(
    p: &TierParams,
    weapon: WeaponKind,
    own: Vec2,
    seen: &Snapshot,
    hp: f32,
    half: Vec2,
) -> Vec2 {
    let threat = nearest(own, seen.enemies.iter().copied());
    let threat_distance = threat.map_or(f32::INFINITY, |t| t.distance(own));
    // With melee a player has to let enemies in: dodge only what is nearly
    // touching, and close the gap if everything is far away.
    // A bot that focuses priority targets walks right up to a summoner instead.
    let priority = nearest(own, seen.priority.iter().copied()).filter(|_| p.focuses_priority);
    let (dodge_radius, engage) = if weapon == WeaponKind::Melee {
        let engage = match priority {
            Some(target) => Some((target, reach(weapon) * 0.5)),
            None => threat.map(|target| (target, MELEE_STANDOFF)),
        };
        (p.dodge_radius.min(MELEE_DODGE_RADIUS), engage)
    } else {
        (p.dodge_radius, None)
    };
    let mut wish = dodge(own, &seen.enemies, dodge_radius, p.strafe);
    let bolt_radius = p.dodge_radius.min(BOLT_DODGE_RADIUS);
    wish += dodge(own, &seen.bolts, bolt_radius, p.strafe) * BOLT_DODGE_WEIGHT;
    if p.reads_tells {
        wish += sidestep(own, &seen.tells, TELL_LANE) * TELL_WEIGHT;
    }
    if let Some((target, _)) = engage.filter(|(t, standoff)| t.distance(own) > *standoff) {
        wish += (target - own).normalize_or_zero() * MELEE_PULL;
    }
    if p.avoids_walls {
        wish += wall_push(own, half, WALL_MARGIN) * WALL_WEIGHT;
    }
    let (wanted, unwanted): (Vec<_>, Vec<_>) = seen
        .pickups
        .iter()
        .partition(|(_, kind)| wants_pickup(p.pickups, *kind, hp, threat_distance));
    let reachable = wanted
        .iter()
        .map(|(at, _)| *at)
        .filter(|at| at.distance(own) <= p.pickup_reach);
    let shards = seen
        .shards
        .iter()
        .copied()
        .filter(|at| at.distance(own) <= p.pickup_reach)
        .filter(|_| wants_shard(p.pickups, threat_distance));
    if let Some(at) = nearest(own, reachable.chain(shards)) {
        wish += (at - own).normalize_or_zero() * PICKUP_PULL;
    }
    if p.pickups == PickupPolicy::Weighed {
        let close: Vec<Vec2> = unwanted.iter().map(|(at, _)| *at).collect();
        wish += dodge(own, &close, PICKUP_SIDESTEP_RADIUS, 0.0);
    }
    if wish.length() < 0.05 && p.avoids_walls {
        wish = -own.normalize_or_zero() * CENTRE_DRIFT * (own.length() / 100.0).min(1.0);
    }
    wish
}

fn hold_keys(keys: &mut ButtonInput<KeyCode>, (x, y): (i8, i8)) {
    for (key, held) in [
        (KeyCode::KeyD, x > 0),
        (KeyCode::KeyA, x < 0),
        (KeyCode::KeyW, y > 0),
        (KeyCode::KeyS, y < 0),
    ] {
        if held {
            keys.press(key);
        } else {
            keys.release(key);
        }
    }
}

const fn weapon_key(weapon: WeaponKind) -> KeyCode {
    match weapon {
        WeaponKind::Projectile => KeyCode::Digit1,
        WeaponKind::Hitscan => KeyCode::Digit2,
        WeaponKind::Melee => KeyCode::Digit3,
    }
}

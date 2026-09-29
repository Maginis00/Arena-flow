//! A simulated player that plays through the real game plugins: it moves by
//! holding `W` `A` `S` `D`, switches weapons with `1` `2` `3`, and aims and
//! fires by writing the same `FireRequested` the mouse would.

use super::choices::{choose_weapon, reach, wants_pickup};
use super::perception::{DelayedView, Rng, Snapshot};
use super::steering::{dodge, eight_way, nearest, rotate, wall_push};
use super::tier::{PickupPolicy, SkillTier, TierParams, WeaponPolicy};
use crate::app_setup::api::{FIXED_HZ, SimSet};
use crate::arena::api::ArenaBounds;
use crate::combat::api::Health;
use crate::enemies::api::Enemy;
use crate::pickups::api::PickupKind;
use crate::player::api::{FireRequested, Player};
use crate::weapons::api::WeaponKind;
use bevy::prelude::*;

/// PLACEHOLDER steering weights shared by every tier.
const WALL_MARGIN: f32 = 140.0;
const WALL_WEIGHT: f32 = 1.2;
const PICKUP_PULL: f32 = 0.6;
/// Weighed pickers sidestep unwanted pickups this close.
const PICKUP_SIDESTEP_RADIUS: f32 = 40.0;
const CENTRE_DRIFT: f32 = 0.3;

/// Plays the game at one skill tier. `seed` varies aim wobble between runs.
/// `weapon` pins one weapon instead of the tier's own weapon policy.
pub struct PlaytestBotPlugin {
    pub tier: SkillTier,
    pub seed: u32,
    pub weapon: Option<WeaponKind>,
}

impl Plugin for PlaytestBotPlugin {
    fn build(&self, app: &mut App) {
        let mut params = self.tier.params();
        if let Some(weapon) = self.weapon {
            params.weapon = WeaponPolicy::Fixed(weapon);
        }
        app.insert_resource(Brain::new(params, self.seed))
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
        }
    }
}

fn perceive(
    mut brain: ResMut<Brain>,
    enemies: Query<&Transform, With<Enemy>>,
    pickups: Query<(&Transform, &PickupKind)>,
) {
    brain.view.push(Snapshot {
        enemies: enemies.iter().map(|t| t.translation.truncate()).collect(),
        pickups: pickups
            .iter()
            .map(|(t, kind)| (t.translation.truncate(), *kind))
            .collect(),
    });
}

fn act(
    time: Res<Time>,
    bounds: Res<ArenaBounds>,
    mut brain: ResMut<Brain>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    player: Query<(Entity, &Transform, &Health), With<Player>>,
    mut fire: MessageWriter<FireRequested>,
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

    brain.until_decision_secs -= time.delta_secs();
    if brain.until_decision_secs <= 0.0 {
        brain.until_decision_secs += p.decision_secs;
        let hp = health.current() as f32 / health.max().max(1) as f32;
        let wish = movement_wish(&p, own, seen, hp, bounds.half_extents());
        hold_keys(&mut keys, eight_way(wish));
        let weapon = choose_weapon(p.weapon, own, &seen.enemies);
        if weapon != brain.weapon {
            brain.weapon = weapon;
            let key = weapon_key(weapon);
            keys.release(key);
            keys.press(key);
        }
        brain.aim_offset_rad = brain.rng.wobble() * p.aim_error_deg.to_radians();
    }

    let Some(target) = nearest(own, seen.enemies.iter().copied()) else {
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

fn movement_wish(p: &TierParams, own: Vec2, seen: &Snapshot, hp: f32, half: Vec2) -> Vec2 {
    let threat_distance =
        nearest(own, seen.enemies.iter().copied()).map_or(f32::INFINITY, |t| t.distance(own));
    let mut wish = dodge(own, &seen.enemies, p.dodge_radius, p.strafe);
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
    if let Some(at) = nearest(own, reachable) {
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

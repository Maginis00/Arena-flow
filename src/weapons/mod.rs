//! The three placeholder weapons: projectile, hitscan and melee arc. Only the
//! selected one fires. Weapons decide *that* a shot happens (cooldown, lock,
//! damage after pickup effects); combat decides *what it hits*.

pub mod api;

use crate::app_setup::api::SimSet;
use crate::arena::api::DespawnOutsideArena;
use crate::combat::api::{Hitbox, Projectile, ShotId, Team};
use crate::pickups::api::{Effects, EffectsChanged, scale_damage};
use crate::player::api::FireRequested;
use api::{Delivery, ShotFired, WeaponKind, WeaponSwitched};
use bevy::prelude::*;

/// PLACEHOLDER per-weapon stats.
struct WeaponStats {
    cooldown_secs: f32,
    damage: u32,
}

const fn stats(kind: WeaponKind) -> WeaponStats {
    match kind {
        // PLACEHOLDER: fast, weak, travels.
        WeaponKind::Projectile => WeaponStats {
            cooldown_secs: 0.18,
            damage: 1,
        },
        // PLACEHOLDER: slower, instant, first target only.
        WeaponKind::Hitscan => WeaponStats {
            cooldown_secs: 0.35,
            damage: 2,
        },
        // PLACEHOLDER: slowest, strong, short range, hits everything in the arc.
        WeaponKind::Melee => WeaponStats {
            cooldown_secs: 0.45,
            damage: 3,
        },
    }
}

/// PLACEHOLDER: projectile speed in world units per second.
const PROJECTILE_SPEED: f32 = 720.0;
/// PLACEHOLDER: projectile box half-size.
const PROJECTILE_HALF_SIZE: f32 = 4.0;
/// PLACEHOLDER: hitscan range in world units.
const HITSCAN_RANGE: f32 = 520.0;
/// PLACEHOLDER: melee reach in world units.
const MELEE_RADIUS: f32 = 75.0;
/// PLACEHOLDER: melee arc half-angle (60 degrees, so a 120 degree swing).
const MELEE_HALF_ANGLE: f32 = std::f32::consts::FRAC_PI_3;

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ShotFired>()
            .add_message::<WeaponSwitched>()
            .init_resource::<WeaponInput>()
            .init_resource::<Armory>()
            .add_systems(Startup, announce_initial)
            .add_systems(Update, sample_selection)
            .add_systems(
                FixedUpdate,
                (track_effects, apply_selection).in_set(SimSet::Intent),
            )
            .add_systems(FixedUpdate, fire.in_set(SimSet::Spawn))
            .add_systems(FixedUpdate, move_projectiles.in_set(SimSet::Movement));
    }
}

/// Weapon selection sampled in `Update`, applied in `FixedUpdate`.
#[derive(Resource, Debug, Default, Clone, Copy)]
struct WeaponInput {
    requested: Option<WeaponKind>,
}

/// One shooter in v1, so weapon state is a resource.
#[derive(Resource, Debug, Default, Clone, Copy)]
struct Armory {
    selected: WeaponKind,
    cooldown_remaining_secs: f32,
    next_shot: u32,
    effects: Effects,
}

#[derive(Component, Debug, Clone, Copy)]
struct Velocity(Vec2);

fn announce_initial(armory: Res<Armory>, mut switched: MessageWriter<WeaponSwitched>) {
    switched.write(WeaponSwitched {
        weapon: armory.selected,
    });
}

fn sample_selection(keys: Res<ButtonInput<KeyCode>>, mut input: ResMut<WeaponInput>) {
    let pressed = [
        (KeyCode::Digit1, WeaponKind::Projectile),
        (KeyCode::Digit2, WeaponKind::Hitscan),
        (KeyCode::Digit3, WeaponKind::Melee),
    ]
    .into_iter()
    .find_map(|(key, kind)| keys.just_pressed(key).then_some(kind));
    if pressed.is_some() {
        input.requested = pressed;
    }
}

fn track_effects(mut changed: MessageReader<EffectsChanged>, mut armory: ResMut<Armory>) {
    if let Some(latest) = changed.read().last() {
        armory.effects = latest.effects;
    }
}

fn apply_selection(
    mut input: ResMut<WeaponInput>,
    mut armory: ResMut<Armory>,
    mut switched: MessageWriter<WeaponSwitched>,
) {
    let Some(weapon) = input.requested.take() else {
        return;
    };
    if weapon != armory.selected {
        armory.selected = weapon;
        switched.write(WeaponSwitched { weapon });
    }
}

fn fire(
    mut commands: Commands,
    time: Res<Time>,
    mut armory: ResMut<Armory>,
    mut requests: MessageReader<FireRequested>,
    mut fired: MessageWriter<ShotFired>,
) {
    armory.cooldown_remaining_secs = (armory.cooldown_remaining_secs - time.delta_secs()).max(0.0);
    // Several requests in one tick still produce at most one shot.
    let Some(request) = requests.read().last().copied() else {
        return;
    };
    if armory.cooldown_remaining_secs > 0.0 || armory.effects.weapons_locked {
        return;
    }
    let weapon = armory.selected;
    let base = stats(weapon);
    // fire_rate is a positive multiplier by construction; guard anyway.
    armory.cooldown_remaining_secs = base.cooldown_secs / armory.effects.fire_rate.max(0.05);
    let damage = scale_damage(base.damage, armory.effects.outgoing_damage);
    let shot = ShotId(armory.next_shot);
    armory.next_shot = armory.next_shot.wrapping_add(1);

    let delivery = match weapon {
        WeaponKind::Projectile => {
            let entity = commands
                .spawn((
                    Projectile {
                        damage,
                        team: Team::Player,
                        shot,
                    },
                    Hitbox::square(PROJECTILE_HALF_SIZE),
                    Velocity(request.direction * PROJECTILE_SPEED),
                    DespawnOutsideArena,
                    Transform::from_translation(request.origin.extend(1.0)),
                ))
                .id();
            Delivery::Projectile { entity }
        }
        WeaponKind::Hitscan => Delivery::Hitscan {
            range: HITSCAN_RANGE,
            damage,
        },
        WeaponKind::Melee => Delivery::Melee {
            radius: MELEE_RADIUS,
            half_angle: MELEE_HALF_ANGLE,
            damage,
        },
    };
    fired.write(ShotFired {
        shot,
        shooter: request.shooter,
        team: Team::Player,
        weapon,
        origin: request.origin,
        direction: request.direction,
        delivery,
    });
}

fn move_projectiles(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    let dt = time.delta_secs();
    for (mut transform, velocity) in &mut query {
        transform.translation += (velocity.0 * dt).extend(0.0);
    }
}

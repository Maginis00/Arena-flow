//! The one placeholder weapon: hold to fire a straight projectile at a fixed
//! cooldown. The weapon fantasy is an open decision; this is a stand-in.

pub mod api;

use crate::app_setup::api::SimSet;
use crate::arena::api::DespawnOutsideArena;
use crate::combat::api::{Hitbox, Projectile, Team};
use crate::player::api::FireRequested;
use api::ShotFired;
use bevy::prelude::*;

/// PLACEHOLDER: seconds between shots while the fire button is held.
const FIRE_COOLDOWN_SECS: f32 = 0.18;
/// PLACEHOLDER: projectile speed in world units per second.
const PROJECTILE_SPEED: f32 = 720.0;
/// PLACEHOLDER: damage per projectile.
const PROJECTILE_DAMAGE: u32 = 1;
/// PLACEHOLDER: projectile box half-size.
const PROJECTILE_HALF_SIZE: f32 = 4.0;

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ShotFired>()
            .init_resource::<WeaponCooldown>()
            .add_systems(FixedUpdate, fire.in_set(SimSet::Spawn))
            .add_systems(FixedUpdate, move_projectiles.in_set(SimSet::Movement));
    }
}

/// Seconds until the weapon may fire again. One weapon, one shooter in v1.
#[derive(Resource, Debug, Default, Clone, Copy)]
struct WeaponCooldown {
    remaining_secs: f32,
}

#[derive(Component, Debug, Clone, Copy)]
struct Velocity(Vec2);

fn fire(
    mut commands: Commands,
    time: Res<Time>,
    mut cooldown: ResMut<WeaponCooldown>,
    mut requests: MessageReader<FireRequested>,
    mut fired: MessageWriter<ShotFired>,
) {
    cooldown.remaining_secs = (cooldown.remaining_secs - time.delta_secs()).max(0.0);
    // Several requests in one tick still produce at most one shot.
    let Some(request) = requests.read().last().copied() else {
        return;
    };
    if cooldown.remaining_secs > 0.0 {
        return;
    }
    cooldown.remaining_secs = FIRE_COOLDOWN_SECS;
    let projectile = commands
        .spawn((
            Projectile {
                damage: PROJECTILE_DAMAGE,
                team: Team::Player,
            },
            Hitbox::square(PROJECTILE_HALF_SIZE),
            Velocity(request.direction * PROJECTILE_SPEED),
            DespawnOutsideArena,
            Transform::from_translation(request.origin.extend(1.0)),
        ))
        .id();
    fired.write(ShotFired {
        shooter: request.shooter,
        projectile,
        origin: request.origin,
        direction: request.direction,
    });
}

fn move_projectiles(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    let dt = time.delta_secs();
    for (mut transform, velocity) in &mut query {
        transform.translation += (velocity.0 * dt).extend(0.0);
    }
}

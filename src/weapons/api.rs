use bevy::prelude::*;

/// A projectile left the weapon.
#[derive(Message, Debug, Clone, Copy)]
pub struct ShotFired {
    pub shooter: Entity,
    pub projectile: Entity,
    pub origin: Vec2,
    pub direction: Vec2,
}

use crate::combat::api::{ShotId, Team};
use bevy::prelude::*;
use std::fmt;

/// The three placeholder weapons. Selected with keys 1, 2, 3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeaponKind {
    #[default]
    Projectile,
    Hitscan,
    Melee,
}

impl fmt::Display for WeaponKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Projectile => "projectile",
            Self::Hitscan => "hitscan",
            Self::Melee => "melee arc",
        })
    }
}

/// How a shot reaches its targets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Delivery {
    /// A projectile entity was spawned; combat resolves it on overlap.
    Projectile { entity: Entity },
    /// Instant ray; combat hits the first target along it.
    Hitscan { range: f32, damage: u32 },
    /// Instant arc centred on the aim direction; combat hits every target inside.
    Melee {
        radius: f32,
        half_angle: f32,
        damage: u32,
    },
}

/// A weapon fired (one trigger pull).
#[derive(Message, Debug, Clone, Copy)]
pub struct ShotFired {
    pub shot: ShotId,
    pub shooter: Entity,
    pub team: Team,
    pub weapon: WeaponKind,
    pub origin: Vec2,
    /// Unit vector.
    pub direction: Vec2,
    pub delivery: Delivery,
}

/// The selected weapon changed (also written once at startup).
#[derive(Message, Debug, Clone, Copy)]
pub struct WeaponSwitched {
    pub weapon: WeaponKind,
}

use crate::combat::api::{ShotId, Team};
use bevy::prelude::*;
use std::fmt;
use std::str::FromStr;

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

/// Where the sword lives. A prototype: `Key3` is the game as it was.
///
/// Set it by inserting the resource, or for the game window with the
/// `ARENA_SWORD` environment variable (`key3` or `right`).
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SwordBinding {
    /// One of three weapons, selected with `3`.
    #[default]
    Key3,
    /// Always at hand on the right mouse button with its own cooldown, next
    /// to the gun; `1` and `2` pick the gun and `3` does nothing.
    RightClick,
}

impl SwordBinding {
    pub const ENV_VAR: &str = "ARENA_SWORD";
}

impl fmt::Display for SwordBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Key3 => "key3",
            Self::RightClick => "right",
        })
    }
}

impl FromStr for SwordBinding {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "key3" | "3" => Ok(Self::Key3),
            "right" | "rightclick" | "right-click" => Ok(Self::RightClick),
            _ => Err(format!(
                "unknown sword binding {s:?}; expected key3 or right"
            )),
        }
    }
}

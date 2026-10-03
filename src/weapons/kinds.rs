//! Which weapons exist and where the sword sits.

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

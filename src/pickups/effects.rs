//! What each pickup does and the combined effects the player carries.

use bevy::prelude::*;
use std::fmt;

/// Every pickup has an upside and a downside. All numbers are PLACEHOLDER.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PickupKind {
    /// Faster fire rate, but you take more damage.
    Overdrive,
    /// Double damage, but you move slower.
    Heavy,
    /// Instant heal, but your weapon is locked for a moment.
    Mend,
}

impl PickupKind {
    pub const ALL: [Self; 3] = [Self::Overdrive, Self::Heavy, Self::Mend];
}

impl fmt::Display for PickupKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Overdrive => "overdrive",
            Self::Heavy => "heavy",
            Self::Mend => "mend",
        })
    }
}

/// Combined modifiers from every active pickup. Multipliers start at 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Effects {
    pub move_speed: f32,
    pub fire_rate: f32,
    pub outgoing_damage: f32,
    pub incoming_damage: f32,
    pub weapons_locked: bool,
}

impl Effects {
    pub const NEUTRAL: Self = Self {
        move_speed: 1.0,
        fire_rate: 1.0,
        outgoing_damage: 1.0,
        incoming_damage: 1.0,
        weapons_locked: false,
    };

    /// Multiply two effect sets; any lock locks.
    pub fn combine(self, other: Self) -> Self {
        Self {
            move_speed: self.move_speed * other.move_speed,
            fire_rate: self.fire_rate * other.fire_rate,
            outgoing_damage: self.outgoing_damage * other.outgoing_damage,
            incoming_damage: self.incoming_damage * other.incoming_damage,
            weapons_locked: self.weapons_locked || other.weapons_locked,
        }
    }
}

impl Default for Effects {
    fn default() -> Self {
        Self::NEUTRAL
    }
}

/// Scale integer damage by a multiplier, rounding to nearest. A non-zero hit
/// never rounds down to zero.
pub fn scale_damage(base: u32, multiplier: f32) -> u32 {
    if base == 0 {
        return 0;
    }
    let scaled = (base as f32 * multiplier.max(0.0)).round();
    // Saturating float-to-int cast; values here are small.
    (scaled as u32).max(1)
}

/// The combined pickup effects changed (pickup taken, expired, or cleared on death).
#[derive(Message, Debug, Clone, Copy)]
pub struct EffectsChanged {
    pub effects: Effects,
}

/// The player picked something up.
#[derive(Message, Debug, Clone, Copy)]
pub struct PickupCollected {
    pub kind: PickupKind,
}

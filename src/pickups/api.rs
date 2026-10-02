use bevy::prelude::*;
use std::fmt;
use std::str::FromStr;

/// Which pickup rules a run plays with. `Classic` is the game as it was; the
/// others are prototypes behind `--pickups` / `ARENA_PICKUPS`.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PickupRules {
    /// Timed buffs drop where the enemy died and carry into the next wave.
    #[default]
    Classic,
    /// The same buffs, but thrown away from the player, short-lived, and gone
    /// when the wave ends: taking one costs a detour.
    Far,
    /// No buffs. Kills drop shards; held shards raise fire rate, spending them
    /// all (Space) blasts everything nearby. Death loses them.
    Shards,
    /// Like `Shards`, but kills close to the player drop nothing.
    ShardsRange,
}

impl PickupRules {
    pub const ALL: [Self; 4] = [Self::Classic, Self::Far, Self::Shards, Self::ShardsRange];

    /// Read from `ARENA_PICKUPS`; unset or unknown is `Classic`.
    pub fn from_env() -> Self {
        std::env::var("ARENA_PICKUPS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or_default()
    }

    pub const fn uses_shards(self) -> bool {
        matches!(self, Self::Shards | Self::ShardsRange)
    }
}

impl fmt::Display for PickupRules {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Classic => "classic",
            Self::Far => "far",
            Self::Shards => "shards",
            Self::ShardsRange => "shards-range",
        })
    }
}

impl FromStr for PickupRules {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|rules| rules.to_string().eq_ignore_ascii_case(s))
            .ok_or_else(|| {
                format!("unknown pickup rules {s:?}; expected classic, far, shards or shards-range")
            })
    }
}

/// A shard on the floor (`Shards` rules). Walk over it to hold it.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shard;

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

/// The player now holds this many shards (picked one up, spent them, or died).
#[derive(Message, Debug, Clone, Copy)]
pub struct ShardsChanged {
    pub held: u32,
}

/// The player spent shards on a blast.
#[derive(Message, Debug, Clone, Copy)]
pub struct ShardsSpent {
    pub count: u32,
    pub at: Vec2,
    pub radius: f32,
}

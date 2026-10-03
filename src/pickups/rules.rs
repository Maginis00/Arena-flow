//! Which pickup rule set a run plays with.

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
    /// Like `Shards`, but the blast also destroys enemy bolts inside it, so
    /// spending answers ranged enemies too.
    ShardsBolts,
}

impl PickupRules {
    pub const ALL: [Self; 5] = [
        Self::Classic,
        Self::Far,
        Self::Shards,
        Self::ShardsRange,
        Self::ShardsBolts,
    ];

    /// Read from `ARENA_PICKUPS`; unset or unknown is `Classic`.
    pub fn from_env() -> Self {
        std::env::var("ARENA_PICKUPS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or_default()
    }

    pub const fn uses_shards(self) -> bool {
        matches!(self, Self::Shards | Self::ShardsRange | Self::ShardsBolts)
    }
}

impl fmt::Display for PickupRules {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Classic => "classic",
            Self::Far => "far",
            Self::Shards => "shards",
            Self::ShardsRange => "shards-range",
            Self::ShardsBolts => "shards-bolts",
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
                format!("unknown pickup rules {s:?}; expected classic, far, shards, shards-range or shards-bolts")
            })
    }
}

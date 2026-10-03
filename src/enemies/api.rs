use bevy::prelude::*;
use std::fmt;
use std::str::FromStr;

/// Marker for enemy entities.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Enemy;

/// What an enemy is. Every enemy entity carries one.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EnemyKind {
    /// Weak, rushes straight in. The original enemy.
    #[default]
    Grunt,
    /// Keeps its distance and fires slow, dodgeable bolts.
    Shooter,
    /// Slow, tough and hits hard.
    Brute,
    /// Closes in, stops to wind up, then dashes in a straight line.
    Charger,
    /// Hangs back and keeps calling in grunts while it lives.
    Summoner,
    /// Tiny, fragile and fast; comes as a pack from one spot on the wall.
    Swarm,
}

impl EnemyKind {
    pub const ALL: [Self; 6] = [
        Self::Grunt,
        Self::Shooter,
        Self::Brute,
        Self::Charger,
        Self::Summoner,
        Self::Swarm,
    ];
}

impl fmt::Display for EnemyKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Grunt => "grunt",
            Self::Shooter => "shooter",
            Self::Brute => "brute",
            Self::Charger => "charger",
            Self::Summoner => "summoner",
            Self::Swarm => "swarm",
        })
    }
}

/// Which enemy kinds a wave is made of. Every wave walks the same
/// deterministic slot pattern, so a mix is repeatable (no RNG).
///
/// Set it by inserting the resource, or for the game window with the
/// `ARENA_ENEMIES` environment variable (`grunt`, a kind name, or `all`).
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnemyMix {
    /// Only grunts: the game as it was before other kinds existed.
    #[default]
    Grunts,
    /// Grunts with one other kind mixed in.
    With(EnemyKind),
    /// Every kind.
    All,
}

impl EnemyMix {
    pub const ENV_VAR: &str = "ARENA_ENEMIES";
}

impl fmt::Display for EnemyMix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Grunts => f.write_str("grunts"),
            Self::With(kind) => write!(f, "grunts + {kind}"),
            Self::All => f.write_str("all"),
        }
    }
}

impl FromStr for EnemyMix {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let name = s.trim().to_ascii_lowercase();
        if name == "all" {
            return Ok(Self::All);
        }
        if name == "grunt" || name == "grunts" {
            return Ok(Self::Grunts);
        }
        EnemyKind::ALL
            .into_iter()
            .find(|kind| kind.to_string() == name)
            .map(Self::With)
            .ok_or_else(|| {
                format!(
                    "unknown enemy mix {s:?}; expected grunt, shooter, brute, charger, summoner, swarm or all"
                )
            })
    }
}

/// What a charger shows: the direction it will dash in while it winds up,
/// `None` otherwise. Every charger carries one.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct ChargeTell {
    pub aim: Option<Vec2>,
}

/// A bolt fired by an enemy. Hurts only the player; player shots pass through it.
#[derive(Component, Debug, Clone, Copy)]
pub struct EnemyBolt {
    pub damage: u32,
}

/// An enemy from the current wave entered the arena.
#[derive(Message, Debug, Clone, Copy)]
pub struct EnemySpawned {
    pub enemy: Entity,
    pub kind: EnemyKind,
    /// Not taken from the wave's own count: called in by a summoner, or the
    /// rest of a swarm after its first member.
    pub extra: bool,
}

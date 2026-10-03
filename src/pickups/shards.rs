//! Shard rules (prototype): the shard on the floor, the shard facts, what
//! holding shards gives and what spending them does. Pure logic, unit-tested.
//!
//! The trade-off is the one Devil Daggers makes with gems and homing daggers:
//! the same resource is a passive bonus while held and a burst when spent, so
//! every blast costs the bonus you built up.

use super::Effects;
use bevy::prelude::*;

/// A shard on the floor (`Shards` rules). Walk over it to hold it.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shard;

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

/// PLACEHOLDER: one shard drops every this many kills.
pub const KILLS_PER_SHARD: u32 = 2;
/// PLACEHOLDER: seconds a shard stays on the floor.
pub const SHARD_LIFETIME_SECS: f32 = 6.0;
/// PLACEHOLDER: shard box half-size.
pub const SHARD_HALF_SIZE: f32 = 5.0;
/// PLACEHOLDER: most shards a player can hold.
pub const MAX_HELD: u32 = 10;
/// PLACEHOLDER: fire rate gained per held shard (10 shards = +30%).
const FIRE_RATE_PER_SHARD: f32 = 0.03;
/// PLACEHOLDER: fewest shards a blast needs.
pub const MIN_SPEND: u32 = 3;
/// PLACEHOLDER: blast radius with no shards, and added per shard spent.
const BLAST_BASE_RADIUS: f32 = 110.0;
const BLAST_RADIUS_PER_SHARD: f32 = 15.0;
/// PLACEHOLDER: blast damage to every enemy inside (a grunt has 3 hp).
pub const BLAST_DAMAGE: u32 = 3;
/// PLACEHOLDER: under `ShardsRange`, kills closer than this to the player drop
/// nothing. Larger than the sword's reach, so sword kills never pay.
pub const RANGE_MIN_KILL_DISTANCE: f32 = 120.0;

/// The passive bonus of holding `held` shards.
pub fn held_effects(held: u32) -> Effects {
    Effects {
        fire_rate: 1.0 + FIRE_RATE_PER_SHARD * held.min(MAX_HELD) as f32,
        ..Effects::NEUTRAL
    }
}

/// Blast radius when spending `spent` shards, or `None` if too few.
pub fn blast_radius(spent: u32) -> Option<f32> {
    (spent >= MIN_SPEND).then_some(BLAST_BASE_RADIUS + BLAST_RADIUS_PER_SHARD * spent as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holding_shards_speeds_up_fire_up_to_the_cap() {
        assert_eq!(held_effects(0), Effects::NEUTRAL);
        assert!((held_effects(5).fire_rate - 1.15).abs() < 1e-6);
        assert_eq!(held_effects(MAX_HELD), held_effects(MAX_HELD + 5));
    }

    #[test]
    fn a_blast_needs_enough_shards_and_grows_with_them() {
        assert_eq!(blast_radius(MIN_SPEND - 1), None);
        let small = blast_radius(MIN_SPEND).expect("enough shards");
        let big = blast_radius(MAX_HELD).expect("enough shards");
        assert!(big > small);
    }
}

//! Skill tiers for simulated players. Every number here is a PLACEHOLDER
//! proposal: a tier is a bundle of measurable human limits, not a strategy.

use crate::weapons::api::WeaponKind;
use std::fmt;
use std::str::FromStr;

/// Which pickups a bot walks to. Pickups it walks over by accident are still
/// collected, exactly as for a human.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickupPolicy {
    /// Never goes for a pickup.
    Ignore,
    /// Goes for any pickup within reach, without weighing the downside.
    Greedy,
    /// Weighs each pickup's downside against the current situation.
    Weighed,
}

/// How a bot picks its weapon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponPolicy {
    /// Keeps one weapon the whole run.
    Fixed(WeaponKind),
    /// Melee when a crowd is close, hitscan otherwise.
    Situational,
}

/// Measurable limits of one simulated player.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TierParams {
    /// Delay between the world changing and the bot acting on it.
    pub reaction_secs: f32,
    /// Seconds between movement and aim decisions (how often "the hands" update).
    pub decision_secs: f32,
    /// Largest aim error in degrees; the actual error is resampled every
    /// decision, clustered near zero.
    pub aim_error_deg: f32,
    /// Enemies closer than this push the bot away. Zero: never dodges.
    pub dodge_radius: f32,
    /// Strength of circling around the threat instead of backing straight off.
    pub strafe: f32,
    /// Whether the bot steers away from walls and corners.
    pub avoids_walls: bool,
    /// Only fires when the target is within the weapon's reach.
    pub trigger_discipline: bool,
    /// Shoots a summoner in reach before the nearest enemy.
    pub focuses_priority: bool,
    pub pickups: PickupPolicy,
    /// How far the bot will walk for a pickup.
    pub pickup_reach: f32,
    pub weapon: WeaponPolicy,
}

/// The proposed tiers, from least to most skilled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkillTier {
    Novice,
    Casual,
    Skilled,
    Expert,
}

impl SkillTier {
    pub const ALL: [Self; 4] = [Self::Novice, Self::Casual, Self::Skilled, Self::Expert];

    /// PLACEHOLDER parameters for every tier.
    pub const fn params(self) -> TierParams {
        match self {
            Self::Novice => TierParams {
                reaction_secs: 0.45,
                decision_secs: 0.30,
                aim_error_deg: 14.0,
                dodge_radius: 70.0,
                strafe: 0.0,
                avoids_walls: false,
                trigger_discipline: false,
                focuses_priority: false,
                pickups: PickupPolicy::Ignore,
                pickup_reach: 0.0,
                weapon: WeaponPolicy::Fixed(WeaponKind::Projectile),
            },
            Self::Casual => TierParams {
                reaction_secs: 0.30,
                decision_secs: 0.20,
                aim_error_deg: 8.0,
                dodge_radius: 130.0,
                strafe: 0.3,
                avoids_walls: false,
                trigger_discipline: false,
                focuses_priority: false,
                pickups: PickupPolicy::Greedy,
                pickup_reach: 200.0,
                weapon: WeaponPolicy::Fixed(WeaponKind::Projectile),
            },
            Self::Skilled => TierParams {
                reaction_secs: 0.20,
                decision_secs: 0.10,
                aim_error_deg: 4.0,
                dodge_radius: 180.0,
                strafe: 0.7,
                avoids_walls: true,
                trigger_discipline: true,
                focuses_priority: true,
                pickups: PickupPolicy::Weighed,
                pickup_reach: 300.0,
                weapon: WeaponPolicy::Fixed(WeaponKind::Hitscan),
            },
            Self::Expert => TierParams {
                reaction_secs: 0.12,
                decision_secs: 0.05,
                aim_error_deg: 1.5,
                dodge_radius: 220.0,
                strafe: 1.0,
                avoids_walls: true,
                trigger_discipline: true,
                focuses_priority: true,
                pickups: PickupPolicy::Weighed,
                pickup_reach: 400.0,
                weapon: WeaponPolicy::Situational,
            },
        }
    }
}

impl fmt::Display for SkillTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Novice => "novice",
            Self::Casual => "casual",
            Self::Skilled => "skilled",
            Self::Expert => "expert",
        })
    }
}

impl FromStr for SkillTier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|tier| tier.to_string().eq_ignore_ascii_case(s))
            .ok_or_else(|| {
                format!("unknown tier {s:?}; expected novice, casual, skilled or expert")
            })
    }
}

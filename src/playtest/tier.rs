//! Skill tiers for simulated players. Every number here is a PLACEHOLDER
//! proposal: a tier is a bundle of measurable human limits, not a strategy.

use crate::weapons::WeaponKind;
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
    /// Goes for any pickup or shard only while no enemy is within this many
    /// units. A tunable rule for strategy searches.
    Clear(u16),
}

impl fmt::Display for PickupPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ignore => f.write_str("ignore"),
            Self::Greedy => f.write_str("greedy"),
            Self::Weighed => f.write_str("weighed"),
            Self::Clear(radius) => write!(f, "clear:{radius}"),
        }
    }
}

impl FromStr for PickupPolicy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_lowercase();
        match s.as_str() {
            "ignore" => return Ok(Self::Ignore),
            "greedy" => return Ok(Self::Greedy),
            "weighed" => return Ok(Self::Weighed),
            _ => {}
        }
        s.strip_prefix("clear:")
            .and_then(|r| r.parse().ok())
            .map(Self::Clear)
            .ok_or_else(|| {
                format!(
                    "unknown pickup policy {s:?}; expected ignore, greedy, weighed or clear:RADIUS"
                )
            })
    }
}

/// When a bot spends its shards on a blast (shard pickup rules only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpendPolicy {
    /// Never spends: keeps the fire-rate bonus.
    Hoard,
    /// Spends once at least `crowd` threats are in blast reach, or once hp is
    /// below `low_hp_pct` percent with anything in reach. Threats are enemies,
    /// plus enemy bolts in flight when `bolts` is set.
    When {
        crowd: u8,
        low_hp_pct: u8,
        bolts: bool,
    },
}

impl SpendPolicy {
    /// PLACEHOLDER: spends only when a crowd is on top of it or it is about to die.
    pub const PANIC: Self = Self::When {
        crowd: 4,
        low_hp_pct: 40,
        bolts: false,
    };
    /// Spends as soon as it can and anything is in range.
    pub const EAGER: Self = Self::When {
        crowd: 1,
        low_hp_pct: 0,
        bolts: false,
    };
}

impl fmt::Display for SpendPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Hoard => f.write_str("hoard"),
            Self::PANIC => f.write_str("panic"),
            Self::EAGER => f.write_str("eager"),
            Self::When {
                crowd,
                low_hp_pct,
                bolts,
            } => {
                write!(f, "when:{crowd}:{low_hp_pct}")?;
                if bolts {
                    f.write_str(":bolts")?;
                }
                Ok(())
            }
        }
    }
}

impl FromStr for SpendPolicy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_lowercase();
        match s.as_str() {
            "hoard" => return Ok(Self::Hoard),
            "panic" => return Ok(Self::PANIC),
            "eager" => return Ok(Self::EAGER),
            _ => {}
        }
        let mut parts = s.strip_prefix("when:").unwrap_or_default().split(':');
        let crowd = parts.next().and_then(|n| n.parse().ok());
        let low_hp_pct = parts.next().and_then(|n| n.parse().ok());
        let bolts = match parts.next() {
            None => Some(false),
            Some("bolts") => Some(true),
            Some(_) => None,
        };
        match (crowd, low_hp_pct, bolts) {
            (Some(crowd), Some(low_hp_pct), Some(bolts)) => Ok(Self::When {
                crowd,
                low_hp_pct,
                bolts,
            }),
            _ => Err(format!(
                "unknown spend policy {s:?}; expected hoard, panic, eager or when:CROWD:HP_PCT[:bolts]"
            )),
        }
    }
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
    /// Sidesteps out of a charger's lane when it shows its tell.
    pub reads_tells: bool,
    pub pickups: PickupPolicy,
    /// How far the bot will walk for a pickup.
    pub pickup_reach: f32,
    pub weapon: WeaponPolicy,
    pub spend: SpendPolicy,
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
                reads_tells: false,
                pickups: PickupPolicy::Ignore,
                pickup_reach: 0.0,
                weapon: WeaponPolicy::Fixed(WeaponKind::Projectile),
                spend: SpendPolicy::EAGER,
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
                reads_tells: false,
                pickups: PickupPolicy::Greedy,
                pickup_reach: 200.0,
                weapon: WeaponPolicy::Fixed(WeaponKind::Projectile),
                spend: SpendPolicy::EAGER,
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
                reads_tells: true,
                pickups: PickupPolicy::Weighed,
                pickup_reach: 300.0,
                weapon: WeaponPolicy::Fixed(WeaponKind::Hitscan),
                spend: SpendPolicy::PANIC,
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
                reads_tells: true,
                pickups: PickupPolicy::Weighed,
                pickup_reach: 400.0,
                weapon: WeaponPolicy::Situational,
                spend: SpendPolicy::PANIC,
            },
        }
    }
}

/// The part of a tier that is a player's hands rather than their choices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HandSkill {
    pub reaction_secs: f32,
    pub decision_secs: f32,
    pub aim_error_deg: f32,
}

/// Highest hand skill level: the expert tier.
pub const MAX_HAND_SKILL: f32 = (SkillTier::ALL.len() - 1) as f32;

/// Hands at a skill level on the tier scale: 0 novice, 1 casual, 2 skilled,
/// 3 expert. Levels in between blend the two neighbouring tiers linearly, so
/// a simulated player can improve gradually.
pub fn hand_skill(level: f32) -> HandSkill {
    let level = level.clamp(0.0, MAX_HAND_SKILL);
    // In range by the clamp above.
    let lower = (level.floor() as usize).min(SkillTier::ALL.len() - 2);
    let t = level - lower as f32;
    let (a, b) = (
        SkillTier::ALL[lower].params(),
        SkillTier::ALL[lower + 1].params(),
    );
    let mix = |x: f32, y: f32| x + (y - x) * t;
    HandSkill {
        reaction_secs: mix(a.reaction_secs, b.reaction_secs),
        decision_secs: mix(a.decision_secs, b.decision_secs),
        aim_error_deg: mix(a.aim_error_deg, b.aim_error_deg),
    }
}

impl SkillTier {
    /// This tier's place on the [`hand_skill`] scale.
    pub fn level(self) -> f32 {
        Self::ALL.iter().position(|t| *t == self).unwrap_or(0) as f32
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hand_skill_matches_tiers_and_blends_between() {
        let skilled = SkillTier::Skilled.params();
        let at_two = hand_skill(SkillTier::Skilled.level());
        assert_eq!(at_two.aim_error_deg, skilled.aim_error_deg);
        let half = hand_skill(2.5);
        let expert = SkillTier::Expert.params();
        let expected = (skilled.aim_error_deg + expert.aim_error_deg) / 2.0;
        assert!((half.aim_error_deg - expected).abs() < 1e-4);
        assert_eq!(hand_skill(9.0), hand_skill(MAX_HAND_SKILL));
    }
}

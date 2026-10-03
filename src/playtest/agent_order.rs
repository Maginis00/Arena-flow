//! The order language an outside agent (an LLM, or a person typing) uses to
//! play in steps: where to move, which weapon to hold, where to shoot, and
//! for how long. Pure parsing; unit-tested.

use super::tier::MAX_HAND_SKILL;
use crate::weapons::WeaponKind;
use bevy::math::Vec2;
use std::fmt;
use std::str::FromStr;

/// PLACEHOLDER: an order lasts this long unless it says `for N`.
pub const DEFAULT_ORDER_SECS: f32 = 0.3;
/// Shortest and longest step an order may ask for.
const MIN_ORDER_SECS: f32 = 0.05;
const MAX_ORDER_SECS: f32 = 3.0;

/// Where to walk while the order lasts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Move {
    /// Hold these keys: each axis -1, 0 or 1, y up.
    Keys(i8, i8),
    /// Walk toward this arena point and stop on it.
    To(Vec2),
}

/// What to shoot at while the order lasts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Aim {
    /// Don't pull the trigger.
    Hold,
    /// The closest enemy.
    Nearest,
    /// The enemy with the most others close to it.
    Densest,
    /// A fixed direction in degrees, 0 = east, 90 = north.
    Angle(f32),
}

/// One step of play.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Order {
    pub movement: Move,
    /// `None` keeps the weapon in hand.
    pub weapon: Option<WeaponKind>,
    pub aim: Aim,
    pub secs: f32,
    /// New hand skill from this order on (see `tier::hand_skill`).
    pub skill: Option<f32>,
}

impl Default for Order {
    fn default() -> Self {
        Self {
            movement: Move::Keys(0, 0),
            weapon: None,
            aim: Aim::Nearest,
            secs: DEFAULT_ORDER_SECS,
            skill: None,
        }
    }
}

fn direction(word: &str) -> Option<(i8, i8)> {
    Some(match word {
        "stop" => (0, 0),
        "n" => (0, 1),
        "ne" => (1, 1),
        "e" => (1, 0),
        "se" => (1, -1),
        "s" => (0, -1),
        "sw" => (-1, -1),
        "w" => (-1, 0),
        "nw" => (-1, 1),
        _ => return None,
    })
}

fn number(word: Option<&str>, what: &str) -> Result<f32, String> {
    let word = word.ok_or_else(|| format!("{what} needs a number"))?;
    word.parse::<f32>()
        .ok()
        .filter(|n| n.is_finite())
        .ok_or_else(|| format!("{what}: {word:?} is not a number"))
}

impl FromStr for Order {
    type Err = String;

    /// Words in any order, all optional: a direction (`n` `ne` ... `stop`) or
    /// `to X Y`; a weapon (`1` `2` `3`, or its name); `fire nearest`,
    /// `fire densest`, `fire DEG` or `hold`; `skill LEVEL` (hands from now on,
    /// 0 novice to 3 expert); `for SECS`.
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let mut order = Self::default();
        let lower = line.to_ascii_lowercase();
        let mut words = lower.split_whitespace();
        while let Some(word) = words.next() {
            if let Some(keys) = direction(word) {
                order.movement = Move::Keys(keys.0, keys.1);
                continue;
            }
            match word {
                "to" => {
                    let x = number(words.next(), "to")?;
                    let y = number(words.next(), "to")?;
                    order.movement = Move::To(Vec2::new(x, y));
                }
                "1" | "projectile" => order.weapon = Some(WeaponKind::Projectile),
                "2" | "hitscan" => order.weapon = Some(WeaponKind::Hitscan),
                "3" | "melee" => order.weapon = Some(WeaponKind::Melee),
                "hold" => order.aim = Aim::Hold,
                "fire" => {
                    order.aim = match words.next() {
                        Some("nearest") => Aim::Nearest,
                        Some("densest") => Aim::Densest,
                        other => Aim::Angle(number(other, "fire")?),
                    }
                }
                "skill" => {
                    let level = number(words.next(), "skill")?;
                    if !(0.0..=MAX_HAND_SKILL).contains(&level) {
                        return Err(format!("skill: {level} is outside 0-{MAX_HAND_SKILL}"));
                    }
                    order.skill = Some(level);
                }
                "for" => {
                    let secs = number(words.next(), "for")?;
                    if !(MIN_ORDER_SECS..=MAX_ORDER_SECS).contains(&secs) {
                        return Err(format!(
                            "for: {secs} s is outside {MIN_ORDER_SECS}-{MAX_ORDER_SECS} s"
                        ));
                    }
                    order.secs = secs;
                }
                _ => return Err(format!("unknown word {word:?}")),
            }
        }
        Ok(order)
    }
}

impl fmt::Display for Order {
    /// Writes the order back in the language [`Order::from_str`] reads.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.movement {
            Move::Keys(x, y) => {
                let word = match (x.signum(), y.signum()) {
                    (0, 1) => "n",
                    (1, 1) => "ne",
                    (1, 0) => "e",
                    (1, -1) => "se",
                    (0, -1) => "s",
                    (-1, -1) => "sw",
                    (-1, 0) => "w",
                    (-1, 1) => "nw",
                    _ => "stop",
                };
                f.write_str(word)?;
            }
            Move::To(at) => write!(f, "to {} {}", at.x, at.y)?,
        }
        if let Some(weapon) = self.weapon {
            let key = match weapon {
                WeaponKind::Projectile => 1,
                WeaponKind::Hitscan => 2,
                WeaponKind::Melee => 3,
            };
            write!(f, " {key}")?;
        }
        match self.aim {
            Aim::Hold => f.write_str(" hold")?,
            Aim::Nearest => f.write_str(" fire nearest")?,
            Aim::Densest => f.write_str(" fire densest")?,
            Aim::Angle(deg) => write!(f, " fire {deg}")?,
        }
        if let Some(level) = self.skill {
            write!(f, " skill {level}")?;
        }
        write!(f, " for {}", self.secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_line_stands_still_and_fires_at_the_nearest() {
        assert_eq!("".parse::<Order>(), Ok(Order::default()));
    }

    #[test]
    fn words_combine_in_any_order() {
        let order: Order = "for 0.5 fire densest 3 sw".parse().expect("valid order");
        assert_eq!(order.movement, Move::Keys(-1, -1));
        assert_eq!(order.weapon, Some(WeaponKind::Melee));
        assert_eq!(order.aim, Aim::Densest);
        assert_eq!(order.secs, 0.5);
    }

    #[test]
    fn to_and_angle_take_numbers() {
        let order: Order = "to -100 50 hold".parse().expect("valid order");
        assert_eq!(order.movement, Move::To(Vec2::new(-100.0, 50.0)));
        assert_eq!(order.aim, Aim::Hold);
        let order: Order = "fire 270".parse().expect("valid order");
        assert_eq!(order.aim, Aim::Angle(270.0));
    }

    #[test]
    fn bad_words_and_steps_are_refused() {
        assert!("jump".parse::<Order>().is_err());
        assert!("to 10".parse::<Order>().is_err());
        assert!("for 10".parse::<Order>().is_err());
        assert!("fire everywhere".parse::<Order>().is_err());
        assert!("skill 4".parse::<Order>().is_err());
    }

    #[test]
    fn display_reads_back_to_the_same_order() {
        for line in [
            "ne 2 fire nearest skill 1.5 for 0.3",
            "to 10 -20 hold for 1",
            "stop fire 45 for 0.1",
        ] {
            let order: Order = line.parse().expect("valid order");
            assert_eq!(order.to_string().parse::<Order>(), Ok(order));
        }
    }
}

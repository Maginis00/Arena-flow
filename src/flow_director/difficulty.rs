//! The difficulty level, the levers it sets for a wave, and the director's
//! decision facts.

use bevy::prelude::*;
use std::fmt;
use thiserror::Error;

/// Difficulty level from 1.0 to 10.0 in quarter steps, always within
/// `MIN..=MAX`. Stored as a whole number of quarters so it stays exact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Difficulty(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("difficulty of {0} quarters is outside {min}..={max}", min = Difficulty::MIN.0, max = Difficulty::MAX.0)]
pub struct DifficultyOutOfRange(pub u8);

impl Difficulty {
    /// Steps per whole level.
    pub const QUARTERS_PER_LEVEL: u8 = 4;
    pub const MIN: Self = Self(Self::QUARTERS_PER_LEVEL);
    pub const MAX: Self = Self(10 * Self::QUARTERS_PER_LEVEL);

    /// A whole level, `1..=10`.
    pub const fn new(level: u8) -> Result<Self, DifficultyOutOfRange> {
        Self::from_quarters(level.saturating_mul(Self::QUARTERS_PER_LEVEL))
    }

    /// A level in quarter steps: 13 quarters is level 3.25.
    pub const fn from_quarters(quarters: u8) -> Result<Self, DifficultyOutOfRange> {
        if quarters >= Self::MIN.0 && quarters <= Self::MAX.0 {
            Ok(Self(quarters))
        } else {
            Err(DifficultyOutOfRange(quarters))
        }
    }

    pub const fn quarters(self) -> u8 {
        self.0
    }

    /// The level as a number, `1.0..=10.0`.
    pub fn level(self) -> f32 {
        f32::from(self.0) / f32::from(Self::QUARTERS_PER_LEVEL)
    }

    /// Move by `quarters` (negative is easier), clamped to `MIN..=MAX`.
    pub fn shifted(self, quarters: i16) -> Self {
        let moved =
            (i16::from(self.0) + quarters).clamp(i16::from(Self::MIN.0), i16::from(Self::MAX.0));
        // In range after the clamp, so the conversion cannot fail.
        Self(u8::try_from(moved).unwrap_or(Self::MIN.0))
    }
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.level())
    }
}

/// How the last waves read against the flow band.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    /// Smoothed risk below the band: the player is safe, bored side.
    TooEasy,
    /// Smoothed risk inside the band.
    InBand,
    /// Smoothed risk above the band: the anxious side.
    TooRisky,
    /// The player died this wave.
    Died,
}

/// Why the director chose the difficulty it did. Shown in telemetry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionReason {
    /// Starting difficulty before any wave was played.
    Initial,
    /// Smoothed risk below the band.
    Raised,
    /// Smoothed risk above the band.
    LoweredRisky,
    /// Player died.
    LoweredDied,
    /// Smoothed risk inside the band.
    HeldInBand,
    /// Wanted to go up but already at the maximum.
    HeldAtMax,
    /// Wanted to go down but already at the minimum.
    HeldAtMin,
}

impl fmt::Display for DecisionReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Initial => "initial difficulty",
            Self::Raised => "too safe: smoothed risk below band, up",
            Self::LoweredRisky => "too risky: smoothed risk above band, down",
            Self::LoweredDied => "player died, down",
            Self::HeldInBand => "in band: hold",
            Self::HeldAtMax => "too safe but at max difficulty, hold",
            Self::HeldAtMin => "too risky but at min difficulty, hold",
        })
    }
}

/// The only knobs difficulty turns in v1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaveLevers {
    pub enemy_count: u32,
    pub enemy_speed: f32,
    pub contact_damage: u32,
}

/// The difficulty for the next wave was decided. Written once at startup
/// (reason `Initial`) and once after every `WaveReport`.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct DifficultyAdjusted {
    pub previous: Difficulty,
    pub difficulty: Difficulty,
    pub reason: DecisionReason,
    pub levers: WaveLevers,
    /// Risk of the wave just finished and the smoothed risk the decision used,
    /// both in `[0, 1]`. `None` for the initial difficulty.
    pub risk: Option<RiskReading>,
}

/// The director's reading of how risky play has been.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RiskReading {
    pub wave: f32,
    pub smoothed: f32,
}

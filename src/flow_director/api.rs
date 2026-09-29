use bevy::prelude::*;
use std::fmt;
use thiserror::Error;

/// Difficulty level, always within `MIN..=MAX`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Difficulty(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("difficulty {0} is outside {min}..={max}", min = Difficulty::MIN.0, max = Difficulty::MAX.0)]
pub struct DifficultyOutOfRange(pub u8);

impl Difficulty {
    pub const MIN: Self = Self(1);
    pub const MAX: Self = Self(10);

    pub const fn new(level: u8) -> Result<Self, DifficultyOutOfRange> {
        if level >= Self::MIN.0 && level <= Self::MAX.0 {
            Ok(Self(level))
        } else {
            Err(DifficultyOutOfRange(level))
        }
    }

    pub const fn get(self) -> u8 {
        self.0
    }

    /// Step up or down by one, clamped to `MIN..=MAX`.
    pub fn stepped(self, step: Step) -> Self {
        let level = match step {
            Step::Up => self.0.saturating_add(1),
            Step::Hold => self.0,
            Step::Down => self.0.saturating_sub(1),
        };
        Self(level.clamp(Self::MIN.0, Self::MAX.0))
    }
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Direction of a difficulty change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Up,
    Hold,
    Down,
}

/// How the last wave read against the flow channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    /// Cleared fast with high hp.
    TooEasy,
    /// Cleared, but close (low hp or slow) or unremarkable.
    InBand,
    /// Player died (wave failed).
    TooHard,
}

impl Signal {
    pub const fn step(self) -> Step {
        match self {
            Self::TooEasy => Step::Up,
            Self::InBand => Step::Hold,
            Self::TooHard => Step::Down,
        }
    }
}

/// Why the director chose the difficulty it did. Shown in telemetry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionReason {
    /// Starting difficulty before any wave was played.
    Initial,
    /// Cleared fast with high hp.
    Raised,
    /// Player died.
    Lowered,
    /// Cleared with low hp.
    HeldLowHp,
    /// Cleared, but slowly.
    HeldSlowClear,
    /// Cleared, neither fast nor close.
    HeldInBand,
    /// Signal asked for a change, but difficulty changed last wave for a
    /// different signal. Waiting for it to repeat.
    HeldHysteresis(Signal),
    /// Wanted to go up but already at the maximum.
    HeldAtMax,
    /// Wanted to go down but already at the minimum.
    HeldAtMin,
}

impl fmt::Display for DecisionReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Initial => f.write_str("initial difficulty"),
            Self::Raised => f.write_str("too easy: cleared fast with high hp, +1"),
            Self::Lowered => f.write_str("too hard: player died, -1"),
            Self::HeldLowHp => f.write_str("in band: cleared with low hp, hold"),
            Self::HeldSlowClear => f.write_str("in band: cleared slowly, hold"),
            Self::HeldInBand => f.write_str("in band: hold"),
            Self::HeldHysteresis(signal) => {
                write!(
                    f,
                    "hysteresis: {signal:?} after a change, waiting for repeat, hold"
                )
            }
            Self::HeldAtMax => f.write_str("too easy but at max difficulty, hold"),
            Self::HeldAtMin => f.write_str("too hard but at min difficulty, hold"),
        }
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
}

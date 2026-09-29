//! The pure decision. No ECS types beyond plain data; unit-tested below.

use super::api::{DecisionReason, Difficulty, Signal, Step, WaveLevers};
use crate::waves::api::WaveReport;

/// Thresholds that turn a report into a [`Signal`]. All PLACEHOLDER values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DirectorConfig {
    /// A clear is "fast" if it took at most this many seconds per enemy spawned.
    pub fast_secs_per_enemy: f32,
    /// A clear is "slow" (close) if it took more than this many seconds per enemy.
    pub slow_secs_per_enemy: f32,
    /// "High hp": lost at most this fraction of max hp during the wave.
    pub high_hp_max_loss: f32,
    /// "Low hp": lost at least this fraction of max hp during the wave.
    pub low_hp_min_loss: f32,
}

impl Default for DirectorConfig {
    fn default() -> Self {
        Self {
            fast_secs_per_enemy: 1.5, // PLACEHOLDER
            slow_secs_per_enemy: 3.0, // PLACEHOLDER
            high_hp_max_loss: 0.25,   // PLACEHOLDER
            low_hp_min_loss: 0.6,     // PLACEHOLDER
        }
    }
}

/// What the director remembers between waves, for hysteresis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DirectorMemory {
    pub last_signal: Option<Signal>,
    pub changed_last_wave: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Decision {
    pub difficulty: Difficulty,
    pub signal: Signal,
    pub reason: DecisionReason,
    pub memory: DirectorMemory,
}

/// Decide the difficulty of the next wave from the one just finished.
///
/// - cleared, fast, high hp: +1
/// - cleared but close (low hp or slow), or unremarkable: hold
/// - player died: -1
///
/// Clamped to `1..=10`. Hysteresis: if difficulty changed after the previous
/// wave, it only changes again when the same signal repeats.
pub fn decide(
    current: Difficulty,
    report: &WaveReport,
    memory: DirectorMemory,
    config: &DirectorConfig,
) -> Decision {
    let (signal, in_band_reason) = classify(report, config);
    let step = signal.step();

    let blocked_by_hysteresis =
        step != Step::Hold && memory.changed_last_wave && memory.last_signal != Some(signal);

    let (difficulty, reason) = if blocked_by_hysteresis {
        (current, DecisionReason::HeldHysteresis(signal))
    } else {
        let next = current.stepped(step);
        let reason = match (step, next == current) {
            (Step::Hold, _) => in_band_reason,
            (Step::Up, true) => DecisionReason::HeldAtMax,
            (Step::Up, false) => DecisionReason::Raised,
            (Step::Down, true) => DecisionReason::HeldAtMin,
            (Step::Down, false) => DecisionReason::Lowered,
        };
        (next, reason)
    };

    Decision {
        difficulty,
        signal,
        reason,
        memory: DirectorMemory {
            last_signal: Some(signal),
            changed_last_wave: difficulty != current,
        },
    }
}

/// Signal plus the hold reason to use if the signal is `InBand`.
fn classify(report: &WaveReport, config: &DirectorConfig) -> (Signal, DecisionReason) {
    if !report.cleared() {
        return (Signal::TooHard, DecisionReason::Lowered);
    }
    let per_enemy = report.duration_secs / report.enemies_spawned.max(1) as f32;
    let loss = report.damage_fraction();

    if loss >= config.low_hp_min_loss {
        (Signal::InBand, DecisionReason::HeldLowHp)
    } else if per_enemy > config.slow_secs_per_enemy {
        (Signal::InBand, DecisionReason::HeldSlowClear)
    } else if per_enemy <= config.fast_secs_per_enemy && loss <= config.high_hp_max_loss {
        (Signal::TooEasy, DecisionReason::Raised)
    } else {
        (Signal::InBand, DecisionReason::HeldInBand)
    }
}

/// Map difficulty to the three v1 levers. PLACEHOLDER curve: linear per level.
pub fn levers_for(difficulty: Difficulty) -> WaveLevers {
    const BASE_ENEMY_COUNT: u32 = 4; // PLACEHOLDER
    const ENEMY_COUNT_PER_LEVEL: u32 = 2; // PLACEHOLDER
    const BASE_ENEMY_SPEED: f32 = 90.0; // PLACEHOLDER, world units / s
    const ENEMY_SPEED_PER_LEVEL: f32 = 12.0; // PLACEHOLDER
    const BASE_CONTACT_DAMAGE: u32 = 8; // PLACEHOLDER, hp per contact hit
    const CONTACT_DAMAGE_PER_LEVEL: u32 = 2; // PLACEHOLDER

    let above_min = u32::from(difficulty.get() - Difficulty::MIN.get());
    WaveLevers {
        enemy_count: BASE_ENEMY_COUNT + ENEMY_COUNT_PER_LEVEL * above_min,
        enemy_speed: BASE_ENEMY_SPEED + ENEMY_SPEED_PER_LEVEL * above_min as f32,
        contact_damage: BASE_CONTACT_DAMAGE + CONTACT_DAMAGE_PER_LEVEL * above_min,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::waves::api::WaveIndex;

    fn difficulty(level: u8) -> Difficulty {
        Difficulty::new(level).expect("test difficulty in range")
    }

    /// 10 enemies, 100 max hp; tweak per case.
    fn report(duration_secs: f32, damage_taken: u32, player_died: bool) -> WaveReport {
        WaveReport {
            index: WaveIndex(1),
            difficulty: difficulty(5),
            duration_secs,
            enemies_spawned: 10,
            enemies_killed: if player_died { 4 } else { 10 },
            damage_taken,
            player_max_hp: 100,
            player_died,
            shots_fired: 40,
            shots_hit: 30,
        }
    }

    fn fresh() -> DirectorMemory {
        DirectorMemory::default()
    }

    #[test]
    fn too_easy_raises() {
        let d = decide(
            difficulty(5),
            &report(8.0, 5, false),
            fresh(),
            &DirectorConfig::default(),
        );
        assert_eq!(d.signal, Signal::TooEasy);
        assert_eq!(d.difficulty, difficulty(6));
        assert_eq!(d.reason, DecisionReason::Raised);
        assert!(d.memory.changed_last_wave);
    }

    #[test]
    fn in_band_holds() {
        // Cleared, but lost 70% hp: close call.
        let d = decide(
            difficulty(5),
            &report(8.0, 70, false),
            fresh(),
            &DirectorConfig::default(),
        );
        assert_eq!(d.signal, Signal::InBand);
        assert_eq!(d.difficulty, difficulty(5));
        assert_eq!(d.reason, DecisionReason::HeldLowHp);

        // Cleared with high hp but slowly.
        let d = decide(
            difficulty(5),
            &report(45.0, 5, false),
            fresh(),
            &DirectorConfig::default(),
        );
        assert_eq!(d.difficulty, difficulty(5));
        assert_eq!(d.reason, DecisionReason::HeldSlowClear);
    }

    #[test]
    fn too_hard_lowers() {
        let d = decide(
            difficulty(5),
            &report(20.0, 100, true),
            fresh(),
            &DirectorConfig::default(),
        );
        assert_eq!(d.signal, Signal::TooHard);
        assert_eq!(d.difficulty, difficulty(4));
        assert_eq!(d.reason, DecisionReason::Lowered);
    }

    #[test]
    fn hysteresis_holds_a_flip_until_the_signal_repeats() {
        let config = DirectorConfig::default();
        let raised = decide(difficulty(5), &report(8.0, 5, false), fresh(), &config);
        assert_eq!(raised.difficulty, difficulty(6));

        // Died right after a raise: different signal, so hold.
        let held = decide(
            raised.difficulty,
            &report(20.0, 100, true),
            raised.memory,
            &config,
        );
        assert_eq!(held.difficulty, difficulty(6));
        assert_eq!(held.reason, DecisionReason::HeldHysteresis(Signal::TooHard));

        // Died again: the signal repeated, so lower.
        let lowered = decide(
            held.difficulty,
            &report(20.0, 100, true),
            held.memory,
            &config,
        );
        assert_eq!(lowered.difficulty, difficulty(5));
    }

    #[test]
    fn repeated_signal_may_change_twice_in_a_row() {
        let config = DirectorConfig::default();
        let first = decide(difficulty(5), &report(8.0, 5, false), fresh(), &config);
        let second = decide(
            first.difficulty,
            &report(8.0, 5, false),
            first.memory,
            &config,
        );
        assert_eq!(second.difficulty, difficulty(7));
    }

    #[test]
    fn clamps_at_bounds() {
        let config = DirectorConfig::default();
        let top = decide(Difficulty::MAX, &report(8.0, 5, false), fresh(), &config);
        assert_eq!(top.difficulty, Difficulty::MAX);
        assert_eq!(top.reason, DecisionReason::HeldAtMax);

        let bottom = decide(Difficulty::MIN, &report(20.0, 100, true), fresh(), &config);
        assert_eq!(bottom.difficulty, Difficulty::MIN);
        assert_eq!(bottom.reason, DecisionReason::HeldAtMin);
    }

    #[test]
    fn difficulty_rejects_out_of_range() {
        assert_eq!(
            Difficulty::new(0),
            Err(super::super::api::DifficultyOutOfRange(0))
        );
        assert_eq!(
            Difficulty::new(11),
            Err(super::super::api::DifficultyOutOfRange(11))
        );
    }
}

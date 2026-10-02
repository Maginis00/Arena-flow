//! The pure decision. No ECS types beyond plain data; unit-tested below.

use super::api::{DecisionReason, Difficulty, RiskReading, Signal, WaveLevers};
use crate::waves::api::WaveReport;

/// The flow band and how the director moves toward it. All PLACEHOLDER values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DirectorConfig {
    /// Smoothed risk below this is too safe.
    pub band_low: f32,
    /// Smoothed risk above this is too risky.
    pub band_high: f32,
    /// Weight of the newest wave in the smoothed risk; the rest is history.
    pub smoothing: f32,
    /// Quarter steps up when too safe.
    pub up_quarters: i16,
    /// Quarter steps down when too risky.
    pub down_quarters: i16,
    /// Quarter steps down when the player died.
    pub died_quarters: i16,
}

impl Default for DirectorConfig {
    fn default() -> Self {
        // Centred on the flow curve's peak (half the hp at stake), quick to
        // climb out of boredom and quick to see a single risky wave. Chosen
        // by a 96-variant bot search for the most waves near the peak.
        Self {
            band_low: 0.35,   // PLACEHOLDER
            band_high: 0.65,  // PLACEHOLDER
            smoothing: 0.5,   // PLACEHOLDER
            up_quarters: 2,   // PLACEHOLDER: +0.5
            down_quarters: 2, // PLACEHOLDER: -0.5
            died_quarters: 4, // PLACEHOLDER: -1.0
        }
    }
}

impl DirectorConfig {
    /// Never moves: every wave plays at the starting difficulty. For
    /// measuring how a player does at one fixed difficulty.
    pub fn pinned() -> Self {
        Self {
            up_quarters: 0,
            down_quarters: 0,
            died_quarters: 0,
            ..Self::default()
        }
    }

    pub fn in_band(&self, risk: f32) -> bool {
        (self.band_low..=self.band_high).contains(&risk)
    }
}

/// What the director remembers between waves.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DirectorMemory {
    /// Smoothed risk so far; `None` before the first wave.
    pub smoothed_risk: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Decision {
    pub difficulty: Difficulty,
    pub signal: Signal,
    pub reason: DecisionReason,
    pub risk: RiskReading,
    pub memory: DirectorMemory,
}

/// How close the wave came to killing the player, in `[0, 1]`: the share of
/// the hp the player brought into the wave that it took away, and 1 if the
/// player died. Measuring against the hp at the start keeps anything that
/// happened before the wave out of this wave's reading.
pub fn wave_risk(report: &WaveReport) -> f32 {
    if report.player_died || report.start_hp == 0 {
        return 1.0;
    }
    let lost = report.start_hp.saturating_sub(report.lowest_hp);
    (lost as f32 / report.start_hp as f32).clamp(0.0, 1.0)
}

/// Decide the difficulty of the next wave from the one just finished.
///
/// The wave's risk is blended into a smoothed risk, then:
/// - player died: down `died_quarters` at once
/// - smoothed risk above the band: down `down_quarters`
/// - inside the band: hold
/// - below the band: up `up_quarters`
///
/// A death steps down further than a safe wave steps up: past the peak the
/// flow curve falls steeply, so overshooting into danger costs more
/// engagement than staying a little safe. Smoothing is the hysteresis: a
/// death keeps the smoothed risk high for a wave or two, so the director does
/// not climb straight back.
pub fn decide(
    current: Difficulty,
    report: &WaveReport,
    memory: DirectorMemory,
    config: &DirectorConfig,
) -> Decision {
    let wave = wave_risk(report);
    let smoothed = match memory.smoothed_risk {
        Some(previous) => config.smoothing * wave + (1.0 - config.smoothing) * previous,
        None => wave,
    };

    let (signal, quarters) = if report.player_died {
        (Signal::Died, -config.died_quarters)
    } else if smoothed > config.band_high {
        (Signal::TooRisky, -config.down_quarters)
    } else if smoothed < config.band_low {
        (Signal::TooEasy, config.up_quarters)
    } else {
        (Signal::InBand, 0)
    };

    let difficulty = current.shifted(quarters);
    let reason = match (signal, difficulty == current) {
        (Signal::InBand, _) => DecisionReason::HeldInBand,
        (Signal::TooEasy, true) => DecisionReason::HeldAtMax,
        (Signal::TooEasy, false) => DecisionReason::Raised,
        (Signal::TooRisky | Signal::Died, true) => DecisionReason::HeldAtMin,
        (Signal::TooRisky, false) => DecisionReason::LoweredRisky,
        (Signal::Died, false) => DecisionReason::LoweredDied,
    };

    Decision {
        difficulty,
        signal,
        reason,
        risk: RiskReading { wave, smoothed },
        memory: DirectorMemory {
            smoothed_risk: Some(smoothed),
        },
    }
}

/// Map difficulty to the three v1 levers. PLACEHOLDER curve: linear per level,
/// interpolated between whole levels.
pub fn levers_for(difficulty: Difficulty) -> WaveLevers {
    const BASE_ENEMY_COUNT: f32 = 4.0; // PLACEHOLDER
    const ENEMY_COUNT_PER_LEVEL: f32 = 2.0; // PLACEHOLDER
    const BASE_ENEMY_SPEED: f32 = 90.0; // PLACEHOLDER, world units / s
    const ENEMY_SPEED_PER_LEVEL: f32 = 12.0; // PLACEHOLDER
    const BASE_CONTACT_DAMAGE: f32 = 8.0; // PLACEHOLDER, hp per contact hit
    const CONTACT_DAMAGE_PER_LEVEL: f32 = 2.0; // PLACEHOLDER

    let above_min = difficulty.level() - Difficulty::MIN.level();
    WaveLevers {
        enemy_count: (BASE_ENEMY_COUNT + ENEMY_COUNT_PER_LEVEL * above_min).round() as u32,
        enemy_speed: BASE_ENEMY_SPEED + ENEMY_SPEED_PER_LEVEL * above_min,
        contact_damage: (BASE_CONTACT_DAMAGE + CONTACT_DAMAGE_PER_LEVEL * above_min).round() as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flow_director::api::DifficultyOutOfRange;
    use crate::waves::api::WaveIndex;

    fn difficulty(level: u8) -> Difficulty {
        Difficulty::new(level).expect("test difficulty in range")
    }

    fn quarters(q: u8) -> Difficulty {
        Difficulty::from_quarters(q).expect("test difficulty in range")
    }

    /// 100 max hp, full at the start; `lowest_hp` sets the risk.
    fn report(lowest_hp: u32, player_died: bool) -> WaveReport {
        WaveReport {
            index: WaveIndex(1),
            attempt: 1,
            difficulty: difficulty(5),
            duration_secs: 10.0,
            enemies_spawned: 10,
            enemies_killed: if player_died { 4 } else { 10 },
            damage_taken: 100 - lowest_hp,
            hits_taken: 3,
            player_max_hp: 100,
            start_hp: 100,
            lowest_hp: if player_died { 0 } else { lowest_hp },
            player_died,
            shots_fired: 40,
            shots_hit: 30,
            pickups_collected: 0,
            danger: Default::default(),
        }
    }

    fn after(smoothed: f32) -> DirectorMemory {
        DirectorMemory {
            smoothed_risk: Some(smoothed),
        }
    }

    fn fresh() -> DirectorMemory {
        DirectorMemory::default()
    }

    #[test]
    fn risk_is_how_close_the_player_came_to_dying() {
        assert_eq!(wave_risk(&report(100, false)), 0.0);
        assert_eq!(wave_risk(&report(40, false)), 0.6);
        assert_eq!(wave_risk(&report(80, true)), 1.0);
    }

    #[test]
    fn risk_ignores_damage_carried_in_from_earlier_waves() {
        // Came in at 40 hp and took no damage: this wave was safe.
        let untouched = WaveReport {
            start_hp: 40,
            lowest_hp: 40,
            ..report(100, false)
        };
        assert_eq!(wave_risk(&untouched), 0.0);
        // Came in at 40 hp and dropped to 10: lost three quarters of it.
        let close = WaveReport {
            start_hp: 40,
            lowest_hp: 10,
            ..report(100, false)
        };
        assert_eq!(wave_risk(&close), 0.75);
    }

    #[test]
    fn too_safe_raises_a_half() {
        let d = decide(
            difficulty(5),
            &report(95, false),
            fresh(),
            &DirectorConfig::default(),
        );
        assert_eq!(d.signal, Signal::TooEasy);
        assert_eq!(d.difficulty, quarters(22));
        assert_eq!(d.reason, DecisionReason::Raised);
    }

    #[test]
    fn in_band_holds() {
        let d = decide(
            difficulty(5),
            &report(50, false),
            fresh(),
            &DirectorConfig::default(),
        );
        assert_eq!(d.signal, Signal::InBand);
        assert_eq!(d.difficulty, difficulty(5));
        assert_eq!(d.reason, DecisionReason::HeldInBand);
    }

    #[test]
    fn too_risky_lowers_a_half() {
        let d = decide(
            difficulty(5),
            &report(20, false),
            fresh(),
            &DirectorConfig::default(),
        );
        assert_eq!(d.signal, Signal::TooRisky);
        assert_eq!(d.difficulty, quarters(18));
        assert_eq!(d.reason, DecisionReason::LoweredRisky);
    }

    #[test]
    fn death_lowers_a_whole_level_whatever_the_history() {
        let d = decide(
            difficulty(5),
            &report(0, true),
            after(0.0),
            &DirectorConfig::default(),
        );
        assert_eq!(d.signal, Signal::Died);
        assert_eq!(d.difficulty, difficulty(4));
        assert_eq!(d.reason, DecisionReason::LoweredDied);
    }

    #[test]
    fn a_death_steps_down_further_than_a_safe_wave_steps_up() {
        let c = DirectorConfig::default();
        assert!(c.down_quarters >= c.up_quarters);
        assert!(c.died_quarters > c.up_quarters);
        assert!(c.died_quarters > c.down_quarters);
    }

    #[test]
    fn the_band_sits_around_the_flow_peak() {
        let c = DirectorConfig::default();
        assert!(c.in_band(crate::flow_director::FLOW_PEAK));
    }

    #[test]
    fn smoothing_blends_the_new_wave_into_history() {
        let c = DirectorConfig::default();
        // One very safe wave after risky history stays in band: no jump up.
        let d = decide(difficulty(5), &report(100, false), after(0.8), &c);
        assert!((d.risk.smoothed - 0.4).abs() < 1e-5);
        assert_eq!(d.reason, DecisionReason::HeldInBand);
        assert_eq!(d.memory.smoothed_risk, Some(d.risk.smoothed));
    }

    #[test]
    fn a_death_keeps_the_director_from_climbing_straight_back() {
        let c = DirectorConfig::default();
        let died = decide(difficulty(5), &report(0, true), after(0.5), &c);
        let next = decide(died.difficulty, &report(100, false), died.memory, &c);
        assert!(next.difficulty <= died.difficulty);
    }

    #[test]
    fn clamps_at_bounds() {
        let c = DirectorConfig::default();
        let top = decide(Difficulty::MAX, &report(100, false), fresh(), &c);
        assert_eq!(top.difficulty, Difficulty::MAX);
        assert_eq!(top.reason, DecisionReason::HeldAtMax);

        let bottom = decide(Difficulty::MIN, &report(0, true), fresh(), &c);
        assert_eq!(bottom.difficulty, Difficulty::MIN);
        assert_eq!(bottom.reason, DecisionReason::HeldAtMin);
    }

    #[test]
    fn difficulty_is_exact_in_quarters() {
        assert_eq!(difficulty(3).quarters(), 12);
        assert_eq!(quarters(13).level(), 3.25);
        assert_eq!(quarters(13).to_string(), "3.25");
        assert_eq!(difficulty(3).shifted(-100), Difficulty::MIN);
        assert_eq!(difficulty(3).shifted(100), Difficulty::MAX);
        assert_eq!(Difficulty::new(0), Err(DifficultyOutOfRange(0)));
        assert_eq!(Difficulty::new(11), Err(DifficultyOutOfRange(44)));
        assert_eq!(Difficulty::from_quarters(41), Err(DifficultyOutOfRange(41)));
    }

    #[test]
    fn levers_match_the_old_curve_on_whole_levels_and_interpolate_between() {
        let at = |d| levers_for(d);
        assert_eq!(at(difficulty(1)).enemy_count, 4);
        assert_eq!(at(difficulty(10)).enemy_count, 22);
        assert_eq!(at(difficulty(10)).contact_damage, 26);
        assert_eq!(at(quarters(6)).enemy_count, 5); // level 1.5
        assert_eq!(at(quarters(6)).enemy_speed, 96.0);
    }
}

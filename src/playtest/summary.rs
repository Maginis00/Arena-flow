//! Turns a session record into the numbers that say whether the director keeps a
//! tier in its own band. Pure; unit-tested.

use crate::flow_director::DecisionReason;
use crate::flow_director::{DirectorConfig, FLOW_PEAK, IN_FLOW, engagement, wave_risk};
use crate::telemetry::SessionRecord;
use crate::waves::WaveReport;
use std::fmt::Write as _;

/// Engagement under this reads as a lost wave on the flow curve: boring
/// below the peak, overwhelming above it. 0.3 is a risk under 0.15 or over
/// about 0.7.
const LOST_BELOW: f32 = 0.3;

/// Headline numbers for one session.
#[derive(Debug, Clone, PartialEq)]
pub struct Summary {
    pub waves: usize,
    pub clears: usize,
    pub deaths: usize,
    /// Highest wave index reached.
    pub furthest_wave: u32,
    /// Mean difficulty over the second half of waves: where the director settled.
    pub settled: f32,
    /// Lowest and highest difficulty over the second half.
    pub settled_range: (f32, f32),
    /// Share of post-wave decisions that held because the wave read in band.
    pub in_band: f32,
    /// Share of waves whose own risk fell inside the director's band.
    pub risk_in_band: f32,
    /// Median risk per wave (share of the starting hp lost, 1 on death).
    pub median_risk: f32,
    /// Shots that hit something, over shots fired.
    pub hit_rate: f32,
    /// Mean seconds per enemy on cleared waves.
    pub secs_per_enemy: f32,
    /// Mean fraction of max hp lost on cleared waves.
    pub hp_lost_on_clear: f32,
    /// Mean engagement on the flow curve over all waves, `0..=1`: how much
    /// of the time the mechanical pressure sat near the peak.
    pub flow: f32,
    /// Share of waves at or above [`IN_FLOW`] engagement.
    pub in_flow: f32,
    /// Share of waves too safe to engage (left of the peak, low engagement).
    pub bored: f32,
    /// Share of waves too dangerous to engage (right of the peak, deaths included).
    pub overwhelmed: f32,
    /// Close calls per wave: enemies that nearly touched and did not hit.
    pub close_calls: f32,
    /// Share of wave time with no enemy within a few seconds of contact.
    pub calm: f32,
    /// Difficulty of each wave in order, with an `x` after deaths.
    pub trajectory: String,
}

impl Summary {
    pub fn of(session: &SessionRecord) -> Self {
        let reports: Vec<&WaveReport> = session.waves.iter().map(|w| &w.report).collect();
        let cleared: Vec<_> = reports.iter().filter(|r| r.cleared()).collect();
        let second_half = &reports[reports.len() / 2..];
        let levels: Vec<f32> = second_half.iter().map(|r| r.difficulty.level()).collect();
        let band = DirectorConfig::default();
        let mut risks: Vec<f32> = reports.iter().map(|r| wave_risk(r)).collect();
        let lost = |side: fn(f32) -> bool| {
            risks
                .iter()
                .filter(|r| side(**r) && engagement(**r) < LOST_BELOW)
                .count()
        };
        let bored = lost(|r| r < FLOW_PEAK);
        let overwhelmed = lost(|r| r > FLOW_PEAK);
        let in_flow = risks.iter().filter(|r| engagement(**r) >= IN_FLOW).count();
        let flow = mean(risks.iter().map(|r| engagement(*r)));
        let wave_secs: f32 = reports.iter().map(|r| r.duration_secs).sum();
        let calm_secs: f32 = reports.iter().map(|r| r.danger.calm_secs).sum();
        risks.sort_by(f32::total_cmp);
        let fired: u32 = reports.iter().map(|r| r.shots_fired).sum();
        let hit: u32 = reports.iter().map(|r| r.shots_hit).sum();
        let decisions: Vec<DecisionReason> =
            session.waves.iter().filter_map(|w| w.decision).collect();
        let held = decisions
            .iter()
            .filter(|d| **d == DecisionReason::HeldInBand)
            .count();
        let mut trajectory = String::new();
        for r in &reports {
            // Writing to a String cannot fail.
            let _ = write!(
                trajectory,
                "{}{} ",
                r.difficulty,
                if r.player_died { "x" } else { "" }
            );
        }
        Self {
            waves: reports.len(),
            clears: cleared.len(),
            deaths: reports.iter().filter(|r| r.player_died).count(),
            furthest_wave: reports.iter().map(|r| r.index.0).max().unwrap_or(0),
            settled: mean(levels.iter().copied()),
            settled_range: (
                levels.iter().copied().reduce(f32::min).unwrap_or(0.0),
                levels.iter().copied().reduce(f32::max).unwrap_or(0.0),
            ),
            in_band: ratio(held, decisions.len()),
            risk_in_band: ratio(
                risks.iter().filter(|r| band.in_band(**r)).count(),
                risks.len(),
            ),
            median_risk: risks.get(risks.len() / 2).copied().unwrap_or(0.0),
            hit_rate: ratio(hit as usize, fired as usize),
            secs_per_enemy: mean(
                cleared
                    .iter()
                    .map(|r| r.duration_secs / r.enemies_spawned.max(1) as f32),
            ),
            hp_lost_on_clear: mean(cleared.iter().map(|r| r.damage_fraction())),
            flow,
            in_flow: ratio(in_flow, reports.len()),
            bored: ratio(bored, reports.len()),
            overwhelmed: ratio(overwhelmed, reports.len()),
            close_calls: mean(reports.iter().map(|r| r.danger.close_calls as f32)),
            calm: if wave_secs > 0.0 {
                calm_secs / wave_secs
            } else {
                0.0
            },
            trajectory: trajectory.trim_end().to_owned(),
        }
    }
}

pub(super) fn mean(values: impl Iterator<Item = f32>) -> f32 {
    let (sum, n) = values.fold((0.0, 0u32), |(s, n), v| (s + v, n + 1));
    if n == 0 { 0.0 } else { sum / n as f32 }
}

pub(super) fn ratio(part: usize, whole: usize) -> f32 {
    if whole == 0 {
        0.0
    } else {
        part as f32 / whole as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flow_director::Difficulty;
    use crate::telemetry::WaveRecord;
    use crate::waves::WaveIndex;

    fn wave(index: u32, level: u8, died: bool) -> WaveRecord {
        let report = WaveReport {
            index: WaveIndex(index),
            attempt: 1,
            difficulty: Difficulty::new(level).unwrap_or(Difficulty::MIN),
            duration_secs: 10.0,
            enemies_spawned: 5,
            enemies_killed: if died { 2 } else { 5 },
            damage_taken: 20,
            hits_taken: 2,
            player_max_hp: 100,
            start_hp: 100,
            lowest_hp: if died { 0 } else { 80 },
            player_died: died,
            shots_fired: 10,
            shots_hit: 5,
            pickups_collected: 0,
            danger: Default::default(),
        };
        WaveRecord {
            report,
            weapons: Default::default(),
            pickups: Vec::new(),
            decision: Some(DecisionReason::HeldInBand),
        }
    }

    #[test]
    fn summary_settles_on_the_second_half() {
        let log = SessionRecord {
            waves: vec![
                wave(1, 3, false),
                wave(2, 4, true),
                wave(2, 5, false),
                wave(3, 5, false),
            ],
        };
        let s = Summary::of(&log);
        assert_eq!((s.waves, s.clears, s.deaths, s.furthest_wave), (4, 3, 1, 3));
        assert_eq!(s.settled, 5.0);
        assert_eq!(s.settled_range, (5.0, 5.0));
        assert_eq!(s.median_risk, 0.2);
        assert_eq!(s.hit_rate, 0.5);
        assert_eq!(s.secs_per_enemy, 2.0);
        assert_eq!(s.trajectory, "3.00 4.00x 5.00 5.00");
        assert_eq!(s.in_band, 1.0);
    }

    #[test]
    fn flow_scores_each_wave_on_the_curve() {
        // Risks 0.2 (lowest 80), 1.0 (died), 0.2, 0.2.
        let log = SessionRecord {
            waves: vec![
                wave(1, 3, false),
                wave(2, 4, true),
                wave(2, 5, false),
                wave(3, 5, false),
            ],
        };
        let s = Summary::of(&log);
        let expected = (3.0 * engagement(0.2) + engagement(1.0)) / 4.0;
        assert!((s.flow - expected).abs() < 1e-6);
        assert_eq!((s.in_flow, s.bored, s.overwhelmed), (0.0, 0.0, 0.25));
    }

    #[test]
    fn empty_log_is_all_zeros() {
        let s = Summary::of(&SessionRecord::default());
        assert_eq!((s.waves, s.settled, s.hit_rate), (0, 0.0, 0.0));
    }
}

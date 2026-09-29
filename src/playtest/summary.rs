//! Turns a session log into the numbers that say whether the director keeps a
//! tier in its own band. Pure; unit-tested.

use super::session::SessionLog;
use crate::flow_director::api::DecisionReason;
use crate::flow_director::{DirectorConfig, wave_risk};
use crate::pickups::api::PickupKind;
use std::fmt::Write as _;

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
    /// Median risk per wave (1 - lowest hp fraction, 1 on death).
    pub median_risk: f32,
    /// Shots that hit something, over shots fired.
    pub hit_rate: f32,
    /// Mean seconds per enemy on cleared waves.
    pub secs_per_enemy: f32,
    /// Mean fraction of max hp lost on cleared waves.
    pub hp_lost_on_clear: f32,
    /// Difficulty of each wave in order, with an `x` after deaths.
    pub trajectory: String,
}

impl Summary {
    pub fn of(log: &SessionLog) -> Self {
        let reports = &log.reports;
        let cleared: Vec<_> = reports.iter().filter(|r| r.cleared()).collect();
        let second_half = &reports[reports.len() / 2..];
        let levels: Vec<f32> = second_half.iter().map(|r| r.difficulty.level()).collect();
        let band = DirectorConfig::default();
        let mut risks: Vec<f32> = reports.iter().map(wave_risk).collect();
        risks.sort_by(f32::total_cmp);
        let fired: u32 = reports.iter().map(|r| r.shots_fired).sum();
        let hit: u32 = reports.iter().map(|r| r.shots_hit).sum();
        let after_waves: Vec<_> = log
            .decisions
            .iter()
            .filter(|d| d.reason != DecisionReason::Initial)
            .collect();
        let held = after_waves
            .iter()
            .filter(|d| d.reason == DecisionReason::HeldInBand)
            .count();
        let mut trajectory = String::new();
        for r in reports {
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
            in_band: ratio(held, after_waves.len()),
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
            trajectory: trajectory.trim_end().to_owned(),
        }
    }
}

/// Header and one row per session, as a Markdown table.
pub fn table(rows: &[(String, &SessionLog)]) -> String {
    let mut out = String::from(
        "| bot | waves | clears | deaths | furthest | settled diff | range | held in band | risk in band | median risk | hit rate | s/enemy | hp lost/clear | pickups o/h/m |\n\
         |---|---|---|---|---|---|---|---|---|---|---|---|---|---|\n",
    );
    for (name, log) in rows {
        let s = Summary::of(log);
        let _ = writeln!(
            out,
            "| {name} | {} | {} | {} | {} | {:.1} | {:.2}-{:.2} | {:.0}% | {:.0}% | {:.2} | {:.0}% | {:.2} | {:.0}% | {}/{}/{} |",
            s.waves,
            s.clears,
            s.deaths,
            s.furthest_wave,
            s.settled,
            s.settled_range.0,
            s.settled_range.1,
            s.in_band * 100.0,
            s.risk_in_band * 100.0,
            s.median_risk,
            s.hit_rate * 100.0,
            s.secs_per_enemy,
            s.hp_lost_on_clear * 100.0,
            log.pickups_of(PickupKind::Overdrive),
            log.pickups_of(PickupKind::Heavy),
            log.pickups_of(PickupKind::Mend),
        );
    }
    out
}

fn mean(values: impl Iterator<Item = f32>) -> f32 {
    let (sum, n) = values.fold((0.0, 0u32), |(s, n), v| (s + v, n + 1));
    if n == 0 { 0.0 } else { sum / n as f32 }
}

fn ratio(part: usize, whole: usize) -> f32 {
    if whole == 0 {
        0.0
    } else {
        part as f32 / whole as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flow_director::api::Difficulty;
    use crate::waves::api::{WaveIndex, WaveReport};

    fn report(index: u32, level: u8, died: bool) -> WaveReport {
        WaveReport {
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
        }
    }

    #[test]
    fn summary_settles_on_the_second_half() {
        let log = SessionLog {
            reports: vec![
                report(1, 3, false),
                report(2, 4, true),
                report(2, 5, false),
                report(3, 5, false),
            ],
            ..SessionLog::default()
        };
        let s = Summary::of(&log);
        assert_eq!((s.waves, s.clears, s.deaths, s.furthest_wave), (4, 3, 1, 3));
        assert_eq!(s.settled, 5.0);
        assert_eq!(s.settled_range, (5.0, 5.0));
        assert_eq!(s.median_risk, 0.2);
        assert_eq!(s.hit_rate, 0.5);
        assert_eq!(s.secs_per_enemy, 2.0);
        assert_eq!(s.trajectory, "3.00 4.00x 5.00 5.00");
    }

    #[test]
    fn empty_log_is_all_zeros() {
        let s = Summary::of(&SessionLog::default());
        assert_eq!((s.waves, s.settled, s.hit_rate), (0, 0.0, 0.0));
    }
}

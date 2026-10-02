//! Strategy search: compares many fixed strategies over many situations and
//! says whether one of them always wins. Pure; unit-tested.
//!
//! A decision is real when no single strategy is the best everywhere: which
//! one wins has to depend on the situation. That is what the dominance table
//! shows. The gap between the best and worst strategy in a situation is how
//! much the choice matters there (macro difficulty as consequence).
//!
//! Every number here comes from bots, so it measures the mechanical outcome
//! of a macro choice: deaths per wave, not how a plan feels.

use std::collections::BTreeMap;
use std::fmt::Write as _;

/// Two death rates closer than this many standard errors count as a tie.
const TIE_STANDARD_ERRORS: f32 = 2.0;
/// Situations where every strategy dies this rarely, or this often, say
/// nothing about strategy and are left out.
const TOO_SAFE: f32 = 0.05;
const TOO_DEADLY: f32 = 0.95;

/// How one strategy did in one situation, seeds pooled.
#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    pub situation: String,
    pub strategy: String,
    pub waves: usize,
    pub deaths: usize,
}

impl Outcome {
    fn rate(&self) -> f32 {
        self.deaths as f32 / self.waves.max(1) as f32
    }

    /// Is `self` within noise of `best`?
    fn ties(&self, best: &Self) -> bool {
        let variance = |o: &Self| {
            let p = o.rate();
            p * (1.0 - p) / o.waves.max(1) as f32
        };
        let se = (variance(self) + variance(best)).sqrt().max(f32::EPSILON);
        self.rate() - best.rate() <= TIE_STANDARD_ERRORS * se
    }
}

/// One informative situation: best and worst strategy and who ties the best.
#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    pub situation: String,
    pub best: Outcome,
    pub worst: Outcome,
    pub near_best: Vec<String>,
}

/// Per strategy: in how many informative situations it ties the best, and
/// how many more deaths per wave it costs on average (regret).
#[derive(Debug, Clone, PartialEq)]
pub struct Standing {
    pub strategy: String,
    /// Informative situations this strategy played.
    pub played: usize,
    pub near_best: usize,
    pub mean_regret: f32,
}

/// Group outcomes by situation and judge each informative one.
pub fn verdicts(outcomes: &[Outcome]) -> Vec<Verdict> {
    let mut by_situation: BTreeMap<&str, Vec<&Outcome>> = BTreeMap::new();
    for o in outcomes {
        by_situation.entry(&o.situation).or_default().push(o);
    }
    by_situation
        .into_values()
        .filter_map(|group| {
            let by_rate = |a: &&&Outcome, b: &&&Outcome| a.rate().total_cmp(&b.rate());
            let best = *group.iter().min_by(by_rate)?;
            let worst = *group.iter().max_by(by_rate)?;
            if worst.rate() <= TOO_SAFE || best.rate() >= TOO_DEADLY {
                return None;
            }
            Some(Verdict {
                situation: best.situation.clone(),
                near_best: group
                    .iter()
                    .filter(|o| o.ties(best))
                    .map(|o| o.strategy.clone())
                    .collect(),
                best: best.clone(),
                worst: worst.clone(),
            })
        })
        .collect()
}

/// Every strategy's standing over the informative situations, best first.
pub fn standings(outcomes: &[Outcome], verdicts: &[Verdict]) -> Vec<Standing> {
    let mut regret: BTreeMap<&str, (usize, f32, usize)> = BTreeMap::new();
    for v in verdicts {
        for o in outcomes.iter().filter(|o| o.situation == v.situation) {
            let entry = regret.entry(&o.strategy).or_default();
            entry.0 += usize::from(v.near_best.contains(&o.strategy));
            entry.1 += o.rate() - v.best.rate();
            entry.2 += 1;
        }
    }
    let mut out: Vec<Standing> = regret
        .into_iter()
        .map(|(strategy, (near_best, total, n))| Standing {
            strategy: strategy.to_owned(),
            played: n,
            near_best,
            mean_regret: total / n.max(1) as f32,
        })
        .collect();
    out.sort_by(|a, b| a.mean_regret.total_cmp(&b.mean_regret));
    out
}

/// Markdown: one row per informative situation.
pub fn verdict_table(verdicts: &[Verdict]) -> String {
    let mut out = String::from(
        "| situation | best | deaths | worst | deaths | gap | tied with best |\n\
         |---|---|---|---|---|---|---|\n",
    );
    for v in verdicts {
        let _ = writeln!(
            out,
            "| {} | {} | {:.0}% | {} | {:.0}% | {:.0} | {} |",
            v.situation,
            v.best.strategy,
            v.best.rate() * 100.0,
            v.worst.strategy,
            v.worst.rate() * 100.0,
            (v.worst.rate() - v.best.rate()) * 100.0,
            v.near_best.len(),
        );
    }
    out
}

/// Markdown: every strategy, least regret first.
pub fn standing_table(standings: &[Standing]) -> String {
    let mut out =
        String::from("| strategy | ties the best in | mean extra deaths/wave |\n|---|---|---|\n");
    for s in standings {
        let _ = writeln!(
            out,
            "| {} | {}/{} | {:.1} |",
            s.strategy,
            s.near_best,
            s.played,
            s.mean_regret * 100.0
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outcome(situation: &str, strategy: &str, deaths: usize) -> Outcome {
        Outcome {
            situation: situation.to_owned(),
            strategy: strategy.to_owned(),
            waves: 100,
            deaths,
        }
    }

    #[test]
    fn a_strategy_that_wins_everywhere_ties_the_best_everywhere() {
        let outcomes = [
            outcome("a", "smart", 20),
            outcome("a", "dumb", 60),
            outcome("b", "smart", 30),
            outcome("b", "dumb", 70),
        ];
        let v = verdicts(&outcomes);
        let s = standings(&outcomes, &v);
        assert_eq!(s[0].strategy, "smart");
        assert_eq!(s[0].near_best, 2);
        assert_eq!(s[1].near_best, 0);
    }

    #[test]
    fn a_real_decision_has_different_winners() {
        let outcomes = [
            outcome("crowd", "spend", 20),
            outcome("crowd", "hoard", 60),
            outcome("sparse", "spend", 60),
            outcome("sparse", "hoard", 20),
        ];
        let v = verdicts(&outcomes);
        assert_eq!(v[0].best.strategy, "spend");
        assert_eq!(v[1].best.strategy, "hoard");
        assert!(standings(&outcomes, &v).iter().all(|s| s.near_best == 1));
    }

    #[test]
    fn close_rates_tie_and_uninformative_situations_drop_out() {
        let outcomes = [
            outcome("a", "x", 30),
            outcome("a", "y", 33),
            outcome("safe", "x", 0),
            outcome("safe", "y", 2),
        ];
        let v = verdicts(&outcomes);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].near_best.len(), 2);
    }
}

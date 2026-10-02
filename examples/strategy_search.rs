//! Strategy search: every combination of pickup and spend rule plays every
//! situation (pickup rules x enemy mix x tier x fixed difficulty), and the
//! report says which strategy wins where, how much the choice matters, and
//! whether one strategy wins everywhere (then the decision is not real).
//!
//! ```sh
//! cargo run --release --example strategy_search
//! cargo run --release --example strategy_search -- --rules shards --rules shards-bolts \
//!     --enemies grunts --enemies all --pin 6 --pin 8 --seeds 3 --minutes 4
//! cargo run --release --example strategy_search -- --take clear:150 --spend when:3:40
//! ```
//!
//! Without `--take` / `--spend` it searches a built-in grid. Under
//! `shards-bolts` the spend rules also count enemy bolts as threats.

use flow_arena::enemies::api::EnemyMix;
use flow_arena::flow_director::api::Difficulty;
use flow_arena::pickups::api::PickupRules;
use flow_arena::playtest::{
    Outcome, PickupPolicy, SessionConfig, SkillTier, SpendPolicy, Summary, play_many,
    standing_table, standings, verdict_table, verdicts,
};
use std::process::ExitCode;

const USAGE: &str = "usage: strategy_search [--rules NAME]... [--enemies MIX]... [--tier NAME]... \
                     [--pin LEVEL]... [--take POLICY]... [--spend POLICY]... [--seeds N] [--minutes N]";

struct Args {
    rules: Vec<PickupRules>,
    enemies: Vec<EnemyMix>,
    tiers: Vec<SkillTier>,
    pins: Vec<Difficulty>,
    takes: Vec<PickupPolicy>,
    spends: Vec<SpendPolicy>,
    seeds: u32,
    minutes: f32,
}

fn parse(mut raw: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut args = Args {
        rules: Vec::new(),
        enemies: Vec::new(),
        tiers: Vec::new(),
        pins: Vec::new(),
        takes: Vec::new(),
        spends: Vec::new(),
        seeds: 3,
        minutes: 4.0,
    };
    while let Some(flag) = raw.next() {
        let mut value = || raw.next().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--rules" => args.rules.push(value()?.parse()?),
            "--enemies" => args.enemies.push(value()?.parse()?),
            "--tier" => args.tiers.push(value()?.parse()?),
            "--pin" => {
                let level: u8 = value()?.parse().map_err(|e| format!("--pin: {e}"))?;
                args.pins
                    .push(Difficulty::new(level).map_err(|e| format!("--pin: {e}"))?);
            }
            "--take" => args.takes.push(value()?.parse()?),
            "--spend" => args.spends.push(value()?.parse()?),
            "--seeds" => args.seeds = value()?.parse().map_err(|e| format!("--seeds: {e}"))?,
            "--minutes" => {
                args.minutes = value()?.parse().map_err(|e| format!("--minutes: {e}"))?
            }
            other => return Err(format!("unknown argument {other:?}")),
        }
    }
    if args.rules.is_empty() {
        args.rules = vec![PickupRules::Shards];
    }
    if args.enemies.is_empty() {
        args.enemies = vec![EnemyMix::Grunts, EnemyMix::All];
    }
    if args.tiers.is_empty() {
        args.tiers = SkillTier::ALL.to_vec();
    }
    if args.pins.is_empty() {
        args.pins = [4, 6, 8, 10]
            .into_iter()
            .filter_map(|l| Difficulty::new(l).ok())
            .collect();
    }
    if args.takes.is_empty() {
        args.takes = vec![
            PickupPolicy::Ignore,
            PickupPolicy::Greedy,
            PickupPolicy::Clear(150),
            PickupPolicy::Clear(300),
        ];
    }
    if args.spends.is_empty() {
        args.spends = std::iter::once(SpendPolicy::Hoard)
            .chain([1, 3, 5].into_iter().flat_map(|crowd| {
                [0, 40].map(|low_hp_pct| SpendPolicy::When {
                    crowd,
                    low_hp_pct,
                    bolts: false,
                })
            }))
            .collect();
    }
    Ok(args)
}

/// Under `shards-bolts`, spend rules watch bolts too: the blast clears them.
fn adapt(spend: SpendPolicy, rules: PickupRules) -> SpendPolicy {
    match spend {
        SpendPolicy::When {
            crowd, low_hp_pct, ..
        } => SpendPolicy::When {
            crowd,
            low_hp_pct,
            bolts: rules == PickupRules::ShardsBolts,
        },
        hoard => hoard,
    }
}

fn main() -> ExitCode {
    let args = match parse(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{e}\n{USAGE}");
            return ExitCode::FAILURE;
        }
    };
    // (situation, strategy) per config, so seeds can be pooled afterwards.
    let mut labels = Vec::new();
    let mut configs = Vec::new();
    for &rules in &args.rules {
        for &enemies in &args.enemies {
            for &tier in &args.tiers {
                for &pin in &args.pins {
                    let situation = format!("{rules} / {enemies} / {tier} @{pin}");
                    for &take in &args.takes {
                        // Spending means nothing without shards.
                        let spends = if rules.uses_shards() {
                            args.spends.as_slice()
                        } else {
                            &[SpendPolicy::Hoard]
                        };
                        for &spend in spends {
                            let spend = adapt(spend, rules);
                            let strategy = if rules.uses_shards() {
                                format!("take:{take} spend:{spend}")
                            } else {
                                format!("take:{take}")
                            };
                            for seed in 1..=args.seeds.max(1) {
                                labels.push((situation.clone(), strategy.clone()));
                                configs.push(SessionConfig {
                                    tier,
                                    seed,
                                    minutes: args.minutes,
                                    weapon_lock: None,
                                    enemies,
                                    pinned: Some(pin),
                                    pickup_rules: rules,
                                    pickup_policy: Some(take),
                                    spend: Some(spend),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    eprintln!("playing {} sessions", configs.len());
    let logs = play_many(&configs);

    let mut outcomes: Vec<Outcome> = Vec::new();
    for ((situation, strategy), log) in labels.into_iter().zip(&logs) {
        let summary = Summary::of(log);
        match outcomes
            .iter_mut()
            .find(|o| o.situation == situation && o.strategy == strategy)
        {
            Some(o) => {
                o.waves += summary.waves;
                o.deaths += summary.deaths;
            }
            None => outcomes.push(Outcome {
                situation,
                strategy,
                waves: summary.waves,
                deaths: summary.deaths,
            }),
        }
    }
    let judged = verdicts(&outcomes);
    println!(
        "{} seeds x {} simulated minutes per strategy and situation; deaths per wave at a fixed difficulty.\n\
         Bots measure the mechanical outcome of a macro choice, not how a plan feels.\n",
        args.seeds, args.minutes
    );
    println!(
        "Strategies, least extra deaths first (a strategy that ties the best everywhere makes the choice moot):\n"
    );
    println!("{}", standing_table(&standings(&outcomes, &judged)));
    println!("Situations (left out: every strategy below 5% or above 95% deaths):\n");
    println!("{}", verdict_table(&judged));
    ExitCode::SUCCESS
}

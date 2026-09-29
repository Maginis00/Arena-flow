//! Playtest report: every skill tier plays the game headless and the flow
//! director's response is printed as a Markdown table.
//!
//! ```sh
//! cargo run --example playtest
//! cargo run --example playtest -- --minutes 20 --seeds 3 --tier expert
//! ```
//!
//! `--watch` instead opens the game window and lets one bot play in real time
//! (the first `--tier`, default expert; `--seeds N` picks the seed). Without
//! `--release` the debug overlay is shown too.
//!
//! ```sh
//! cargo run --example playtest -- --watch --tier skilled --weapon melee
//! ```

use flow_arena::playtest::{
    SessionConfig, SessionLog, SkillTier, Summary, WatchConfig, play, table, watch,
};
use flow_arena::weapons::api::WeaponKind;
use std::process::ExitCode;
use std::thread;

struct Args {
    minutes: f32,
    seeds: u32,
    tiers: Vec<SkillTier>,
    watch: bool,
    weapon: Option<WeaponKind>,
}

fn weapon(name: &str) -> Result<WeaponKind, String> {
    match name.to_ascii_lowercase().as_str() {
        "projectile" => Ok(WeaponKind::Projectile),
        "hitscan" => Ok(WeaponKind::Hitscan),
        "melee" => Ok(WeaponKind::Melee),
        _ => Err(format!(
            "unknown weapon {name:?}; expected projectile, hitscan or melee"
        )),
    }
}

fn parse(mut raw: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut args = Args {
        minutes: 10.0,
        seeds: 1,
        tiers: Vec::new(),
        watch: false,
        weapon: None,
    };
    while let Some(flag) = raw.next() {
        let mut value = || raw.next().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--minutes" => {
                args.minutes = value()?.parse().map_err(|e| format!("--minutes: {e}"))?
            }
            "--seeds" => args.seeds = value()?.parse().map_err(|e| format!("--seeds: {e}"))?,
            "--tier" => args.tiers.push(value()?.parse()?),
            "--watch" => args.watch = true,
            "--weapon" => args.weapon = Some(weapon(&value()?)?),
            other => return Err(format!("unknown argument {other:?}")),
        }
    }
    if args.weapon.is_some() && !args.watch {
        return Err("--weapon only works with --watch".to_owned());
    }
    if args.tiers.is_empty() {
        args.tiers = if args.watch {
            vec![SkillTier::Expert]
        } else {
            SkillTier::ALL.to_vec()
        };
    }
    Ok(args)
}

fn main() -> ExitCode {
    let args = match parse(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(e) => {
            eprintln!(
                "{e}\nusage: playtest [--minutes N] [--seeds N] [--tier NAME]...\n       \
                 playtest --watch [--tier NAME] [--seeds N] [--weapon NAME]"
            );
            return ExitCode::FAILURE;
        }
    };
    if args.watch {
        let config = WatchConfig {
            tier: args.tiers.first().copied().unwrap_or(SkillTier::Expert),
            seed: args.seeds.max(1),
            weapon: args.weapon,
        };
        return if watch(config).is_success() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        };
    }
    let configs: Vec<SessionConfig> = args
        .tiers
        .iter()
        .flat_map(|&tier| {
            (1..=args.seeds.max(1)).map(move |seed| SessionConfig {
                tier,
                seed,
                minutes: args.minutes,
            })
        })
        .collect();
    // Sessions are independent and deterministic; run them side by side.
    let logs: Vec<SessionLog> = thread::scope(|scope| {
        let handles: Vec<_> = configs
            .iter()
            .map(|&config| scope.spawn(move || play(config)))
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap_or_default())
            .collect()
    });

    let named: Vec<(String, &SessionLog)> = configs
        .iter()
        .zip(&logs)
        .map(|(c, log)| (format!("{} #{}", c.tier, c.seed), log))
        .collect();
    println!("{} simulated minutes per bot\n", args.minutes);
    println!("{}", table(&named));
    println!("Difficulty per wave (x = died):\n");
    for (name, log) in &named {
        println!("- {name}: {}", Summary::of(log).trajectory);
    }
    ExitCode::SUCCESS
}

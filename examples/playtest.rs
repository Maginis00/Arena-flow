//! Playtest report: every skill tier plays the game headless and the flow
//! director's response is printed as Markdown tables, overall and per weapon.
//!
//! ```sh
//! cargo run --release --example playtest
//! cargo run --release --example playtest -- --minutes 20 --seeds 3 --tier expert
//! cargo run --release --example playtest -- --weapon melee --weapon hitscan
//! cargo run --release --example playtest -- --matrix   # every tier x every weapon
//! ```

use flow_arena::playtest::{
    SessionConfig, SessionLog, SkillTier, Summary, WEAPONS, play, table, weapon_table,
};
use flow_arena::weapons::api::WeaponKind;
use std::process::ExitCode;
use std::thread;

const USAGE: &str = "usage: playtest [--minutes N] [--seeds N] [--tier NAME]... \
                     [--weapon projectile|hitscan|melee]... [--matrix]";

struct Args {
    minutes: f32,
    seeds: u32,
    tiers: Vec<SkillTier>,
    /// `None` is the tier's own weapon choice.
    weapons: Vec<Option<WeaponKind>>,
}

fn parse_weapon(name: &str) -> Result<WeaponKind, String> {
    match name.to_ascii_lowercase().as_str() {
        "projectile" | "1" => Ok(WeaponKind::Projectile),
        "hitscan" | "2" => Ok(WeaponKind::Hitscan),
        "melee" | "3" => Ok(WeaponKind::Melee),
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
        weapons: Vec::new(),
    };
    while let Some(flag) = raw.next() {
        let mut value = || raw.next().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--minutes" => {
                args.minutes = value()?.parse().map_err(|e| format!("--minutes: {e}"))?
            }
            "--seeds" => args.seeds = value()?.parse().map_err(|e| format!("--seeds: {e}"))?,
            "--tier" => args.tiers.push(value()?.parse()?),
            "--weapon" => args.weapons.push(Some(parse_weapon(&value()?)?)),
            "--matrix" => {
                args.weapons = std::iter::once(None).chain(WEAPONS.map(Some)).collect();
            }
            other => return Err(format!("unknown argument {other:?}")),
        }
    }
    if args.tiers.is_empty() {
        args.tiers = SkillTier::ALL.to_vec();
    }
    if args.weapons.is_empty() {
        args.weapons.push(None);
    }
    Ok(args)
}

fn main() -> ExitCode {
    let args = match parse(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{e}\n{USAGE}");
            return ExitCode::FAILURE;
        }
    };
    let mut configs = Vec::new();
    for &tier in &args.tiers {
        for &weapon_lock in &args.weapons {
            for seed in 1..=args.seeds.max(1) {
                configs.push(SessionConfig {
                    tier,
                    seed,
                    minutes: args.minutes,
                    weapon_lock,
                });
            }
        }
    }
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
        .map(|(c, log)| {
            let lock = c
                .weapon_lock
                .map_or_else(String::new, |w| format!(" [{w} only]"));
            (format!("{} #{}{lock}", c.tier, c.seed), log)
        })
        .collect();
    println!("{} simulated minutes per bot\n", args.minutes);
    println!("{}", table(&named));
    println!("Per weapon (held = share of wave time; damage and deaths count while holding it):\n");
    println!("{}", weapon_table(&named));
    println!("Difficulty per wave (x = died):\n");
    for (name, log) in &named {
        println!("- {name}: {}", Summary::of(log).trajectory);
    }
    ExitCode::SUCCESS
}

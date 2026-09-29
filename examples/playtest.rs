//! Playtest report: every skill tier plays the game headless and the flow
//! director's response is printed as Markdown tables, overall and per weapon.
//!
//! ```sh
//! cargo run --release --example playtest
//! cargo run --release --example playtest -- --minutes 20 --seeds 3 --tier expert
//! cargo run --release --example playtest -- --weapon melee --weapon hitscan
//! cargo run --release --example playtest -- --matrix   # every tier x every weapon
//! cargo run --release --example playtest -- --human       # add your own sessions
//! cargo run --release --example playtest -- --human-only  # only your own sessions
//! ```
//!
//! Your sessions are the files the game writes to `playtests/` when you play
//! it in a window (`cargo run`), one file per launch.

use flow_arena::playtest::{
    Named, SessionConfig, SkillTier, Summary, pickup_table, play, table, weapon_table,
};
use flow_arena::telemetry::api::{SESSION_DIR, SessionRecord, WEAPONS, read_session_file};
use flow_arena::weapons::api::WeaponKind;
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;

const USAGE: &str = "usage: playtest [--minutes N] [--seeds N] [--tier NAME]... \
                     [--weapon projectile|hitscan|melee]... [--matrix] [--human] [--human-only]";

struct Args {
    minutes: f32,
    seeds: u32,
    tiers: Vec<SkillTier>,
    /// `None` is the tier's own weapon choice.
    weapons: Vec<Option<WeaponKind>>,
    human: bool,
    bots: bool,
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
        human: false,
        bots: true,
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
            "--human" => args.human = true,
            "--human-only" => (args.human, args.bots) = (true, false),
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
    let humans = if args.human {
        match human_sessions() {
            Ok(found) => found,
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        Vec::new()
    };
    let mut configs = Vec::new();
    let tiers = if args.bots {
        args.tiers.as_slice()
    } else {
        &[]
    };
    for &tier in tiers {
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
    let logs: Vec<SessionRecord> = thread::scope(|scope| {
        let handles: Vec<_> = configs
            .iter()
            .map(|&config| scope.spawn(move || play(config)))
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap_or_default())
            .collect()
    });

    let mut named: Vec<Named> = humans
        .iter()
        .map(|(name, session)| (name.clone(), session))
        .collect();
    named.extend(configs.iter().zip(&logs).map(|(c, log)| {
        let lock = c
            .weapon_lock
            .map_or_else(String::new, |w| format!(" [{w} only]"));
        (format!("{} #{}{lock}", c.tier, c.seed), log)
    }));
    if args.bots {
        println!("{} simulated minutes per bot\n", args.minutes);
    }
    println!("{}", table(&named));
    println!("Per weapon (held = share of wave time; damage and deaths count while holding it):\n");
    println!("{}", weapon_table(&named));
    println!("Pickups (hp is what the player had just before taking it):\n");
    println!("{}", pickup_table(&named));
    println!("Difficulty per wave (x = died):\n");
    for (name, log) in &named {
        println!("- {name}: {}", Summary::of(log).trajectory);
    }
    ExitCode::SUCCESS
}

/// Every session file in `playtests/`, oldest first, named "you: <file>".
fn human_sessions() -> Result<Vec<(String, SessionRecord)>, String> {
    let dir = PathBuf::from(SESSION_DIR);
    let entries = std::fs::read_dir(&dir).map_err(|e| {
        format!(
            "no session files in {}/ ({e}); play with `cargo run` first",
            dir.display()
        )
    })?;
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "jsonl"))
        .collect();
    paths.sort();
    paths
        .iter()
        .map(|path| {
            let stem = path
                .file_stem()
                .map_or_else(String::new, |s| s.to_string_lossy().into_owned());
            read_session_file(path).map(|session| (format!("you: {stem}"), session))
        })
        .filter(|r| r.as_ref().map_or(true, |(_, s)| !s.waves.is_empty()))
        .collect::<Result<Vec<_>, _>>()
        .and_then(|found| {
            if found.is_empty() {
                Err(format!(
                    "no finished waves in {}/ yet; play with `cargo run` first",
                    dir.display()
                ))
            } else {
                Ok(found)
            }
        })
}

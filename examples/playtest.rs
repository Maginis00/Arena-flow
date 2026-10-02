//! Playtest report: every skill tier plays the game headless and the flow
//! director's response is printed as Markdown tables, overall and per weapon.
//!
//! ```sh
//! cargo run --example playtest
//! cargo run --example playtest -- --minutes 20 --seeds 3 --tier expert
//! cargo run --example playtest -- --weapon melee --weapon hitscan
//! cargo run --example playtest -- --matrix   # every tier x every weapon
//! cargo run --example playtest -- --enemies shooter --enemies all  # enemy kinds mixed in
//! cargo run --example playtest -- --human       # add your own sessions
//! cargo run --example playtest -- --human-only  # only your own sessions
//! cargo run --example playtest -- --pin 5 --pin 7.5  # no director: fixed difficulty
//! cargo run --example playtest -- --sweep            # pinned at every level 1 to 10
//! cargo run --example playtest -- --jsonl out/       # also save each bot session
//! ```
//!
//! Your sessions are the files the game writes to `playtests/` when you play
//! it in a window (`cargo run`), one file per launch.
//!
//! `--watch` instead opens the game window and lets one bot play in real time
//! (the first `--tier`, default expert; the first `--weapon`, if any; `--seeds N`
//! picks the seed). Without `--release` the debug overlay is shown too.
//!
//! ```sh
//! cargo run --example playtest -- --watch --tier skilled --weapon melee
//! cargo run --example playtest -- --watch --enemies all
//! ```
//!
//! `--enemies` takes grunt (the default), shooter, brute, charger, summoner
//! or all; repeat it to compare mixes. To play a mix yourself, set
//! `ARENA_ENEMIES=all` before `cargo run`.
//!
//! `--watch-all` opens one window per tier (or per `--tier` given, at most
//! four), each a separate process of this same binary filling one quarter of
//! the primary monitor's work area: novice top left, casual top right, skilled
//! bottom left, expert bottom right. Close them all with Ctrl+C in the
//! terminal, or one at a time with Alt+F4.
//!
//! ```sh
//! cargo run --example playtest -- --watch-all
//! ```

use flow_arena::enemies::api::EnemyMix;
use flow_arena::flow_director::api::Difficulty;
use flow_arena::playtest::{
    Named, Quadrant, SessionConfig, SkillTier, Summary, WatchConfig, flow_table, pickup_table,
    play, table, watch, weapon_table,
};
use flow_arena::telemetry::api::{
    SESSION_DIR, SessionRecord, WEAPONS, read_session_file, save_session,
};
use flow_arena::weapons::api::WeaponKind;
use std::path::PathBuf;
use std::process::{Command, ExitCode};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

const USAGE: &str = "usage: playtest [--minutes N] [--seeds N] [--tier NAME]... \
                     [--weapon projectile|hitscan|melee]... [--matrix] [--enemies MIX]... [--human] [--human-only]\n       \
                     [--pin LEVEL]... [--sweep] [--jsonl DIR]\n       \
                     playtest --watch [--tier NAME] [--seeds N] [--weapon NAME] [--enemies MIX] [--quadrant NAME]\n       \
                     playtest --watch-all [--tier NAME]... [--seeds N] [--weapon NAME] [--enemies MIX]";

struct Args {
    minutes: f32,
    seeds: u32,
    tiers: Vec<SkillTier>,
    /// `None` is the tier's own weapon choice.
    weapons: Vec<Option<WeaponKind>>,
    enemies: Vec<EnemyMix>,
    /// `None` is the director steering.
    pins: Vec<Option<Difficulty>>,
    jsonl: Option<PathBuf>,
    human: bool,
    bots: bool,
    watch: bool,
    watch_all: bool,
    quadrant: Option<Quadrant>,
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

/// A difficulty level in quarter steps, e.g. `7.25`.
fn parse_level(text: &str) -> Result<Difficulty, String> {
    let level: f32 = text.parse().map_err(|e| format!("--pin: {e}"))?;
    let quarters = (level * f32::from(Difficulty::QUARTERS_PER_LEVEL)).round();
    if !(0.0..=255.0).contains(&quarters) {
        return Err(format!("--pin {text}: expected a level from 1 to 10"));
    }
    // In u8 range after the check above.
    Difficulty::from_quarters(quarters as u8).map_err(|e| format!("--pin {text}: {e}"))
}

fn parse(mut raw: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut args = Args {
        minutes: 10.0,
        seeds: 1,
        tiers: Vec::new(),
        weapons: Vec::new(),
        enemies: Vec::new(),
        pins: Vec::new(),
        jsonl: None,
        human: false,
        bots: true,
        watch: false,
        watch_all: false,
        quadrant: None,
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
            "--enemies" => args.enemies.push(value()?.parse()?),
            "--pin" => args.pins.push(Some(parse_level(&value()?)?)),
            "--sweep" => {
                args.pins = (1..=10)
                    .filter_map(|l| Difficulty::new(l).ok().map(Some))
                    .collect();
            }
            "--jsonl" => args.jsonl = Some(PathBuf::from(value()?)),
            "--watch" => args.watch = true,
            "--watch-all" => args.watch_all = true,
            "--quadrant" => args.quadrant = Some(value()?.parse()?),
            "--human" => args.human = true,
            "--human-only" => (args.human, args.bots) = (true, false),
            other => return Err(format!("unknown argument {other:?}")),
        }
    }
    if args.watch_all && args.tiers.len() > Quadrant::ALL.len() {
        return Err("--watch-all shows at most four tiers".to_owned());
    }
    if args.tiers.is_empty() {
        args.tiers = if args.watch {
            vec![SkillTier::Expert]
        } else {
            SkillTier::ALL.to_vec()
        };
    }
    if args.weapons.is_empty() {
        args.weapons.push(None);
    }
    if args.enemies.is_empty() {
        args.enemies.push(EnemyMix::Grunts);
    }
    if args.pins.is_empty() {
        args.pins.push(None);
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
    if args.watch_all {
        return watch_all(&args);
    }
    if args.watch {
        let config = WatchConfig {
            tier: args.tiers.first().copied().unwrap_or(SkillTier::Expert),
            seed: args.seeds.max(1),
            weapon: args.weapons.iter().find_map(|w| *w),
            quadrant: args.quadrant,
            enemies: args.enemies.first().copied().unwrap_or_default(),
        };
        return if watch(config).is_success() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        };
    }
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
    for &enemies in &args.enemies {
        for &tier in tiers {
            for &weapon_lock in &args.weapons {
                for &pinned in &args.pins {
                    for seed in 1..=args.seeds.max(1) {
                        configs.push(SessionConfig {
                            tier,
                            seed,
                            minutes: args.minutes,
                            weapon_lock,
                            enemies,
                            pinned,
                        });
                    }
                }
            }
        }
    }
    let logs = play_all(&configs);

    let mut named: Vec<Named> = humans
        .iter()
        .map(|(name, session)| (name.clone(), session))
        .collect();
    named.extend(configs.iter().zip(&logs).map(|(c, log)| {
        let lock = c
            .weapon_lock
            .map_or_else(String::new, |w| format!(" [{w} only]"));
        let mix = match c.enemies {
            EnemyMix::Grunts => String::new(),
            mix => format!(" ({mix})"),
        };
        let pin = c.pinned.map_or_else(String::new, |d| format!(" @{d}"));
        (format!("{} #{}{lock}{mix}{pin}", c.tier, c.seed), log)
    }));
    if let Some(dir) = &args.jsonl
        && let Err(e) = save_all(dir, &configs, &logs)
    {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }
    if args.bots {
        println!("{} simulated minutes per bot\n", args.minutes);
    }
    println!("Flow (each wave's hp at stake on the flow curve; mechanical pressure only):\n");
    println!("{}", flow_table(&named));
    println!("Director and play:\n");
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

/// Sessions are independent and deterministic, so they run side by side, one
/// worker per core: a thread per session makes the cores fight over Bevy's
/// shared task pool once a sweep has a hundred sessions.
fn play_all(configs: &[SessionConfig]) -> Vec<SessionRecord> {
    let workers = thread::available_parallelism().map_or(4, |n| n.get());
    let next = AtomicUsize::new(0);
    let mut logs = vec![SessionRecord::default(); configs.len()];
    let finished: Vec<(usize, SessionRecord)> = thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|_| {
                scope.spawn(|| {
                    let mut done = Vec::new();
                    loop {
                        let i = next.fetch_add(1, Ordering::Relaxed);
                        let Some(&config) = configs.get(i) else {
                            return done;
                        };
                        done.push((i, play(config)));
                    }
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap_or_default())
            .collect()
    });
    for (i, log) in finished {
        logs[i] = log;
    }
    logs
}

/// Starts one `--watch` process of this binary per tier, each in its own
/// quadrant, and waits for all of them. Separate processes rather than
/// windows of one app: each bot keeps its own world, clock and seed, exactly
/// as in a single `--watch`.
fn watch_all(args: &Args) -> ExitCode {
    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(e) => {
            eprintln!("cannot find this program to start the bots: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut children = Vec::new();
    for (tier, quadrant) in args.tiers.iter().zip(Quadrant::ALL) {
        let mut command = Command::new(&exe);
        command.args(["--watch", "--tier", &tier.to_string()]);
        command.args(["--quadrant", &quadrant.to_string()]);
        command.args(["--seeds", &args.seeds.max(1).to_string()]);
        if let Some(mix) = args.enemies.first() {
            let name = match mix {
                EnemyMix::Grunts => "grunt".to_owned(),
                EnemyMix::With(kind) => kind.to_string(),
                EnemyMix::All => "all".to_owned(),
            };
            command.args(["--enemies", &name]);
        }
        if let Some(weapon) = args.weapons.iter().find_map(|w| *w) {
            command.args(["--weapon", weapon_name(weapon)]);
        }
        match command.spawn() {
            Ok(child) => children.push(child),
            Err(e) => eprintln!("could not start the {tier} bot: {e}"),
        }
    }
    let started = children.len();
    let ok = children
        .into_iter()
        .filter_map(|mut child| child.wait().ok())
        .filter(|status| status.success())
        .count();
    if started == args.tiers.len() && ok == started {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// One file per bot session: `<tier>-<weapon or own>-<pinned or director>-<seed>.jsonl`.
fn save_all(
    dir: &std::path::Path,
    configs: &[SessionConfig],
    logs: &[SessionRecord],
) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for (c, log) in configs.iter().zip(logs) {
        let weapon = c.weapon_lock.map_or("own", weapon_name);
        let pin = c
            .pinned
            .map_or_else(|| "director".to_owned(), |d| d.to_string());
        let mix = match c.enemies {
            EnemyMix::Grunts => "grunts".to_owned(),
            EnemyMix::With(kind) => kind.to_string(),
            EnemyMix::All => "all".to_owned(),
        };
        let name = format!("{}-{weapon}-{mix}-{pin}-{}.jsonl", c.tier, c.seed);
        save_session(&dir.join(name), log)?;
    }
    Ok(())
}

/// The name `parse_weapon` reads back (Display says "melee arc").
const fn weapon_name(weapon: WeaponKind) -> &'static str {
    match weapon {
        WeaponKind::Projectile => "projectile",
        WeaponKind::Hitscan => "hitscan",
        WeaponKind::Melee => "melee",
    }
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

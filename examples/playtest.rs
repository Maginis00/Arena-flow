//! Playtest report: every skill tier plays the game headless and the flow
//! director's response is printed as a Markdown table.
//!
//! ```sh
//! cargo run --release --example playtest
//! cargo run --release --example playtest -- --minutes 20 --seeds 3 --tier expert
//! ```

use flow_arena::playtest::{SessionConfig, SessionLog, SkillTier, Summary, play, table};
use std::process::ExitCode;
use std::thread;

struct Args {
    minutes: f32,
    seeds: u32,
    tiers: Vec<SkillTier>,
}

fn parse(mut raw: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut args = Args {
        minutes: 10.0,
        seeds: 1,
        tiers: Vec::new(),
    };
    while let Some(flag) = raw.next() {
        let mut value = || raw.next().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--minutes" => {
                args.minutes = value()?.parse().map_err(|e| format!("--minutes: {e}"))?
            }
            "--seeds" => args.seeds = value()?.parse().map_err(|e| format!("--seeds: {e}"))?,
            "--tier" => args.tiers.push(value()?.parse()?),
            other => return Err(format!("unknown argument {other:?}")),
        }
    }
    if args.tiers.is_empty() {
        args.tiers = SkillTier::ALL.to_vec();
    }
    Ok(args)
}

fn main() -> ExitCode {
    let args = match parse(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{e}\nusage: playtest [--minutes N] [--seeds N] [--tier NAME]...");
            return ExitCode::FAILURE;
        }
    };
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

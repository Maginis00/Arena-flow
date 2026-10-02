//! Play flow_arena in steps, as an outside agent (an LLM, or a person typing).
//! The simulation stands still between calls: each call appends orders to the
//! run's file, replays the whole run headless (the game is deterministic) and
//! prints what the player sees now.
//!
//! ```sh
//! cargo run --release --example agent_play -- runs/a start novice
//! cargo run --release --example agent_play -- runs/a "ne 2 fire nearest for 0.3"
//! cargo run --release --example agent_play -- runs/a "w for 0.2; sw 3 fire densest for 0.4"
//! cargo run --release --example agent_play -- runs/a feel -1 "too calm, nothing came close"
//! cargo run --release --example agent_play -- runs/a show
//! cargo run --release --example agent_play -- runs/a journal
//! ```
//!
//! An order is words in any order, all optional: a direction (`n` `ne` `e`
//! `se` `s` `sw` `w` `nw` `stop`) or `to X Y`; a weapon `1` `2` `3`;
//! `fire nearest` (default), `fire densest`, `fire DEG` or `hold`;
//! `skill LEVEL` (0 to 3); and `for SECS` (default 0.3, at most 3). Several
//! orders split by `;` play back to back without a look in between.
//! `feel N NOTE` rates the wave that just ended on the flow curve, -2 bored
//! to +2 overwhelmed; `journal` prints every wave's numbers next to those
//! ratings.
//!
//! `start TIER` picks the starting hands (novice, casual, skilled, expert, or a
//! level from 0 to 3 in between; default skilled): the reaction time and aim
//! error that carry out every `fire` order. The brain is the agent's; the hands
//! are a bot's. The word `skill LEVEL` in an order changes the hands from then
//! on, so a run can play a player who improves over time. The journal shows
//! the skill and the pickups taken for every wave.

use flow_arena::playtest::{Feel, Order, SkillTier, journal, render, replay};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::str::FromStr;

const ORDERS_FILE: &str = "orders.txt";
const FEEL_PREFIX: &str = "# feel ";
const HANDS_PREFIX: &str = "# hands ";
const USAGE: &str =
    "usage: agent_play DIR start [TIER] | show | journal | feel N NOTE | \"ORDER[; ORDER...]\"";

/// The run so far: orders and the agent's ratings, from `DIR/orders.txt`.
struct Run {
    file: PathBuf,
    /// Starting hand skill, 0 novice to 3 expert.
    hands: f32,
    orders: Vec<Order>,
    feels: Vec<Feel>,
}

impl Run {
    fn load(dir: &Path) -> Result<Self, String> {
        let file = dir.join(ORDERS_FILE);
        let text = fs::read_to_string(&file)
            .map_err(|e| format!("{}: {e} (start a run first)", file.display()))?;
        let mut run = Self {
            file,
            hands: SkillTier::Skilled.level(),
            orders: Vec::new(),
            feels: Vec::new(),
        };
        for (n, line) in text.lines().enumerate() {
            if let Some(tier) = line.strip_prefix(HANDS_PREFIX) {
                run.hands = parse_skill(tier)?;
            } else if let Some(feel) = line.strip_prefix(FEEL_PREFIX) {
                run.feels
                    .push(parse_feel(feel).map_err(|e| format!("line {}: {e}", n + 1))?);
            } else if !line.starts_with('#') {
                run.orders
                    .push(line.parse().map_err(|e| format!("line {}: {e}", n + 1))?);
            }
        }
        Ok(run)
    }

    fn append(&self, lines: &[String]) -> Result<(), String> {
        let mut text = fs::read_to_string(&self.file).map_err(|e| e.to_string())?;
        for line in lines {
            text.push_str(line);
            text.push('\n');
        }
        fs::write(&self.file, text).map_err(|e| e.to_string())
    }
}

/// A tier name or a number from 0 (novice) to 3 (expert).
fn parse_skill(text: &str) -> Result<f32, String> {
    if let Ok(tier) = SkillTier::from_str(text) {
        return Ok(tier.level());
    }
    text.parse::<f32>()
        .ok()
        .filter(|level| (0.0..=3.0).contains(level))
        .ok_or_else(|| format!("hands: {text:?} is not a tier name or a level from 0 to 3"))
}

/// `AFTER VALUE NOTE...`, as written by `feel`.
fn parse_feel(text: &str) -> Result<Feel, String> {
    let mut parts = text.splitn(3, ' ');
    let after_waves = parts
        .next()
        .and_then(|w| w.parse().ok())
        .ok_or("feel: bad wave count")?;
    let value = parts
        .next()
        .and_then(|v| v.parse().ok())
        .ok_or("feel: bad value")?;
    let note = parts.next().unwrap_or_default().to_owned();
    Ok(Feel {
        after_waves,
        value,
        note,
    })
}

fn run(args: &[String]) -> Result<String, String> {
    let (dir, command) = match args {
        [dir, rest @ ..] if !rest.is_empty() => (Path::new(dir), rest),
        _ => return Err(USAGE.into()),
    };
    match command[0].as_str() {
        "start" => {
            let hands = match command.get(1) {
                Some(level) => parse_skill(level)?,
                None => SkillTier::Skilled.level(),
            };
            fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            fs::write(dir.join(ORDERS_FILE), format!("{HANDS_PREFIX}{hands}\n"))
                .map_err(|e| e.to_string())?;
            Ok(render(&replay(hands, &[], &[]).frame))
        }
        "show" => {
            let run = Run::load(dir)?;
            Ok(render(&replay(run.hands, &run.orders, &[]).frame))
        }
        "journal" => {
            let run = Run::load(dir)?;
            let played = replay(run.hands, &run.orders, &[]);
            Ok(journal(&played.record, &played.wave_skills, &run.feels))
        }
        "feel" => {
            let run = Run::load(dir)?;
            let value: i8 = command
                .get(1)
                .and_then(|v| v.parse().ok())
                .filter(|v| (-2..=2).contains(v))
                .ok_or("feel needs a value from -2 (bored) to 2 (overwhelmed)")?;
            let note = command[2..].join(" ").replace('\n', " ");
            let record = replay(run.hands, &run.orders, &[]).record;
            let Some(last) = record.waves.last() else {
                return Err("no wave has ended yet".into());
            };
            let waves = record.waves.len();
            run.append(&[format!("{FEEL_PREFIX}{waves} {value} {note}")])?;
            Ok(format!(
                "noted for wave {} try {}: {value:+} {note}\n",
                last.report.index, last.report.attempt
            ))
        }
        _ => {
            let run = Run::load(dir)?;
            let latest = command
                .join(" ")
                .split(';')
                .map(|o| o.trim().parse::<Order>())
                .collect::<Result<Vec<_>, _>>()?;
            let frame = replay(run.hands, &run.orders, &latest).frame;
            run.append(&latest.iter().map(Order::to_string).collect::<Vec<_>>())?;
            Ok(render(&frame))
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

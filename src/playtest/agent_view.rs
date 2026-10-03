//! What an outside agent reads between steps: the game state as text, with a
//! coarse map and exact offsets to the closest enemies. Pure; unit-tested.

use crate::flow_director::wave_risk;
use crate::pickups::{Effects, PickupKind};
use crate::telemetry::SessionRecord;
use crate::waves::{WaveReport, WaveSpec};
use crate::weapons::WeaponKind;
use bevy::math::Vec2;
use std::fmt::Write as _;

/// World units per map cell.
const CELL: f32 = 40.0;
/// How many enemies get exact offsets.
const LISTED_ENEMIES: usize = 8;

/// What happened during the last step.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StepEvents {
    pub damage_taken: u32,
    pub hits_taken: u32,
    pub healed: u32,
    pub shots: u32,
    pub kills: u32,
    pub pickups: Vec<PickupKind>,
    pub died: bool,
}

/// A wave that ended during this call, with the director's answer.
#[derive(Debug, Clone, PartialEq)]
pub struct FinishedWave {
    pub report: WaveReport,
    /// Difficulty chosen for the next wave and the director's reason.
    pub next: Option<(f32, String)>,
}

/// The game at the end of a step.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Frame {
    pub secs: f32,
    pub half_extents: Vec2,
    /// The wave in play or last played.
    pub wave: Option<WaveSpec>,
    pub in_wave: bool,
    pub still_to_spawn: u32,
    /// Position, hp, max hp. `None` while dead.
    pub player: Option<(Vec2, u32, u32)>,
    /// Hand skill, 0 novice to 3 expert.
    pub skill: f32,
    pub weapon: WeaponKind,
    pub effects: Effects,
    pub enemies: Vec<Vec2>,
    pub pickups: Vec<(Vec2, PickupKind)>,
    pub step: StepEvents,
    pub finished: Vec<FinishedWave>,
}

fn effects_text(e: &Effects) -> String {
    let mut parts = Vec::new();
    let mut scale = |name: &str, value: f32| {
        if (value - 1.0).abs() > 1e-3 {
            parts.push(format!("{name} x{value:.2}"));
        }
    };
    scale("fire rate", e.fire_rate);
    scale("damage dealt", e.outgoing_damage);
    scale("damage taken", e.incoming_damage);
    scale("move speed", e.move_speed);
    if e.weapons_locked {
        parts.push("WEAPONS LOCKED".into());
    }
    if parts.is_empty() {
        "none".into()
    } else {
        parts.join(", ")
    }
}

const fn pickup_glyph(kind: PickupKind) -> char {
    match kind {
        PickupKind::Overdrive => 'O',
        PickupKind::Heavy => 'H',
        PickupKind::Mend => 'M',
    }
}

/// The arena as characters, north up: `@` you, `1`-`9` enemies in a cell
/// (`+` for more), `O` `H` `M` pickups.
fn map(frame: &Frame) -> String {
    let half = frame.half_extents;
    let cols = ((half.x * 2.0) / CELL).ceil().max(1.0) as usize;
    let rows = ((half.y * 2.0) / CELL).ceil().max(1.0) as usize;
    let cell = |at: Vec2| {
        let col = ((at.x + half.x) / CELL)
            .floor()
            .clamp(0.0, cols as f32 - 1.0) as usize;
        let row = ((half.y - at.y) / CELL)
            .floor()
            .clamp(0.0, rows as f32 - 1.0) as usize;
        (row, col)
    };
    let mut counts = vec![vec![0u32; cols]; rows];
    for at in &frame.enemies {
        let (r, c) = cell(*at);
        counts[r][c] += 1;
    }
    let mut grid: Vec<Vec<char>> = counts
        .iter()
        .map(|row| {
            row.iter()
                .map(|n| match n {
                    0 => '.',
                    1..=9 => char::from_digit(*n, 10).unwrap_or('+'),
                    _ => '+',
                })
                .collect()
        })
        .collect();
    for (at, kind) in &frame.pickups {
        let (r, c) = cell(*at);
        if grid[r][c] == '.' {
            grid[r][c] = pickup_glyph(*kind);
        }
    }
    if let Some((at, ..)) = frame.player {
        let (r, c) = cell(at);
        grid[r][c] = '@';
    }
    let border = format!("+{}+\n", "-".repeat(cols));
    let mut out = border.clone();
    for row in grid {
        out.push('|');
        out.extend(row);
        out.push_str("|\n");
    }
    out.push_str(&border);
    out
}

fn wave_line(out: &mut String, w: &FinishedWave) {
    let r = &w.report;
    let outcome = if r.player_died { "DIED" } else { "cleared" };
    // Writing to a String cannot fail.
    let _ = write!(
        out,
        "== wave {} try {} {outcome} at difficulty {:.2}: {:.1}s, hp {} -> lowest {}, \
         {} damage in {} hits, {}/{} killed",
        r.index,
        r.attempt,
        r.difficulty.level(),
        r.duration_secs,
        r.start_hp,
        r.lowest_hp,
        r.damage_taken,
        r.hits_taken,
        r.enemies_killed,
        r.enemies_spawned,
    );
    if let Some((level, reason)) = &w.next {
        let _ = write!(out, "; next difficulty {level:.2} ({reason})");
    }
    out.push('\n');
}

/// The whole text an agent reads after a step.
pub fn render(frame: &Frame) -> String {
    let mut out = String::new();
    for w in &frame.finished {
        wave_line(&mut out, w);
    }
    let _ = write!(out, "t {:.2}s | ", frame.secs);
    match frame.wave {
        Some(spec) => {
            let phase = if frame.in_wave {
                "in play"
            } else {
                "between waves"
            };
            let _ = writeln!(
                out,
                "wave {} try {} {phase} | difficulty {:.2} | enemy speed {:.0}, contact damage {}",
                spec.index,
                spec.attempt,
                spec.difficulty.level(),
                spec.enemy_speed,
                spec.contact_damage,
            );
        }
        None => out.push_str("before the first wave\n"),
    }
    match frame.player {
        Some((at, hp, max)) => {
            let _ = writeln!(
                out,
                "you at ({:.0}, {:.0}) | hp {hp}/{max} | weapon {} | hand skill {:.2} | effects: {}",
                at.x,
                at.y,
                frame.weapon,
                frame.skill,
                effects_text(&frame.effects),
            );
        }
        None => out.push_str("you are dead; the wave restarts after the pause\n"),
    }
    let s = &frame.step;
    let _ = write!(
        out,
        "last step: -{} hp in {} hits, +{} healed, {} shots, {} kills",
        s.damage_taken, s.hits_taken, s.healed, s.shots, s.kills
    );
    for kind in &s.pickups {
        let _ = write!(out, ", took {kind}");
    }
    if s.died {
        out.push_str(", YOU DIED");
    }
    let _ = writeln!(
        out,
        "\nenemies: {} on field, {} still to spawn",
        frame.enemies.len(),
        frame.still_to_spawn
    );
    out.push_str(&map(frame));
    if let Some((own, ..)) = frame.player {
        let mut near: Vec<Vec2> = frame.enemies.iter().map(|e| *e - own).collect();
        near.sort_by(|a, b| a.length_squared().total_cmp(&b.length_squared()));
        if !near.is_empty() {
            out.push_str("closest enemies (dx, dy) dist:");
            for d in near.iter().take(LISTED_ENEMIES) {
                let _ = write!(out, " ({:.0},{:.0}) {:.0};", d.x, d.y, d.length());
            }
            out.push('\n');
        }
        if !frame.pickups.is_empty() {
            out.push_str("pickups:");
            for (at, kind) in &frame.pickups {
                let _ = write!(
                    out,
                    " {kind} at ({:.0},{:.0}) dist {:.0};",
                    at.x,
                    at.y,
                    at.distance(own)
                );
            }
            out.push('\n');
        }
    }
    out
}

/// The agent's own reading of a wave on the flow curve: -2 bored, 0 flow,
/// +2 overwhelmed, with a short note.
#[derive(Debug, Clone, PartialEq)]
pub struct Feel {
    /// How many waves had finished when the agent wrote it; it rates the last.
    pub after_waves: usize,
    pub value: i8,
    pub note: String,
}

/// One line per finished wave: the game's numbers next to the hand skill it
/// was played with (`skills`, in record order) and the agent's feel.
pub fn journal(record: &SessionRecord, skills: &[f32], feels: &[Feel]) -> String {
    let mut out = String::from(
        "| wave | try | skill | difficulty | secs | start hp | lowest hp | damage | died | risk | pickups (secs into wave, hp) | next | feel | note |\n\
         |---|---|---|---|---|---|---|---|---|---|---|---|---|---|\n",
    );
    for (i, wave) in record.waves.iter().enumerate() {
        let r = &wave.report;
        let feel = feels.iter().rev().find(|f| f.after_waves == i + 1);
        let next = wave.decision.map_or(String::new(), |d| d.to_string());
        let pickups: Vec<String> = wave
            .pickups
            .iter()
            .map(|p| format!("{} {:.1}s {}hp", p.kind, p.at_secs, p.hp))
            .collect();
        let _ = writeln!(
            out,
            "| {} | {} | {} | {:.2} | {:.1} | {} | {} | {} | {} | {:.2} | {} | {next} | {} | {} |",
            r.index,
            r.attempt,
            skills.get(i).map_or(String::new(), |s| format!("{s:.2}")),
            r.difficulty.level(),
            r.duration_secs,
            r.start_hp,
            r.lowest_hp,
            r.damage_taken,
            if r.player_died { "yes" } else { "" },
            wave_risk(r),
            pickups.join(", "),
            feel.map_or(String::new(), |f| format!("{:+}", f.value)),
            feel.map_or("", |f| f.note.as_str()),
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame() -> Frame {
        Frame {
            half_extents: Vec2::new(200.0, 80.0),
            player: Some((Vec2::ZERO, 80, 100)),
            enemies: vec![Vec2::new(150.0, 50.0), Vec2::new(155.0, 55.0)],
            pickups: vec![(Vec2::new(-150.0, -50.0), PickupKind::Mend)],
            ..Frame::default()
        }
    }

    #[test]
    fn map_marks_player_enemies_and_pickups_north_up() {
        let text = map(&frame());
        let rows: Vec<&str> = text.lines().collect();
        // 400 x 160 units at 40 per cell: 10 columns, 4 rows plus borders.
        assert_eq!(rows.len(), 6);
        assert_eq!(rows[1], "|........2.|");
        assert_eq!(rows[3], "|.....@....|");
        assert_eq!(rows[4], "|.M........|");
    }

    #[test]
    fn render_lists_closest_enemy_first_and_effects() {
        let mut f = frame();
        f.effects.weapons_locked = true;
        let text = render(&f);
        assert!(text.contains("hp 80/100"), "{text}");
        assert!(text.contains("WEAPONS LOCKED"), "{text}");
        assert!(
            text.contains("closest enemies (dx, dy) dist: (150,50) 158;"),
            "{text}"
        );
        assert!(text.contains("mend at (-150,-50)"), "{text}");
    }
}

//! The playtest report's Markdown tables. One row per named session, so bot
//! runs and a human's session files sit side by side.

use super::summary::{Summary, mean, ratio};
use crate::pickups::api::PickupKind;
use crate::telemetry::api::{SessionRecord, WEAPONS};
use std::fmt::Write as _;

/// A session with the name it gets in the report.
pub type Named<'a> = (String, &'a SessionRecord);

/// Overall: where the director settled each session.
pub fn table(rows: &[Named]) -> String {
    let mut out = String::from(
        "| player | waves | clears | deaths | furthest | settled diff | range | in band | hit rate | s/enemy | hp lost/clear | pickups o/h/m |\n\
         |---|---|---|---|---|---|---|---|---|---|---|---|\n",
    );
    for (name, session) in rows {
        let s = Summary::of(session);
        let taken = |kind: PickupKind| session.pickups().filter(|p| p.kind == kind).count();
        // Writing to a String cannot fail.
        let _ = writeln!(
            out,
            "| {name} | {} | {} | {} | {} | {:.1} | {}-{} | {:.0}% | {:.0}% | {:.2} | {:.0}% | {}/{}/{} |",
            s.waves,
            s.clears,
            s.deaths,
            s.furthest_wave,
            s.settled,
            s.settled_range.0,
            s.settled_range.1,
            s.in_band * 100.0,
            s.hit_rate * 100.0,
            s.secs_per_enemy,
            s.hp_lost_on_clear * 100.0,
            taken(PickupKind::Overdrive),
            taken(PickupKind::Heavy),
            taken(PickupKind::Mend),
        );
    }
    out
}

/// One row per session and weapon actually held.
pub fn weapon_table(rows: &[Named]) -> String {
    let mut out = String::from(
        "| player | weapon | held | kills | kills/min | kills/hit | hit rate | dmg taken/min | deaths | deaths/10 min |\n\
         |---|---|---|---|---|---|---|---|---|---|\n",
    );
    for (name, session) in rows {
        let totals = WEAPONS.map(|w| session.weapon_total(w));
        let all_secs: f32 = totals.iter().map(|t| t.secs_held).sum();
        for (weapon, t) in WEAPONS.iter().zip(&totals) {
            if t.secs_held < 1.0 {
                continue;
            }
            let _ = writeln!(
                out,
                "| {name} | {weapon} | {:.0}% | {} | {:.1} | {:.2} | {:.0}% | {:.1} | {} | {:.1} |",
                if all_secs > 0.0 {
                    t.secs_held / all_secs * 100.0
                } else {
                    0.0
                },
                t.kills,
                t.per_minute(t.kills),
                t.kills_per_hit(),
                t.hit_rate() * 100.0,
                t.per_minute(t.damage_taken),
                t.deaths,
                t.per_minute(t.deaths) * 10.0,
            );
        }
    }
    out
}

/// One row per session and pickup kind taken: in what situation it was taken.
pub fn pickup_table(rows: &[Named]) -> String {
    let mut out = String::from(
        "| player | pickup | taken | mean hp when taken | taken below 50% hp | mean secs into wave |\n\
         |---|---|---|---|---|---|\n",
    );
    for (name, session) in rows {
        for kind in PickupKind::ALL {
            let taken: Vec<_> = session.pickups().filter(|p| p.kind == kind).collect();
            if taken.is_empty() {
                continue;
            }
            let hp = |p: &&crate::telemetry::api::PickupTaken| p.hp as f32 / p.max_hp.max(1) as f32;
            let low = taken.iter().filter(|p| hp(p) < 0.5).count();
            let _ = writeln!(
                out,
                "| {name} | {kind} | {} | {:.0}% | {:.0}% | {:.1} |",
                taken.len(),
                mean(taken.iter().map(hp)) * 100.0,
                ratio(low, taken.len()) * 100.0,
                mean(taken.iter().map(|p| p.at_secs)),
            );
        }
    }
    out
}

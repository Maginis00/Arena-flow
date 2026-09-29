//! Rolling window of recent wave reports plus a debug text overlay, and a
//! record of the whole session (per wave, per weapon, every pickup) that a
//! windowed game also writes to `playtests/` for the playtest report.
//! Runs in `Update`: it only observes facts, it never drives simulation.
//! The overlay (which shows the difficulty number) exists only in debug builds.

pub mod api;
mod record;
mod record_file;
mod session_line;

use crate::combat::api::{EnemyKilled, PlayerDamaged, PlayerHealed};
use crate::flow_director::api::{DecisionReason, Difficulty, DifficultyAdjusted};
use crate::pickups::api::{Effects, EffectsChanged, PickupCollected, PickupKind};
use crate::player::api::PlayerSpawned;
use crate::waves::api::{WaveReport, WaveSpec, WaveStarted};
use crate::weapons::api::{WeaponKind, WeaponSwitched};
use bevy::prelude::*;
use std::collections::VecDeque;
use std::fmt::Write as _;

/// How many past wave reports the overlay keeps.
const REPORT_WINDOW: usize = 5;

pub struct TelemetryPlugin;

impl Plugin for TelemetryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Telemetry>()
            .init_resource::<api::SessionRecord>()
            .init_resource::<api::SessionFileEnabled>()
            .init_resource::<record::Recorder>()
            .init_resource::<record_file::SessionFile>()
            .add_systems(
                Update,
                (
                    collect,
                    (record::record_session, record_file::write_session_file).chain(),
                ),
            );
        if cfg!(debug_assertions) {
            app.add_systems(Startup, spawn_overlay)
                .add_systems(Update, render_overlay.after(collect));
        }
    }
}

#[derive(Resource, Debug, Default)]
struct Telemetry {
    wave: Option<WaveSpec>,
    wave_started_at: f32,
    /// `Some` once the current wave ended; freezes the timer.
    wave_duration: Option<f32>,
    next_difficulty: Option<Difficulty>,
    last_reason: Option<DecisionReason>,
    hp: u32,
    max_hp: u32,
    kills: u32,
    weapon: WeaponKind,
    effects: Effects,
    last_pickup: Option<PickupKind>,
    recent: VecDeque<WaveReport>,
}

#[derive(Component)]
struct OverlayText;

fn spawn_overlay(mut commands: Commands) {
    commands.spawn((
        OverlayText,
        Text::new(""),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            top: Val::Px(8.0),
            ..default()
        },
    ));
}

#[allow(clippy::too_many_arguments)] // one reader per observed fact
fn collect(
    time: Res<Time>,
    mut telemetry: ResMut<Telemetry>,
    mut started: MessageReader<WaveStarted>,
    mut reports: MessageReader<WaveReport>,
    mut adjusted: MessageReader<DifficultyAdjusted>,
    mut spawned: MessageReader<PlayerSpawned>,
    mut damaged: MessageReader<PlayerDamaged>,
    mut healed: MessageReader<PlayerHealed>,
    mut killed: MessageReader<EnemyKilled>,
    mut switched: MessageReader<WeaponSwitched>,
    mut effects: MessageReader<EffectsChanged>,
    mut pickups: MessageReader<PickupCollected>,
) {
    let t = &mut *telemetry;
    for s in spawned.read() {
        t.hp = s.max_hp;
        t.max_hp = s.max_hp;
    }
    for d in damaged.read() {
        t.hp = d.remaining;
        t.max_hp = d.max;
    }
    for h in healed.read() {
        t.hp = h.remaining;
        t.max_hp = h.max;
    }
    for s in started.read() {
        t.wave = Some(s.spec);
        t.wave_started_at = time.elapsed_secs();
        t.wave_duration = None;
        t.kills = 0;
    }
    t.kills = t
        .kills
        .saturating_add(u32::try_from(killed.read().count()).unwrap_or(u32::MAX));
    for r in reports.read() {
        t.wave_duration = Some(r.duration_secs);
        if t.recent.len() == REPORT_WINDOW {
            t.recent.pop_front();
        }
        t.recent.push_back(*r);
    }
    for a in adjusted.read() {
        t.next_difficulty = Some(a.difficulty);
        t.last_reason = Some(a.reason);
    }
    if let Some(w) = switched.read().last() {
        t.weapon = w.weapon;
    }
    if let Some(e) = effects.read().last() {
        t.effects = e.effects;
    }
    if let Some(p) = pickups.read().last() {
        t.last_pickup = Some(p.kind);
    }
}

fn render_overlay(
    time: Res<Time>,
    telemetry: Res<Telemetry>,
    mut text: Query<&mut Text, With<OverlayText>>,
) {
    let t = &*telemetry;
    let elapsed = t
        .wave_duration
        .unwrap_or_else(|| (time.elapsed_secs() - t.wave_started_at).max(0.0));
    let e = t.effects;
    for mut text in &mut text {
        let s = &mut text.0;
        s.clear();
        // Writing to a String cannot fail; ignore the fmt::Result.
        let _ = writeln!(
            s,
            "wave {} (try {})  difficulty {}  next {}",
            opt(t.wave.map(|w| w.index)),
            opt(t.wave.map(|w| w.attempt)),
            opt(t.wave.map(|w| w.difficulty)),
            opt(t.next_difficulty),
        );
        let _ = writeln!(
            s,
            "hp {}/{}  kills {}  time {elapsed:.1}s",
            t.hp, t.max_hp, t.kills
        );
        let _ = writeln!(
            s,
            "weapon {} [1 2 3]  last pickup {}",
            t.weapon,
            opt(t.last_pickup)
        );
        if e != Effects::NEUTRAL {
            let _ = writeln!(
                s,
                "effects: move x{:.2} fire x{:.2} dmg x{:.2} taken x{:.2}{}",
                e.move_speed,
                e.fire_rate,
                e.outgoing_damage,
                e.incoming_damage,
                if e.weapons_locked { " LOCKED" } else { "" },
            );
        }
        let _ = writeln!(s, "last decision: {}", opt(t.last_reason));
        if !t.recent.is_empty() {
            let _ = writeln!(s, "recent (wave.try diff secs kills dmg died acc pickups):");
        }
        for r in t.recent.iter().rev() {
            let accuracy = if r.shots_fired == 0 {
                0.0
            } else {
                r.shots_hit as f32 / r.shots_fired as f32 * 100.0
            };
            let _ = writeln!(
                s,
                "  #{}.{} d{} {:.1}s {}/{} {} {} {accuracy:.0}% {}",
                r.index,
                r.attempt,
                r.difficulty,
                r.duration_secs,
                r.enemies_killed,
                r.enemies_spawned,
                r.damage_taken,
                if r.player_died { "yes" } else { "no" },
                r.pickups_collected,
            );
        }
    }
}

fn opt<T: std::fmt::Display>(value: Option<T>) -> String {
    value.map_or_else(|| "-".to_owned(), |v| v.to_string())
}

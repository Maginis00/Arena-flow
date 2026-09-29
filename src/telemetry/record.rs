//! Builds the [`SessionRecord`]: per wave, the report, what each weapon did,
//! every pickup with the situation it was taken in, and the director's call.

use super::api::{PickupTaken, SessionRecord, WaveRecord, WeaponTally, weapon_slot};
use crate::combat::api::{
    EnemyKilled, Hit, HitSource, PlayerDamaged, PlayerDied, PlayerHealed, ShotId,
};
use crate::flow_director::api::{DecisionReason, DifficultyAdjusted};
use crate::pickups::api::PickupCollected;
use crate::player::api::PlayerSpawned;
use crate::waves::api::{WaveReport, WaveStarted};
use crate::weapons::api::{ShotFired, WeaponKind, WeaponSwitched};
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;

/// The wave in progress, before its report arrives.
#[derive(Resource, Debug, Default)]
pub(super) struct Recorder {
    in_wave: bool,
    wave_secs: f32,
    held: WeaponKind,
    hp: u32,
    max_hp: u32,
    weapons: [WeaponTally; 3],
    pickups: Vec<PickupTaken>,
    shot_weapon: HashMap<ShotId, WeaponKind>,
    shots_landed: HashSet<ShotId>,
    last_hit_by: HashMap<Entity, WeaponKind>,
}

impl Recorder {
    fn tally(&mut self, weapon: WeaponKind) -> &mut WeaponTally {
        &mut self.weapons[weapon_slot(weapon)]
    }

    /// Pickups are kept: one taken during the intermission counts toward the
    /// next wave, at 0 seconds.
    fn reset_wave(&mut self) {
        self.wave_secs = 0.0;
        self.weapons = Default::default();
        self.shot_weapon.clear();
        self.shots_landed.clear();
        self.last_hit_by.clear();
    }
}

/// Runs in `Update`: it only observes facts.
#[allow(clippy::too_many_arguments)] // one reader per observed fact
pub(super) fn record_session(
    time: Res<Time>,
    mut rec: ResMut<Recorder>,
    mut session: ResMut<SessionRecord>,
    mut started: MessageReader<WaveStarted>,
    mut reports: MessageReader<WaveReport>,
    mut adjusted: MessageReader<DifficultyAdjusted>,
    mut switched: MessageReader<WeaponSwitched>,
    mut spawned: MessageReader<PlayerSpawned>,
    mut healed: MessageReader<PlayerHealed>,
    mut shots: MessageReader<ShotFired>,
    mut hits: MessageReader<Hit>,
    mut killed: MessageReader<EnemyKilled>,
    mut damaged: MessageReader<PlayerDamaged>,
    mut died: MessageReader<PlayerDied>,
    mut collected: MessageReader<PickupCollected>,
) {
    let r = &mut *rec;
    if let Some(latest) = switched.read().last() {
        r.held = latest.weapon;
    }
    if started.read().count() > 0 {
        r.in_wave = true;
        r.reset_wave();
    }
    if r.in_wave {
        let dt = time.delta_secs();
        r.wave_secs += dt;
        let held = r.held;
        r.tally(held).secs_held += dt;
    }
    for s in spawned.read() {
        (r.hp, r.max_hp) = (s.max_hp, s.max_hp);
    }
    // Pickups before this frame's heals, so hp is what it was when taken.
    for p in collected.read() {
        r.pickups.push(PickupTaken {
            kind: p.kind,
            at_secs: r.wave_secs,
            hp: r.hp,
            max_hp: r.max_hp,
            weapon: r.held,
        });
    }
    for h in healed.read() {
        (r.hp, r.max_hp) = (h.remaining, h.max);
    }
    for shot in shots.read() {
        r.shot_weapon.insert(shot.shot, shot.weapon);
        r.tally(shot.weapon).shots += 1;
    }
    for hit in hits.read() {
        let HitSource::Shot { shot, .. } = hit.source else {
            continue;
        };
        let Some(&weapon) = r.shot_weapon.get(&shot) else {
            continue;
        };
        r.last_hit_by.insert(hit.target, weapon);
        if r.shots_landed.insert(shot) {
            r.tally(weapon).shots_hit += 1;
        }
    }
    for kill in killed.read() {
        if let Some(weapon) = r.last_hit_by.remove(&kill.enemy) {
            r.tally(weapon).kills += 1;
        }
    }
    let held = r.held;
    for d in damaged.read() {
        (r.hp, r.max_hp) = (d.remaining, d.max);
        r.tally(held).damage_taken += d.amount;
    }
    r.tally(held).deaths += u32::try_from(died.read().count()).unwrap_or(u32::MAX);

    for report in reports.read() {
        r.in_wave = false;
        session.waves.push(WaveRecord {
            report: *report,
            weapons: r.weapons,
            pickups: std::mem::take(&mut r.pickups),
            decision: None,
        });
        r.reset_wave();
    }
    // The director decides right after the report, usually in the same frame.
    for a in adjusted.read() {
        if a.reason == DecisionReason::Initial {
            continue;
        }
        if let Some(last) = session.waves.last_mut().filter(|w| w.decision.is_none()) {
            last.decision = Some(a.reason);
        }
    }
}

//! What telemetry records for a whole play session, bot or human. Other
//! plugins never read this; the playtest report and the session files do.

pub use super::record_file::{SESSION_DIR, read_session_file};

use crate::flow_director::api::DecisionReason;
use crate::pickups::api::PickupKind;
use crate::waves::api::WaveReport;
use crate::weapons::api::WeaponKind;
use bevy::prelude::*;

/// Every weapon, in key order (1, 2, 3).
pub const WEAPONS: [WeaponKind; 3] = [
    WeaponKind::Projectile,
    WeaponKind::Hitscan,
    WeaponKind::Melee,
];

/// What happened while one weapon was in hand during one wave.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct WeaponTally {
    /// Seconds held while the wave was in play.
    pub secs_held: f32,
    pub shots: u32,
    /// Shots that hit at least one enemy.
    pub shots_hit: u32,
    /// Enemies whose killing blow came from this weapon.
    pub kills: u32,
    /// Damage the player took while holding it.
    pub damage_taken: u32,
    /// Deaths while holding it.
    pub deaths: u32,
}

impl WeaponTally {
    pub fn add(&mut self, other: &Self) {
        self.secs_held += other.secs_held;
        self.shots += other.shots;
        self.shots_hit += other.shots_hit;
        self.kills += other.kills;
        self.damage_taken += other.damage_taken;
        self.deaths += other.deaths;
    }

    pub fn per_minute(&self, count: u32) -> f32 {
        if self.secs_held <= 0.0 {
            0.0
        } else {
            count as f32 * 60.0 / self.secs_held
        }
    }

    /// Enemies killed per shot that landed: above 1 means one shot often
    /// kills several (area damage), below 1 means targets need several hits.
    pub fn kills_per_hit(&self) -> f32 {
        if self.shots_hit == 0 {
            0.0
        } else {
            self.kills as f32 / self.shots_hit as f32
        }
    }

    pub fn hit_rate(&self) -> f32 {
        if self.shots == 0 {
            0.0
        } else {
            self.shots_hit as f32 / self.shots as f32
        }
    }
}

/// One pickup taken, with the situation it was taken in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PickupTaken {
    pub kind: PickupKind,
    /// Seconds since the wave started.
    pub at_secs: f32,
    /// Hp just before the pickup took effect.
    pub hp: u32,
    pub max_hp: u32,
    pub weapon: WeaponKind,
}

/// Everything recorded about one finished wave.
#[derive(Debug, Clone, PartialEq)]
pub struct WaveRecord {
    pub report: WaveReport,
    /// Indexed like [`WEAPONS`].
    pub weapons: [WeaponTally; 3],
    /// Taken during this wave or the intermission before it.
    pub pickups: Vec<PickupTaken>,
    /// What the director decided after this wave, once known.
    pub decision: Option<DecisionReason>,
}

impl WaveRecord {
    pub fn weapon(&self, weapon: WeaponKind) -> &WeaponTally {
        &self.weapons[weapon_slot(weapon)]
    }
}

/// The whole session so far, one record per finished wave.
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct SessionRecord {
    pub waves: Vec<WaveRecord>,
}

impl SessionRecord {
    /// One weapon's tally summed over every wave.
    pub fn weapon_total(&self, weapon: WeaponKind) -> WeaponTally {
        let mut total = WeaponTally::default();
        for wave in &self.waves {
            total.add(wave.weapon(weapon));
        }
        total
    }

    pub fn pickups(&self) -> impl Iterator<Item = &PickupTaken> {
        self.waves.iter().flat_map(|w| w.pickups.iter())
    }
}

pub const fn weapon_slot(weapon: WeaponKind) -> usize {
    match weapon {
        WeaponKind::Projectile => 0,
        WeaponKind::Hitscan => 1,
        WeaponKind::Melee => 2,
    }
}

//! Per-weapon numbers for one session: how long each weapon was held during
//! waves, what it killed, and what the player suffered while holding it.

use crate::combat::api::{EnemyKilled, Hit, HitSource, PlayerDamaged, PlayerDied, ShotId};
use crate::waves::api::{WaveReport, WaveStarted};
use crate::weapons::api::{ShotFired, WeaponKind, WeaponSwitched};
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;

pub const WEAPONS: [WeaponKind; 3] = [
    WeaponKind::Projectile,
    WeaponKind::Hitscan,
    WeaponKind::Melee,
];

/// What happened while one weapon was in hand.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct WeaponTally {
    /// Seconds held while a wave was in play.
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
    pub fn per_minute(&self, count: u32) -> f32 {
        if self.secs_held <= 0.0 {
            0.0
        } else {
            count as f32 * 60.0 / self.secs_held
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

/// One tally per weapon, indexed like [`WEAPONS`], plus the bookkeeping to
/// attribute kills to the weapon that landed the last hit.
#[derive(Resource, Debug, Clone, Default)]
pub struct WeaponUse {
    pub tallies: [WeaponTally; 3],
    held: WeaponKind,
    in_wave: bool,
    shot_weapon: HashMap<ShotId, WeaponKind>,
    shots_landed: HashSet<ShotId>,
    last_hit_by: HashMap<Entity, WeaponKind>,
}

impl WeaponUse {
    pub fn of(&self, weapon: WeaponKind) -> WeaponTally {
        self.tallies[slot(weapon)]
    }

    fn tally(&mut self, weapon: WeaponKind) -> &mut WeaponTally {
        &mut self.tallies[slot(weapon)]
    }
}

const fn slot(weapon: WeaponKind) -> usize {
    match weapon {
        WeaponKind::Projectile => 0,
        WeaponKind::Hitscan => 1,
        WeaponKind::Melee => 2,
    }
}

/// Runs in `Update`, like the game's own telemetry: it only observes.
#[allow(clippy::too_many_arguments)] // one reader per observed fact
pub fn record_weapon_use(
    time: Res<Time>,
    mut usage: ResMut<WeaponUse>,
    mut switched: MessageReader<WeaponSwitched>,
    mut started: MessageReader<WaveStarted>,
    mut reports: MessageReader<WaveReport>,
    mut shots: MessageReader<ShotFired>,
    mut hits: MessageReader<Hit>,
    mut killed: MessageReader<EnemyKilled>,
    mut damaged: MessageReader<PlayerDamaged>,
    mut died: MessageReader<PlayerDied>,
) {
    let u = &mut *usage;
    if let Some(latest) = switched.read().last() {
        u.held = latest.weapon;
    }
    if started.read().count() > 0 {
        u.in_wave = true;
    }
    if u.in_wave {
        let held = u.held;
        u.tally(held).secs_held += time.delta_secs();
    }
    if reports.read().count() > 0 {
        u.in_wave = false;
        // Shots and enemies never outlive a wave.
        u.shot_weapon.clear();
        u.shots_landed.clear();
        u.last_hit_by.clear();
    }
    for shot in shots.read() {
        u.shot_weapon.insert(shot.shot, shot.weapon);
        u.tally(shot.weapon).shots += 1;
    }
    for hit in hits.read() {
        let HitSource::Shot { shot, .. } = hit.source else {
            continue;
        };
        let Some(&weapon) = u.shot_weapon.get(&shot) else {
            continue;
        };
        u.last_hit_by.insert(hit.target, weapon);
        if u.shots_landed.insert(shot) {
            u.tally(weapon).shots_hit += 1;
        }
    }
    for kill in killed.read() {
        if let Some(weapon) = u.last_hit_by.remove(&kill.enemy) {
            u.tally(weapon).kills += 1;
        }
    }
    let held = u.held;
    let damage: u32 = damaged.read().map(|d| d.amount).sum();
    u.tally(held).damage_taken += damage;
    u.tally(held).deaths += u32::try_from(died.read().count()).unwrap_or(u32::MAX);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rates_are_zero_without_use() {
        let t = WeaponTally::default();
        assert_eq!((t.per_minute(5), t.hit_rate()), (0.0, 0.0));
    }

    #[test]
    fn per_minute_scales_by_time_held() {
        let t = WeaponTally {
            secs_held: 30.0,
            shots: 4,
            shots_hit: 3,
            ..WeaponTally::default()
        };
        assert_eq!(t.per_minute(10), 20.0);
        assert_eq!(t.hit_rate(), 0.75);
    }
}

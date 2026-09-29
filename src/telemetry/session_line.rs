//! The session file format: one flat JSON object per wave, enums written as
//! lowercase words so the files stay readable and stable.

use super::api::{PickupTaken, WEAPONS, WaveRecord, WeaponTally, weapon_slot};
use crate::flow_director::api::{DecisionReason, Difficulty, Signal};
use crate::pickups::api::PickupKind;
use crate::waves::api::{WaveIndex, WaveReport};
use crate::weapons::api::WeaponKind;
use serde::{Deserialize, Serialize};

/// One line of a session file. Flat, stable names; enums as lowercase words.
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct WaveLine {
    wave: u32,
    attempt: u32,
    difficulty: u8,
    duration_secs: f32,
    enemies_spawned: u32,
    enemies_killed: u32,
    damage_taken: u32,
    player_max_hp: u32,
    player_died: bool,
    shots_fired: u32,
    shots_hit: u32,
    /// As the wave counted them: taken while the wave was in play.
    pickups_collected: u32,
    decision: Option<String>,
    weapons: Vec<WeaponLine>,
    pickups: Vec<PickupLine>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WeaponLine {
    weapon: String,
    secs_held: f32,
    shots: u32,
    shots_hit: u32,
    kills: u32,
    damage_taken: u32,
    deaths: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct PickupLine {
    kind: String,
    at_secs: f32,
    hp: u32,
    max_hp: u32,
    weapon: String,
}

impl From<&WaveRecord> for WaveLine {
    fn from(w: &WaveRecord) -> Self {
        let r = &w.report;
        Self {
            wave: r.index.0,
            attempt: r.attempt,
            difficulty: r.difficulty.get(),
            duration_secs: r.duration_secs,
            enemies_spawned: r.enemies_spawned,
            enemies_killed: r.enemies_killed,
            damage_taken: r.damage_taken,
            player_max_hp: r.player_max_hp,
            player_died: r.player_died,
            shots_fired: r.shots_fired,
            shots_hit: r.shots_hit,
            pickups_collected: r.pickups_collected,
            decision: w.decision.map(decision_name),
            weapons: WEAPONS
                .iter()
                .map(|&weapon| {
                    let t = w.weapon(weapon);
                    WeaponLine {
                        weapon: weapon_name(weapon).to_owned(),
                        secs_held: t.secs_held,
                        shots: t.shots,
                        shots_hit: t.shots_hit,
                        kills: t.kills,
                        damage_taken: t.damage_taken,
                        deaths: t.deaths,
                    }
                })
                .collect(),
            pickups: w
                .pickups
                .iter()
                .map(|p| PickupLine {
                    kind: p.kind.to_string(),
                    at_secs: p.at_secs,
                    hp: p.hp,
                    max_hp: p.max_hp,
                    weapon: weapon_name(p.weapon).to_owned(),
                })
                .collect(),
        }
    }
}

impl WaveLine {
    pub(super) fn into_record(self) -> Result<WaveRecord, String> {
        let mut weapons = [WeaponTally::default(); 3];
        for l in &self.weapons {
            weapons[weapon_slot(parse_weapon(&l.weapon)?)] = WeaponTally {
                secs_held: l.secs_held,
                shots: l.shots,
                shots_hit: l.shots_hit,
                kills: l.kills,
                damage_taken: l.damage_taken,
                deaths: l.deaths,
            };
        }
        let pickups = self
            .pickups
            .iter()
            .map(|p| {
                Ok(PickupTaken {
                    kind: parse_pickup(&p.kind)?,
                    at_secs: p.at_secs,
                    hp: p.hp,
                    max_hp: p.max_hp,
                    weapon: parse_weapon(&p.weapon)?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let report = WaveReport {
            index: WaveIndex(self.wave),
            attempt: self.attempt,
            difficulty: Difficulty::new(self.difficulty).map_err(|e| e.to_string())?,
            duration_secs: self.duration_secs,
            enemies_spawned: self.enemies_spawned,
            enemies_killed: self.enemies_killed,
            damage_taken: self.damage_taken,
            player_max_hp: self.player_max_hp,
            player_died: self.player_died,
            shots_fired: self.shots_fired,
            shots_hit: self.shots_hit,
            pickups_collected: self.pickups_collected,
        };
        Ok(WaveRecord {
            report,
            weapons,
            pickups,
            decision: self.decision.as_deref().map(parse_decision).transpose()?,
        })
    }
}

const fn weapon_name(weapon: WeaponKind) -> &'static str {
    match weapon {
        WeaponKind::Projectile => "projectile",
        WeaponKind::Hitscan => "hitscan",
        WeaponKind::Melee => "melee",
    }
}

fn parse_weapon(name: &str) -> Result<WeaponKind, String> {
    WEAPONS
        .into_iter()
        .find(|w| weapon_name(*w) == name)
        .ok_or_else(|| format!("unknown weapon {name:?}"))
}

fn parse_pickup(name: &str) -> Result<PickupKind, String> {
    PickupKind::ALL
        .into_iter()
        .find(|k| k.to_string() == name)
        .ok_or_else(|| format!("unknown pickup {name:?}"))
}

const DECISIONS: [DecisionReason; 11] = [
    DecisionReason::Initial,
    DecisionReason::Raised,
    DecisionReason::Lowered,
    DecisionReason::HeldLowHp,
    DecisionReason::HeldSlowClear,
    DecisionReason::HeldInBand,
    DecisionReason::HeldHysteresis(Signal::TooEasy),
    DecisionReason::HeldHysteresis(Signal::InBand),
    DecisionReason::HeldHysteresis(Signal::TooHard),
    DecisionReason::HeldAtMax,
    DecisionReason::HeldAtMin,
];

fn decision_name(reason: DecisionReason) -> String {
    match reason {
        DecisionReason::Initial => "initial".into(),
        DecisionReason::Raised => "raised".into(),
        DecisionReason::Lowered => "lowered".into(),
        DecisionReason::HeldLowHp => "held_low_hp".into(),
        DecisionReason::HeldSlowClear => "held_slow_clear".into(),
        DecisionReason::HeldInBand => "held_in_band".into(),
        DecisionReason::HeldHysteresis(signal) => format!("held_hysteresis_{signal:?}"),
        DecisionReason::HeldAtMax => "held_at_max".into(),
        DecisionReason::HeldAtMin => "held_at_min".into(),
    }
}

fn parse_decision(name: &str) -> Result<DecisionReason, String> {
    DECISIONS
        .into_iter()
        .find(|d| decision_name(*d) == name)
        .ok_or_else(|| format!("unknown decision {name:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wave_survives_a_round_trip() {
        let mut weapons = [WeaponTally::default(); 3];
        weapons[2] = WeaponTally {
            secs_held: 12.5,
            shots: 20,
            shots_hit: 15,
            kills: 9,
            damage_taken: 30,
            deaths: 1,
        };
        let wave = WaveRecord {
            report: WaveReport {
                index: WaveIndex(4),
                attempt: 2,
                difficulty: Difficulty::new(5).expect("in range"),
                duration_secs: 12.5,
                enemies_spawned: 14,
                enemies_killed: 9,
                damage_taken: 100,
                player_max_hp: 100,
                player_died: true,
                shots_fired: 20,
                shots_hit: 15,
                pickups_collected: 1,
            },
            weapons,
            pickups: vec![PickupTaken {
                kind: PickupKind::Mend,
                at_secs: 3.0,
                hp: 40,
                max_hp: 100,
                weapon: WeaponKind::Melee,
            }],
            decision: Some(DecisionReason::HeldHysteresis(Signal::TooHard)),
        };
        let json = serde_json::to_string(&WaveLine::from(&wave)).expect("serialises");
        let back: WaveLine = serde_json::from_str(&json).expect("parses");
        assert_eq!(back.into_record(), Ok(wave));
    }
}

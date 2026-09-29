//! Pure decisions a simulated player makes: which pickup is worth it, which
//! weapon to hold, and whether a target is in reach. Unit-tested.

use super::tier::{PickupPolicy, WeaponPolicy};
use crate::pickups::api::PickupKind;
use crate::weapons::api::WeaponKind;
use bevy::math::Vec2;

/// What the bot believes about weapon reach. Mirrors the weapons' own
/// PLACEHOLDER numbers (plus an enemy half-size); update both together.
pub const fn reach(weapon: WeaponKind) -> f32 {
    match weapon {
        WeaponKind::Projectile => f32::INFINITY,
        WeaponKind::Hitscan => 520.0,
        WeaponKind::Melee => 85.0,
    }
}

/// PLACEHOLDER: a threat this close makes slow or locked pickups dangerous.
const DANGER_RADIUS: f32 = 150.0;
/// PLACEHOLDER: situational weapon switches to melee with this many enemies this close.
const CROWD_RADIUS: f32 = 100.0;
const CROWD_SIZE: usize = 3;

/// Is this pickup worth walking to right now?
pub fn wants_pickup(
    policy: PickupPolicy,
    kind: PickupKind,
    hp_fraction: f32,
    nearest_threat: f32,
) -> bool {
    match policy {
        PickupPolicy::Ignore => false,
        PickupPolicy::Greedy => true,
        PickupPolicy::Weighed => match kind {
            // Heal, but the weapon lock only pays off with space around you.
            PickupKind::Mend => hp_fraction <= 0.6 && nearest_threat > DANGER_RADIUS,
            // Faster fire, but more damage taken: only with hp to spare.
            PickupKind::Overdrive => hp_fraction >= 0.5,
            // Double damage, but slow: only when nothing is on top of you.
            PickupKind::Heavy => nearest_threat > DANGER_RADIUS,
        },
    }
}

pub fn choose_weapon(policy: WeaponPolicy, own: Vec2, threats: &[Vec2]) -> WeaponKind {
    match policy {
        WeaponPolicy::Fixed(weapon) => weapon,
        WeaponPolicy::Situational => {
            let close = threats
                .iter()
                .filter(|t| t.distance(own) < CROWD_RADIUS)
                .count();
            if close >= CROWD_SIZE {
                WeaponKind::Melee
            } else {
                WeaponKind::Hitscan
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weighed_policy_refuses_mend_when_crowded() {
        let p = PickupPolicy::Weighed;
        assert!(wants_pickup(p, PickupKind::Mend, 0.3, 400.0));
        assert!(!wants_pickup(p, PickupKind::Mend, 0.3, 50.0));
        assert!(!wants_pickup(p, PickupKind::Mend, 0.9, 400.0));
        assert!(!wants_pickup(p, PickupKind::Overdrive, 0.2, 400.0));
    }

    #[test]
    fn situational_weapon_goes_melee_in_a_crowd() {
        let own = Vec2::ZERO;
        let crowd = [Vec2::X * 40.0, Vec2::Y * 40.0, Vec2::NEG_X * 40.0];
        let policy = WeaponPolicy::Situational;
        assert_eq!(choose_weapon(policy, own, &crowd), WeaponKind::Melee);
        assert_eq!(choose_weapon(policy, own, &crowd[..1]), WeaponKind::Hitscan);
    }
}

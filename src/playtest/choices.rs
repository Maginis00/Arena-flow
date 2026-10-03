//! Pure decisions a simulated player makes: which pickup is worth it, which
//! weapon to hold, and whether a target is in reach. Unit-tested.

use super::tier::{PickupPolicy, SpendPolicy, WeaponPolicy};
use crate::pickups::PickupKind;
use crate::weapons::{SwordBinding, WeaponKind};
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
        PickupPolicy::Clear(radius) => nearest_threat > f32::from(radius),
    }
}

/// Is a shard worth walking to right now? Weighed pickers stay out of crowds.
pub fn wants_shard(policy: PickupPolicy, nearest_threat: f32) -> bool {
    match policy {
        PickupPolicy::Ignore => false,
        PickupPolicy::Greedy => true,
        PickupPolicy::Weighed => nearest_threat > DANGER_RADIUS,
        PickupPolicy::Clear(radius) => nearest_threat > f32::from(radius),
    }
}

/// What the bot believes about the shard blast. Mirrors the pickups' own
/// PLACEHOLDER numbers (fewest shards, smallest radius); update both together.
const SPEND_MIN: u32 = 3;
const BLAST_REACH: f32 = 150.0;

/// Spend the held shards on a blast now?
pub fn wants_to_spend(
    policy: SpendPolicy,
    held: u32,
    hp_fraction: f32,
    own: Vec2,
    enemies: &[Vec2],
    bolts: &[Vec2],
) -> bool {
    if held < SPEND_MIN {
        return false;
    }
    let in_reach = |at: &[Vec2]| at.iter().filter(|t| t.distance(own) <= BLAST_REACH).count();
    match policy {
        SpendPolicy::Hoard => false,
        SpendPolicy::When {
            crowd,
            low_hp_pct,
            bolts: counts_bolts,
        } => {
            let in_reach = in_reach(enemies) + if counts_bolts { in_reach(bolts) } else { 0 };
            in_reach > 0
                && (in_reach >= usize::from(crowd) || hp_fraction * 100.0 < f32::from(low_hp_pct))
        }
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

/// The weapon to select. With the sword on its own button there is nothing
/// to select for melee, so a melee pick holds hitscan and leaves the sword
/// to [`swing_target`].
pub fn primary(weapon: WeaponKind, sword: SwordBinding) -> WeaponKind {
    match (sword, weapon) {
        (SwordBinding::RightClick, WeaponKind::Melee) => WeaponKind::Hitscan,
        _ => weapon,
    }
}

/// With the sword on its own button: the nearest enemy close enough to swing
/// at. Every bot swings whenever one is, with no further judgement.
pub fn swing_target(sword: SwordBinding, own: Vec2, threats: &[Vec2]) -> Option<Vec2> {
    if sword != SwordBinding::RightClick {
        return None;
    }
    let melee = reach(WeaponKind::Melee);
    threats
        .iter()
        .copied()
        .filter(|t| t.distance(own) <= melee)
        .min_by(|a, b| a.distance_squared(own).total_cmp(&b.distance_squared(own)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sword_on_its_own_button_swings_only_at_what_is_close() {
        let own = Vec2::ZERO;
        let threats = [Vec2::new(300.0, 0.0), Vec2::new(60.0, 0.0)];
        assert_eq!(
            swing_target(SwordBinding::RightClick, own, &threats),
            Some(Vec2::new(60.0, 0.0))
        );
        assert_eq!(
            swing_target(SwordBinding::RightClick, own, &threats[..1]),
            None
        );
        assert_eq!(swing_target(SwordBinding::Key3, own, &threats), None);
        assert_eq!(
            primary(WeaponKind::Melee, SwordBinding::RightClick),
            WeaponKind::Hitscan
        );
        assert_eq!(
            primary(WeaponKind::Melee, SwordBinding::Key3),
            WeaponKind::Melee
        );
    }

    #[test]
    fn weighed_policy_refuses_mend_when_crowded() {
        let p = PickupPolicy::Weighed;
        assert!(wants_pickup(p, PickupKind::Mend, 0.3, 400.0));
        assert!(!wants_pickup(p, PickupKind::Mend, 0.3, 50.0));
        assert!(!wants_pickup(p, PickupKind::Mend, 0.9, 400.0));
        assert!(!wants_pickup(p, PickupKind::Overdrive, 0.2, 400.0));
    }

    #[test]
    fn spend_policies_differ_in_when_they_blast() {
        let own = Vec2::ZERO;
        let one = [Vec2::X * 50.0];
        let crowd = [Vec2::X * 50.0; 4];
        let spend = |policy, held, hp, enemies: &[Vec2], bolts: &[Vec2]| {
            wants_to_spend(policy, held, hp, own, enemies, bolts)
        };
        assert!(!spend(SpendPolicy::EAGER, 2, 1.0, &crowd, &[]));
        assert!(spend(SpendPolicy::EAGER, 3, 1.0, &one, &[]));
        assert!(!spend(SpendPolicy::PANIC, 3, 1.0, &one, &[]));
        assert!(spend(SpendPolicy::PANIC, 3, 0.2, &one, &[]));
        assert!(spend(SpendPolicy::PANIC, 3, 1.0, &crowd, &[]));
        assert!(!spend(SpendPolicy::Hoard, 10, 0.1, &crowd, &[]));
        // Bolts count only for a rule that watches them.
        let watching = SpendPolicy::When {
            crowd: 2,
            low_hp_pct: 0,
            bolts: true,
        };
        assert!(spend(watching, 3, 1.0, &one, &one));
        assert!(!spend(SpendPolicy::PANIC, 3, 1.0, &one, &crowd));
    }

    #[test]
    fn policies_read_back_what_they_print() {
        for text in [
            "ignore",
            "clear:150",
            "hoard",
            "panic",
            "eager",
            "when:3:40:bolts",
        ] {
            let pickup = text.parse::<PickupPolicy>().map(|p| p.to_string());
            let spend = text.parse::<SpendPolicy>().map(|s| s.to_string());
            assert!(
                pickup == Ok(text.to_owned()) || spend == Ok(text.to_owned()),
                "{text}"
            );
        }
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

//! What each enemy kind is (hp, size, how it scales off the wave's levers)
//! and which kind fills each spawn slot of a wave. Pure; unit-tested.

use super::api::{EnemyKind, EnemyMix};

/// Fixed numbers for one kind. Speed and contact damage are multipliers on
/// the wave's levers, so the director still scales every kind.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct KindStats {
    pub(super) max_hp: u32,
    pub(super) half_size: f32,
    pub(super) speed_scale: f32,
    pub(super) contact_scale: f32,
}

/// PLACEHOLDER: every number in this table.
pub(super) const fn stats(kind: EnemyKind) -> KindStats {
    match kind {
        EnemyKind::Grunt => KindStats {
            max_hp: 3,
            half_size: 12.0,
            speed_scale: 1.0,
            contact_scale: 1.0,
        },
        EnemyKind::Shooter => KindStats {
            max_hp: 4,
            half_size: 12.0,
            speed_scale: 0.8,
            contact_scale: 0.5,
        },
        EnemyKind::Brute => KindStats {
            max_hp: 12,
            half_size: 20.0,
            speed_scale: 0.6,
            contact_scale: 1.5,
        },
        EnemyKind::Charger => KindStats {
            max_hp: 4,
            half_size: 12.0,
            speed_scale: 0.9,
            contact_scale: 1.5,
        },
        EnemyKind::Summoner => KindStats {
            max_hp: 10,
            half_size: 16.0,
            speed_scale: 0.5,
            contact_scale: 1.0,
        },
    }
}

/// A lever value scaled for one kind, never below 1 when the lever is above 0.
pub(super) fn scaled(lever: u32, scale: f32) -> u32 {
    if lever == 0 {
        return 0;
    }
    // Levers are small positive numbers; the product stays far below u32::MAX.
    ((lever as f32 * scale).round() as u32).max(1)
}

/// One special kind in a mix: it takes slot `first` and every `every`th
/// slot after it. Slots count from 1 within each wave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Slot {
    kind: EnemyKind,
    first: u32,
    every: u32,
}

impl Slot {
    const fn takes(self, slot: u32) -> bool {
        slot >= self.first && (slot - self.first).is_multiple_of(self.every)
    }
}

/// PLACEHOLDER: how often each kind appears when mixed in alone.
const fn alone(kind: EnemyKind) -> Slot {
    let (first, every) = match kind {
        EnemyKind::Grunt => (1, 1),
        EnemyKind::Shooter => (2, 4),
        EnemyKind::Brute => (4, 6),
        EnemyKind::Charger => (2, 3),
        EnemyKind::Summoner => (2, 8),
    };
    Slot { kind, first, every }
}

/// PLACEHOLDER: the `all` mix. Earlier entries win a shared slot.
const ALL_MIX: [Slot; 4] = [
    Slot {
        kind: EnemyKind::Summoner,
        first: 3,
        every: 10,
    },
    Slot {
        kind: EnemyKind::Brute,
        first: 6,
        every: 7,
    },
    Slot {
        kind: EnemyKind::Shooter,
        first: 2,
        every: 4,
    },
    Slot {
        kind: EnemyKind::Charger,
        first: 4,
        every: 5,
    },
];

/// The kind for spawn `slot` (1-based) of a wave.
pub(super) fn kind_for_slot(mix: EnemyMix, slot: u32) -> EnemyKind {
    let special = match mix {
        EnemyMix::Grunts => None,
        EnemyMix::With(kind) => Some(alone(kind)).filter(|s| s.takes(slot)),
        EnemyMix::All => ALL_MIX.into_iter().find(|s| s.takes(slot)),
    };
    special.map_or(EnemyKind::Grunt, |s| s.kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wave(mix: EnemyMix, count: u32) -> Vec<EnemyKind> {
        (1..=count).map(|slot| kind_for_slot(mix, slot)).collect()
    }

    #[test]
    fn grunt_mix_is_only_grunts() {
        assert!(
            wave(EnemyMix::Grunts, 30)
                .iter()
                .all(|k| *k == EnemyKind::Grunt)
        );
        assert!(
            wave(EnemyMix::With(EnemyKind::Grunt), 30)
                .iter()
                .all(|k| *k == EnemyKind::Grunt)
        );
    }

    #[test]
    fn a_single_kind_takes_its_slots() {
        use EnemyKind::{Grunt as G, Shooter as S};
        assert_eq!(
            wave(EnemyMix::With(S), 10),
            [G, S, G, G, G, S, G, G, G, S].to_vec()
        );
    }

    #[test]
    fn every_kind_appears_in_a_starting_wave_of_the_all_mix() {
        // Difficulty 3 brings 8 enemies (levers_for); the summoner should be
        // there, and the rest of the kinds by 10.
        let small = wave(EnemyMix::All, 8);
        assert!(small.contains(&EnemyKind::Summoner));
        let big = wave(EnemyMix::All, 10);
        for kind in EnemyKind::ALL {
            assert!(big.contains(&kind), "{kind} missing from {big:?}");
        }
    }

    #[test]
    fn a_summoner_shows_up_even_in_the_smallest_wave() {
        assert!(wave(EnemyMix::With(EnemyKind::Summoner), 4).contains(&EnemyKind::Summoner));
    }

    #[test]
    fn scaled_levers_round_and_never_drop_to_zero() {
        assert_eq!(scaled(12, 0.5), 6);
        assert_eq!(scaled(1, 0.1), 1);
        assert_eq!(scaled(0, 2.0), 0);
        assert_eq!(scaled(9, 1.5), 14);
    }

    #[test]
    fn mix_names_parse() {
        assert_eq!("all".parse(), Ok(EnemyMix::All));
        assert_eq!("Grunt".parse(), Ok(EnemyMix::Grunts));
        assert_eq!("brute".parse(), Ok(EnemyMix::With(EnemyKind::Brute)));
        assert!("dragon".parse::<EnemyMix>().is_err());
    }
}

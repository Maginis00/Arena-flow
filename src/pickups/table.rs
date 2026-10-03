//! What each pickup does. Pure data and functions, unit-tested.

use super::{Effects, PickupKind};

/// PLACEHOLDER numbers for every pickup.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PickupDef {
    /// Modifiers while active.
    pub effects: Effects,
    /// How long the modifiers last.
    pub duration_secs: f32,
    /// Hp restored instantly on pickup.
    pub heal: u32,
}

pub const fn def(kind: PickupKind) -> PickupDef {
    let n = Effects::NEUTRAL;
    match kind {
        PickupKind::Overdrive => PickupDef {
            effects: Effects {
                fire_rate: 1.6,
                incoming_damage: 1.5,
                ..n
            },
            duration_secs: 8.0,
            heal: 0,
        },
        PickupKind::Heavy => PickupDef {
            effects: Effects {
                outgoing_damage: 2.0,
                move_speed: 0.7,
                ..n
            },
            duration_secs: 8.0,
            heal: 0,
        },
        PickupKind::Mend => PickupDef {
            effects: Effects {
                weapons_locked: true,
                ..n
            },
            duration_secs: 2.5,
            heal: 30,
        },
    }
}

/// Seconds left per kind, indexed like [`PickupKind::ALL`].
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Active {
    remaining_secs: [f32; 3],
}

impl Active {
    fn slot(kind: PickupKind) -> usize {
        match kind {
            PickupKind::Overdrive => 0,
            PickupKind::Heavy => 1,
            PickupKind::Mend => 2,
        }
    }

    /// Taking a kind that is already active refreshes it; it does not stack.
    pub fn activate(&mut self, kind: PickupKind) {
        self.remaining_secs[Self::slot(kind)] = def(kind).duration_secs;
    }

    pub fn tick(&mut self, dt: f32) {
        for t in &mut self.remaining_secs {
            *t = (*t - dt).max(0.0);
        }
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn remaining(&self, kind: PickupKind) -> f32 {
        self.remaining_secs[Self::slot(kind)]
    }

    pub fn effects(&self) -> Effects {
        PickupKind::ALL
            .into_iter()
            .filter(|kind| self.remaining(*kind) > 0.0)
            .fold(Effects::NEUTRAL, |acc, kind| acc.combine(def(kind).effects))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pickups::scale_damage;

    #[test]
    fn every_pickup_has_a_downside() {
        for kind in PickupKind::ALL {
            let d = def(kind);
            let e = d.effects;
            let downside = e.move_speed < 1.0
                || e.fire_rate < 1.0
                || e.outgoing_damage < 1.0
                || e.incoming_damage > 1.0
                || e.weapons_locked;
            let upside = e.move_speed > 1.0
                || e.fire_rate > 1.0
                || e.outgoing_damage > 1.0
                || e.incoming_damage < 1.0
                || d.heal > 0;
            assert!(downside && upside, "{kind} must trade something");
        }
    }

    #[test]
    fn effects_stack_across_kinds_and_expire() {
        let mut active = Active::default();
        active.activate(PickupKind::Overdrive);
        active.activate(PickupKind::Heavy);
        let e = active.effects();
        assert_eq!(e.fire_rate, 1.6);
        assert_eq!(e.outgoing_damage, 2.0);
        assert_eq!(e.incoming_damage, 1.5);
        assert_eq!(e.move_speed, 0.7);

        active.tick(100.0);
        assert_eq!(active.effects(), Effects::NEUTRAL);
    }

    #[test]
    fn same_kind_refreshes_instead_of_stacking() {
        let mut active = Active::default();
        active.activate(PickupKind::Overdrive);
        active.tick(5.0);
        active.activate(PickupKind::Overdrive);
        assert_eq!(active.remaining(PickupKind::Overdrive), 8.0);
        assert_eq!(active.effects().fire_rate, 1.6);
    }

    #[test]
    fn damage_scaling_rounds_and_never_zeroes_a_hit() {
        assert_eq!(scale_damage(12, 1.5), 18);
        assert_eq!(scale_damage(1, 2.0), 2);
        assert_eq!(scale_damage(1, 0.1), 1);
        assert_eq!(scale_damage(0, 2.0), 0);
    }
}

//! How each enemy kind wants to move, as pure decisions: a direction (with a
//! speed share) given where the player is. Unit-tested; the systems in the
//! domain root apply them.

use bevy::math::Vec2;

/// PLACEHOLDER: ranged kinds settle within this many units of their range.
const RANGE_SLACK: f32 = 40.0;
/// PLACEHOLDER: share of their speed ranged kinds use to circle once in range.
const ORBIT_SHARE: f32 = 0.5;
/// PLACEHOLDER: share of their speed ranged kinds keep creeping in once in
/// range, so waiting them out works too and no wave can stall.
const CREEP_SHARE: f32 = 0.25;

/// PLACEHOLDER: a charger starts its wind-up this close to the player.
const CHARGE_TRIGGER_RANGE: f32 = 240.0;
/// PLACEHOLDER: seconds a charger stands still before dashing (the tell).
const CHARGE_WINDUP_SECS: f32 = 0.6;
/// PLACEHOLDER: seconds a dash lasts.
const CHARGE_DASH_SECS: f32 = 0.45;
/// PLACEHOLDER: dash speed as a multiple of the charger's own speed.
const CHARGE_DASH_SCALE: f32 = 4.0;
/// PLACEHOLDER: seconds a charger stands still after a dash (the opening).
const CHARGE_RECOVER_SECS: f32 = 0.9;

/// Straight at the player.
pub(super) fn chase(to_player: Vec2) -> Vec2 {
    to_player.normalize_or_zero()
}

/// Close to `range`, then circle the player while slowly creeping in. Never
/// backs off: a player who walks up to a ranged enemy can reach it, through
/// its fire, and one who waits gets it in the end.
/// `orbit` is +1 or -1 so neighbours do not all circle the same way.
pub(super) fn hold_range(to_player: Vec2, range: f32, orbit: f32) -> Vec2 {
    let toward = to_player.normalize_or_zero();
    if to_player.length() > range + RANGE_SLACK {
        toward
    } else {
        toward.perp() * orbit * ORBIT_SHARE + toward * CREEP_SHARE
    }
}

/// Where a charger is in its approach, tell, dash and recovery.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(super) enum Charge {
    #[default]
    Approach,
    /// Standing still, aiming at where the player was when it stopped.
    WindUp { left_secs: f32, direction: Vec2 },
    /// Locked direction: a sidestep makes it miss.
    Dash { left_secs: f32, direction: Vec2 },
    /// Standing still after the dash.
    Recover { left_secs: f32 },
}

impl Charge {
    /// Advance by `dt` and return the next phase and this tick's movement
    /// (a direction times a share of the charger's speed).
    pub(super) fn step(self, to_player: Vec2, dt: f32) -> (Self, Vec2) {
        match self {
            Self::Approach => {
                if to_player.length() <= CHARGE_TRIGGER_RANGE {
                    let direction = to_player.normalize_or(Vec2::X);
                    let windup = Self::WindUp {
                        left_secs: CHARGE_WINDUP_SECS,
                        direction,
                    };
                    (windup, Vec2::ZERO)
                } else {
                    (self, chase(to_player))
                }
            }
            Self::WindUp {
                left_secs,
                direction,
            } => {
                let left_secs = left_secs - dt;
                if left_secs > 0.0 {
                    (
                        Self::WindUp {
                            left_secs,
                            direction,
                        },
                        Vec2::ZERO,
                    )
                } else {
                    let dash = Self::Dash {
                        left_secs: CHARGE_DASH_SECS,
                        direction,
                    };
                    (dash, direction * CHARGE_DASH_SCALE)
                }
            }
            Self::Dash {
                left_secs,
                direction,
            } => {
                let left_secs = left_secs - dt;
                if left_secs > 0.0 {
                    let dash = Self::Dash {
                        left_secs,
                        direction,
                    };
                    (dash, direction * CHARGE_DASH_SCALE)
                } else {
                    let recover = Self::Recover {
                        left_secs: CHARGE_RECOVER_SECS,
                    };
                    (recover, Vec2::ZERO)
                }
            }
            Self::Recover { left_secs } => {
                let left_secs = left_secs - dt;
                if left_secs > 0.0 {
                    (Self::Recover { left_secs }, Vec2::ZERO)
                } else {
                    (Self::Approach, Vec2::ZERO)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hold_range_closes_in_then_circles_inward_without_backing_off() {
        let far = hold_range(Vec2::new(600.0, 0.0), 300.0, 1.0);
        assert_eq!(far, Vec2::X);
        for distance in [300.0, 100.0] {
            let held = hold_range(Vec2::new(distance, 0.0), 300.0, -1.0);
            assert!(held.x > 0.0 && held.x < held.y.abs(), "{held:?}");
            assert!(held.y < 0.0, "{held:?}");
        }
    }

    #[test]
    fn charger_tells_then_dashes_in_a_locked_direction_then_rests() {
        let dt = 1.0 / 60.0;
        let (mut charge, step) = Charge::Approach.step(Vec2::new(500.0, 0.0), dt);
        assert_eq!((charge, step), (Charge::Approach, Vec2::X));

        (charge, _) = charge.step(Vec2::new(200.0, 0.0), dt);
        assert!(matches!(charge, Charge::WindUp { .. }));

        // The player moves during the tell; the dash keeps the old direction.
        let mut dashed = Vec::new();
        for _ in 0..200 {
            let (next, step) = charge.step(Vec2::new(0.0, 200.0), dt);
            charge = next;
            if step != Vec2::ZERO {
                dashed.push(step);
            }
            if charge == Charge::Approach {
                break;
            }
        }
        assert_eq!(charge, Charge::Approach);
        assert!(!dashed.is_empty());
        assert!(dashed.iter().all(|s| *s == Vec2::X * CHARGE_DASH_SCALE));
    }
}

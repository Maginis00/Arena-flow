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
const CHARGE_TRIGGER_RANGE: f32 = 400.0;
/// PLACEHOLDER: seconds a charger stands still before dashing (the tell).
const CHARGE_WINDUP_SECS: f32 = 0.8;
/// PLACEHOLDER: seconds a dash lasts.
const CHARGE_DASH_SECS: f32 = 0.6;
/// PLACEHOLDER: dash speed as a multiple of the charger's own speed.
const CHARGE_DASH_SCALE: f32 = 6.0;
/// PLACEHOLDER: seconds a charger stands still after a dash (the opening).
const CHARGE_RECOVER_SECS: f32 = 0.9;
/// PLACEHOLDER: the fastest a dash may go at difficulty 1, in world units per
/// second (the player runs 280).
const DASH_CAP_AT_MIN: f32 = 500.0;
/// PLACEHOLDER: how much that limit rises per difficulty level.
const DASH_CAP_PER_LEVEL: f32 = 40.0;

/// PLACEHOLDER: a swarm circles the player at this distance.
const SWARM_RING: f32 = 200.0;
/// PLACEHOLDER: how far off the ring a swarm still blends circling with
/// closing in (or backing out); farther out it heads straight in.
const SWARM_RING_SLACK: f32 = 60.0;
/// PLACEHOLDER: seconds a new swarm circles before its first wind-up; counts
/// from its spawn, so it includes the walk in.
const SWARM_FIRST_CIRCLE_SECS: f32 = 4.0;
/// PLACEHOLDER: seconds a swarm circles between dives.
const SWARM_CIRCLE_SECS: f32 = 3.0;
/// PLACEHOLDER: seconds a swarm holds still before it dives (the tell).
const SWARM_WINDUP_SECS: f32 = 0.7;
/// PLACEHOLDER: seconds a dive lasts.
const SWARM_DIVE_SECS: f32 = 1.2;
/// PLACEHOLDER: dive speed as a multiple of the swarm's own speed.
const SWARM_DIVE_SCALE: f32 = 1.8;

/// The fastest a dash may go at difficulty `level`. The dash scales with the
/// enemy speed lever; this keeps it readable at the top.
pub(super) fn dash_cap(level: f32) -> f32 {
    DASH_CAP_AT_MIN + DASH_CAP_PER_LEVEL * (level - 1.0).max(0.0)
}

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
    /// The dash direction while winding up: what a player can read.
    pub(super) fn tell(self) -> Option<Vec2> {
        match self {
            Self::WindUp { direction, .. } => Some(direction),
            Self::Approach | Self::Dash { .. } | Self::Recover { .. } => None,
        }
    }

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

/// Where a swarm is in its circle, tell and dive. Every member of a pack
/// spawns in the same tick with the same timer, so the pack moves as one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Flock {
    /// Circling the player at the ring; `orbit` (+1 or -1) is the way round.
    Circle { left_secs: f32 },
    /// Holding still before the dive: what a player can read.
    WindUp { left_secs: f32 },
    /// Rushing at the player, homing.
    Dive { left_secs: f32 },
}

impl Default for Flock {
    fn default() -> Self {
        Self::Circle {
            left_secs: SWARM_FIRST_CIRCLE_SECS,
        }
    }
}

impl Flock {
    /// True while the pack holds still before diving.
    pub(super) fn winding_up(self) -> bool {
        matches!(self, Self::WindUp { .. })
    }

    /// Advance by `dt` and return the next phase and this tick's movement
    /// (a direction times a share of the swarm's speed).
    pub(super) fn step(self, to_player: Vec2, orbit: f32, dt: f32) -> (Self, Vec2) {
        match self {
            Self::Circle { left_secs } => {
                let left_secs = left_secs - dt;
                if left_secs > 0.0 {
                    (Self::Circle { left_secs }, circle(to_player, orbit))
                } else {
                    let windup = Self::WindUp {
                        left_secs: SWARM_WINDUP_SECS,
                    };
                    (windup, Vec2::ZERO)
                }
            }
            Self::WindUp { left_secs } => {
                let left_secs = left_secs - dt;
                if left_secs > 0.0 {
                    (Self::WindUp { left_secs }, Vec2::ZERO)
                } else {
                    let dive = Self::Dive {
                        left_secs: SWARM_DIVE_SECS,
                    };
                    (dive, chase(to_player) * SWARM_DIVE_SCALE)
                }
            }
            Self::Dive { left_secs } => {
                let left_secs = left_secs - dt;
                if left_secs > 0.0 {
                    let dive = Self::Dive { left_secs };
                    (dive, chase(to_player) * SWARM_DIVE_SCALE)
                } else {
                    let circle = Self::Circle {
                        left_secs: SWARM_CIRCLE_SECS,
                    };
                    (circle, Vec2::ZERO)
                }
            }
        }
    }
}

/// Head straight in from far away; near the ring, circle it and drift onto
/// it (out again after a dive).
fn circle(to_player: Vec2, orbit: f32) -> Vec2 {
    let distance = to_player.length();
    let toward = to_player.normalize_or_zero();
    if distance > SWARM_RING + SWARM_RING_SLACK {
        return toward;
    }
    let pull = ((distance - SWARM_RING) / SWARM_RING_SLACK).clamp(-1.0, 1.0);
    (toward * pull + toward.perp() * orbit).normalize_or_zero()
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
    fn swarm_closes_in_then_circles_then_backs_out_after_a_dive() {
        assert_eq!(circle(Vec2::new(600.0, 0.0), 1.0), Vec2::X);
        let on_ring = circle(Vec2::new(SWARM_RING, 0.0), 1.0);
        assert!(on_ring.x.abs() < 1e-6 && on_ring.y > 0.0, "{on_ring:?}");
        let too_close = circle(Vec2::new(SWARM_RING / 4.0, 0.0), -1.0);
        assert!(too_close.x < 0.0 && too_close.y < 0.0, "{too_close:?}");
    }

    #[test]
    fn swarm_circles_tells_dives_and_circles_again() {
        let dt = 1.0 / 60.0;
        let to_player = Vec2::new(SWARM_RING, 0.0);
        let mut flock = Flock::default();
        let mut phases = Vec::new();
        for _ in
            0..((SWARM_FIRST_CIRCLE_SECS + SWARM_WINDUP_SECS + SWARM_DIVE_SECS + 1.0) / dt) as usize
        {
            let (next, step) = flock.step(to_player, 1.0, dt);
            if next.winding_up() {
                assert_eq!(step, Vec2::ZERO);
            }
            if let Flock::Dive { .. } = next {
                assert_eq!(step, Vec2::X * SWARM_DIVE_SCALE);
            }
            let phase = std::mem::discriminant(&next);
            if phases.last() != Some(&phase) {
                phases.push(phase);
            }
            flock = next;
        }
        let circle = std::mem::discriminant(&Flock::default());
        let windup = std::mem::discriminant(&Flock::WindUp { left_secs: 0.0 });
        let dive = std::mem::discriminant(&Flock::Dive { left_secs: 0.0 });
        assert_eq!(phases, [circle, windup, dive, circle]);
    }

    #[test]
    fn dash_cap_rises_with_difficulty() {
        assert_eq!(dash_cap(1.0), DASH_CAP_AT_MIN);
        assert!(dash_cap(10.0) > dash_cap(5.0));
        assert_eq!(dash_cap(10.0), DASH_CAP_AT_MIN + 9.0 * DASH_CAP_PER_LEVEL);
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

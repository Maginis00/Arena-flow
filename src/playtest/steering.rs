//! Pure movement and aim math for simulated players. No ECS; unit-tested.

use bevy::math::Vec2;

/// sin(22.5 deg): a component below this share of the direction is dropped
/// when snapping to the eight directions `W` `A` `S` `D` can express.
const EIGHT_WAY_THRESHOLD: f32 = 0.383;

/// Push away from every threat inside `radius`, stronger the closer it is.
/// `strafe` adds a sideways share so the bot circles instead of backing into
/// a wall. Always turns the same way, like a player with a habit.
pub fn dodge(own: Vec2, threats: &[Vec2], radius: f32, strafe: f32) -> Vec2 {
    if radius <= 0.0 {
        return Vec2::ZERO;
    }
    threats
        .iter()
        .filter_map(|threat| {
            let away = own - *threat;
            let distance = away.length();
            (distance < radius).then(|| {
                let closeness = 1.0 - distance / radius;
                let away = away.normalize_or(Vec2::X);
                (away + away.perp() * strafe) * closeness * closeness
            })
        })
        .sum()
}

/// Step sideways out of every dash lane: `tells` are a charger's position and
/// dash direction. Only lanes that point at `own` within `lane` count, and the
/// push grows the nearer `own` is to the lane's centre line.
pub fn sidestep(own: Vec2, tells: &[(Vec2, Vec2)], lane: f32) -> Vec2 {
    if lane <= 0.0 {
        return Vec2::ZERO;
    }
    tells
        .iter()
        .filter_map(|(at, aim)| {
            let offset = own - *at;
            let along = offset.dot(*aim);
            let across = aim.perp_dot(offset);
            (along > 0.0 && across.abs() < lane).then(|| {
                // Step to the side already nearer; straight on the line, turn left.
                let side = if across >= 0.0 { 1.0 } else { -1.0 };
                aim.perp() * side * (1.0 - across.abs() / lane)
            })
        })
        .sum()
}

/// Push back toward the middle when within `margin` of a wall.
pub fn wall_push(own: Vec2, half_extents: Vec2, margin: f32) -> Vec2 {
    if margin <= 0.0 {
        return Vec2::ZERO;
    }
    let axis = |pos: f32, half: f32| {
        let into_wall = pos.abs() - (half - margin);
        if into_wall > 0.0 {
            -pos.signum() * (into_wall / margin).min(1.0)
        } else {
            0.0
        }
    };
    Vec2::new(axis(own.x, half_extents.x), axis(own.y, half_extents.y))
}

/// Snap a wish direction to what the movement keys can do: each axis is
/// -1, 0 or 1. Returns zero for a tiny or zero wish.
pub fn eight_way(wish: Vec2) -> (i8, i8) {
    let length = wish.length();
    if length < 1e-3 {
        return (0, 0);
    }
    let snap = |c: f32| {
        if c / length > EIGHT_WAY_THRESHOLD {
            1
        } else if c / length < -EIGHT_WAY_THRESHOLD {
            -1
        } else {
            0
        }
    };
    (snap(wish.x), snap(wish.y))
}

/// Rotate a unit aim direction by `radians`.
pub fn rotate(direction: Vec2, radians: f32) -> Vec2 {
    Vec2::from_angle(radians).rotate(direction)
}

/// The point in `points` closest to `own`.
pub fn nearest(own: Vec2, points: impl IntoIterator<Item = Vec2>) -> Option<Vec2> {
    points
        .into_iter()
        .min_by(|a, b| a.distance_squared(own).total_cmp(&b.distance_squared(own)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dodge_ignores_far_threats_and_backs_off_near_ones() {
        let own = Vec2::ZERO;
        assert_eq!(dodge(own, &[Vec2::new(500.0, 0.0)], 100.0, 0.0), Vec2::ZERO);
        let push = dodge(own, &[Vec2::new(50.0, 0.0)], 100.0, 0.0);
        assert!(push.x < 0.0 && push.y == 0.0, "{push:?}");
    }

    #[test]
    fn strafe_adds_a_sideways_share() {
        let push = dodge(Vec2::ZERO, &[Vec2::new(50.0, 0.0)], 100.0, 1.0);
        assert!(push.x < 0.0 && push.y != 0.0, "{push:?}");
    }

    #[test]
    fn sidestep_leaves_the_dash_lane_and_ignores_lanes_pointing_away() {
        let tell = (Vec2::ZERO, Vec2::X);
        let left_of_lane = sidestep(Vec2::new(200.0, 10.0), &[tell], 50.0);
        assert!(
            left_of_lane.y > 0.0 && left_of_lane.x == 0.0,
            "{left_of_lane:?}"
        );
        assert_eq!(sidestep(Vec2::new(200.0, 80.0), &[tell], 50.0), Vec2::ZERO);
        assert_eq!(sidestep(Vec2::new(-200.0, 0.0), &[tell], 50.0), Vec2::ZERO);
    }

    #[test]
    fn wall_push_points_inward_only_near_walls() {
        let half = Vec2::new(600.0, 330.0);
        assert_eq!(wall_push(Vec2::ZERO, half, 100.0), Vec2::ZERO);
        let push = wall_push(Vec2::new(590.0, -320.0), half, 100.0);
        assert!(push.x < 0.0 && push.y > 0.0, "{push:?}");
    }

    #[test]
    fn eight_way_snaps_to_keys() {
        assert_eq!(eight_way(Vec2::ZERO), (0, 0));
        assert_eq!(eight_way(Vec2::new(1.0, 0.1)), (1, 0));
        assert_eq!(eight_way(Vec2::new(-1.0, 1.0)), (-1, 1));
        assert_eq!(eight_way(Vec2::new(0.0, -3.0)), (0, -1));
    }
}

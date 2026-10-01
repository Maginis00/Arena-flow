//! Where a new enemy appears: on the arena perimeter, but never right on top
//! of the player.

use bevy::prelude::*;

/// Picks the spawn point for a perimeter `candidate`.
///
/// A candidate closer than `safe_radius` to the player is mirrored through the
/// arena centre, which puts it on the opposite wall. If the mirror is too
/// close as well (only in an arena smaller than the radius), the farther of
/// the two wins. With no player alive the candidate is used as is.
pub(super) fn spawn_point(candidate: Vec2, player: Option<Vec2>, safe_radius: f32) -> Vec2 {
    let Some(player) = player else {
        return candidate;
    };
    if candidate.distance(player) >= safe_radius {
        return candidate;
    }
    let mirrored = -candidate;
    if mirrored.distance(player) > candidate.distance(player) {
        mirrored
    } else {
        candidate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn far_candidate_is_kept() {
        let at = Vec2::new(500.0, 300.0);
        assert_eq!(spawn_point(at, Some(Vec2::ZERO), 200.0), at);
    }

    #[test]
    fn no_player_keeps_candidate() {
        let at = Vec2::new(10.0, 0.0);
        assert_eq!(spawn_point(at, None, 200.0), at);
    }

    #[test]
    fn close_candidate_moves_to_opposite_wall() {
        let player = Vec2::new(550.0, 280.0);
        let at = Vec2::new(576.0, 306.0);
        let picked = spawn_point(at, Some(player), 200.0);
        assert_eq!(picked, -at);
        assert!(picked.distance(player) >= 200.0);
    }

    #[test]
    fn candidate_exactly_on_radius_is_kept() {
        let at = Vec2::new(200.0, 0.0);
        assert_eq!(spawn_point(at, Some(Vec2::ZERO), 200.0), at);
    }

    #[test]
    fn tiny_arena_takes_the_farther_point() {
        // Player at the centre of a small arena: both points are equally
        // close, so the candidate stays.
        let at = Vec2::new(50.0, 0.0);
        assert_eq!(spawn_point(at, Some(Vec2::ZERO), 200.0), at);
        // Player off-centre: the mirror is farther and wins, though still
        // inside the radius.
        let player = Vec2::new(30.0, 0.0);
        assert_eq!(spawn_point(at, Some(player), 200.0), -at);
    }
}

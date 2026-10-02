//! Where a dropped pickup lands. Pure function, unit-tested.

use bevy::math::Vec2;

/// PLACEHOLDER: under `Far` rules a buff lands this far beyond the kill, on
/// the side away from the player.
pub const FAR_THROW: f32 = 160.0;
/// PLACEHOLDER: and stays this many seconds instead of the classic eight.
pub const FAR_LIFETIME_SECS: f32 = 5.0;
/// Keep thrown pickups this far inside the walls so they stay reachable.
const WALL_INSET: f32 = 30.0;

/// Throw a drop from `kill` directly away from `player`, kept inside the arena.
/// A kill on top of the player throws nowhere: the drop stays at the kill.
pub fn thrown(kill: Vec2, player: Vec2, half_extents: Vec2) -> Vec2 {
    let away = (kill - player).normalize_or_zero();
    let limit = (half_extents - Vec2::splat(WALL_INSET)).max(Vec2::ZERO);
    (kill + away * FAR_THROW).clamp(-limit, limit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_fly_away_from_the_player_and_stay_inside() {
        let half = Vec2::new(600.0, 400.0);
        assert_eq!(
            thrown(Vec2::new(100.0, 0.0), Vec2::ZERO, half),
            Vec2::new(260.0, 0.0)
        );
        let at_wall = thrown(Vec2::new(550.0, 0.0), Vec2::ZERO, half);
        assert_eq!(at_wall, Vec2::new(570.0, 0.0));
        assert_eq!(thrown(Vec2::ONE, Vec2::ONE, half), Vec2::ONE);
    }
}

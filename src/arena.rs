//! Arena bounds and out-of-bounds despawn.

use crate::app_setup::SimSet;
use bevy::prelude::*;

/// Axis-aligned arena rectangle centred on the world origin.
#[derive(Resource, Debug, Clone, Copy)]
pub struct ArenaBounds {
    half_extents: Vec2,
}

impl ArenaBounds {
    pub const fn new(half_extents: Vec2) -> Self {
        Self { half_extents }
    }

    pub const fn half_extents(&self) -> Vec2 {
        self.half_extents
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.abs().cmple(self.half_extents).all()
    }

    /// Clamp a box of `half_size` so it stays fully inside the arena.
    pub fn clamp(&self, point: Vec2, half_size: Vec2) -> Vec2 {
        let limit = (self.half_extents - half_size).max(Vec2::ZERO);
        point.clamp(-limit, limit)
    }

    /// A point on the arena perimeter, inset by `inset`, parameterised by
    /// `t` in `[0, 1)` walking clockwise from the top-left corner.
    pub fn perimeter_point(&self, t: f32, inset: f32) -> Vec2 {
        let h = (self.half_extents - Vec2::splat(inset)).max(Vec2::ZERO);
        let (w, ht) = (h.x * 2.0, h.y * 2.0);
        let perimeter = 2.0 * (w + ht);
        if perimeter <= f32::EPSILON {
            return Vec2::ZERO;
        }
        let d = t.rem_euclid(1.0) * perimeter;
        if d < w {
            Vec2::new(-h.x + d, h.y)
        } else if d < w + ht {
            Vec2::new(h.x, h.y - (d - w))
        } else if d < 2.0 * w + ht {
            Vec2::new(h.x - (d - w - ht), -h.y)
        } else {
            Vec2::new(-h.x, -h.y + (d - 2.0 * w - ht))
        }
    }
}

/// Entities with this marker are despawned once their centre leaves the arena.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct DespawnOutsideArena;

/// PLACEHOLDER: arena size in world units (fits a 1280x720 window).
const ARENA_SIZE: Vec2 = Vec2::new(1200.0, 660.0);

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ArenaBounds::new(ARENA_SIZE / 2.0))
            .add_systems(FixedUpdate, despawn_outside.in_set(SimSet::Cleanup));
    }
}

fn despawn_outside(
    mut commands: Commands,
    bounds: Res<ArenaBounds>,
    query: Query<(Entity, &Transform), With<DespawnOutsideArena>>,
) {
    for (entity, transform) in &query {
        if !bounds.contains(transform.translation.truncate()) {
            commands.entity(entity).try_despawn();
        }
    }
}

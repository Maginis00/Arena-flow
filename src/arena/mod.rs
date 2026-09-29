//! Arena bounds and out-of-bounds despawn.

pub mod api;

use crate::app_setup::api::SimSet;
use api::{ArenaBounds, DespawnOutsideArena};
use bevy::prelude::*;

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

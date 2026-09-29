//! Light camera follow: the view drifts toward the player but stays clamped
//! so it never shows much beyond the arena. Presentation only, so it runs in
//! `Update`, not in the simulation.

pub mod api;

use crate::arena::api::ArenaBounds;
use crate::player::api::Player;
use bevy::prelude::*;

/// PLACEHOLDER: how quickly the camera catches up (per second, exponential).
const FOLLOW_RATE: f32 = 4.0;
/// PLACEHOLDER: how far past the arena edge the view may show, in world units.
const EDGE_MARGIN: f32 = 30.0;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, follow_player);
    }
}

fn follow_player(
    time: Res<Time>,
    bounds: Res<ArenaBounds>,
    player: Option<Single<&Transform, (With<Player>, Without<Camera2d>)>>,
    camera: Option<Single<(&mut Transform, &Projection), (With<Camera2d>, Without<Player>)>>,
) {
    let Some(camera) = camera else {
        return;
    };
    let (mut transform, projection) = camera.into_inner();
    let Projection::Orthographic(ortho) = projection else {
        return;
    };
    // No player (dead, awaiting respawn): drift back to the arena centre.
    let wanted = player.map_or(Vec2::ZERO, |p| p.translation.truncate());
    let view_half = ortho.area.half_size();
    let limit = (bounds.half_extents() + Vec2::splat(EDGE_MARGIN) - view_half).max(Vec2::ZERO);
    let target = wanted.clamp(-limit, limit);

    let here = transform.translation.truncate();
    let blend = 1.0 - (-FOLLOW_RATE * time.delta_secs()).exp();
    let next = here.lerp(target, blend);
    transform.translation = next.extend(transform.translation.z);
}

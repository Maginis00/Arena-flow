//! Window-adjacent setup: clear color, the fixed 2D camera, the fixed timestep,
//! and the ordering of simulation sets inside `FixedUpdate`.

pub mod api;

use api::{FIXED_HZ, SimSet};
use bevy::prelude::*;

const CLEAR_COLOR: Color = Color::srgb(0.06, 0.06, 0.08);
/// PLACEHOLDER: world units visible vertically. The arena is 660 tall, so the
/// view is slightly tighter than the arena and the camera follows lightly.
const VIEW_HEIGHT: f32 = 580.0;

pub struct AppSetupPlugin;

impl Plugin for AppSetupPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(CLEAR_COLOR))
            .insert_resource(Time::<Fixed>::from_hz(FIXED_HZ))
            .configure_sets(
                FixedUpdate,
                (
                    SimSet::Intent,
                    SimSet::Spawn,
                    SimSet::Movement,
                    SimSet::Detect,
                    SimSet::Resolve,
                    SimSet::Progress,
                    SimSet::Direct,
                    SimSet::Cleanup,
                )
                    .chain(),
            )
            .add_systems(Startup, spawn_camera);
    }
}

fn spawn_camera(mut commands: Commands) {
    // Orthographic camera; the camera plugin moves it with a light follow.
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: VIEW_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

//! Window-adjacent setup: clear color, the fixed 2D camera, the fixed timestep,
//! and the ordering of simulation sets inside `FixedUpdate`.

pub mod api;

use api::{FIXED_HZ, SimSet};
use bevy::prelude::*;

const CLEAR_COLOR: Color = Color::srgb(0.06, 0.06, 0.08);

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
    // Fixed orthographic camera centred on the arena origin.
    commands.spawn(Camera2d);
}

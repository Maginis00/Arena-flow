use bevy::prelude::*;
use flow_arena::FlowArenaPlugins;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "flow_arena".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FlowArenaPlugins)
        .run()
}

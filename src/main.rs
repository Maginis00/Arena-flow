use bevy::prelude::*;
use flow_arena::FlowArenaPlugins;

fn main() -> AppExit {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "flow_arena".into(),
            resolution: (1280, 720).into(),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(FlowArenaPlugins);
    // Sound and particles: only for a person playing, never for bots.
    #[cfg(feature = "fx")]
    app.add_plugins(flow_arena::fx::FxPlugin);
    app.run()
}

//! A game with a window writes its session to `playtests/`; reading the file
//! back gives the same record telemetry holds in memory.

use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy::window::PrimaryWindow;
use flow_arena::FlowArenaPlugins;
use flow_arena::debug_render::DebugRenderPlugin;
use flow_arena::playtest::{PlaytestBotPlugin, SkillTier};
use flow_arena::telemetry::{SESSION_DIR, SessionRecord, read_session_file};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

fn session_files() -> HashSet<PathBuf> {
    std::fs::read_dir(SESSION_DIR)
        .map(|entries| entries.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default()
}

#[test]
fn windowed_session_is_written_and_reads_back() {
    let before = session_files();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin))
        .add_plugins(FlowArenaPlugins.build().disable::<DebugRenderPlugin>())
        .add_plugins(PlaytestBotPlugin {
            params: SkillTier::Casual.params(),
            seed: 1,
        })
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 60.0,
        )));
    // Stands in for the real window: the file writer only checks for one.
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    for _ in 0..(60 * 90) {
        app.update();
    }

    let written: Vec<PathBuf> = session_files().difference(&before).cloned().collect();
    let [path] = written.as_slice() else {
        panic!("expected one new session file, got {written:?}");
    };
    let from_file = read_session_file(path);
    let _ = std::fs::remove_file(path);
    let from_file = from_file.expect("session file parses");

    let in_memory = app.world().resource::<SessionRecord>();
    assert!(!from_file.waves.is_empty());
    // The last wave may still wait for its decision; everything written must match.
    assert_eq!(
        from_file.waves[..],
        in_memory.waves[..from_file.waves.len()]
    );
    assert!(from_file.waves.iter().all(|w| w.decision.is_some()));
}

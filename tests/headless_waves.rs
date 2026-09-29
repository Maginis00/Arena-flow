//! Runs the whole simulation headless with nobody at the controls.
//! An idle player should die every wave, and the director should walk
//! difficulty down to the floor, honouring hysteresis on the way.

use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use flow_arena::FlowArenaPlugins;
use flow_arena::debug_render::DebugRenderPlugin;
use flow_arena::flow_director::api::{DecisionReason, DifficultyAdjusted};
use flow_arena::waves::api::WaveReport;
use std::time::Duration;

#[derive(Resource, Default)]
struct Observed {
    adjustments: Vec<DifficultyAdjusted>,
    reports: Vec<WaveReport>,
}

fn observe(
    mut observed: ResMut<Observed>,
    mut adjusted: MessageReader<DifficultyAdjusted>,
    mut reports: MessageReader<WaveReport>,
) {
    observed.adjustments.extend(adjusted.read().copied());
    observed.reports.extend(reports.read().copied());
}

#[test]
fn idle_player_fails_waves_and_difficulty_falls_to_floor() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin))
        .add_plugins(FlowArenaPlugins.build().disable::<DebugRenderPlugin>())
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .init_resource::<Observed>()
        .add_systems(Update, observe);

    // Five simulated minutes at 60 fps.
    for _ in 0..(60 * 60 * 5) {
        app.update();
    }

    let observed = app.world().resource::<Observed>();
    assert!(
        observed.reports.len() >= 3,
        "expected several waves, got {}",
        observed.reports.len()
    );
    assert!(
        observed.reports.iter().all(|r| r.player_died),
        "idle player should die every wave"
    );

    let levels: Vec<u8> = observed
        .adjustments
        .iter()
        .map(|a| a.difficulty.get())
        .collect();
    assert_eq!(
        observed.adjustments.first().map(|a| a.reason),
        Some(DecisionReason::Initial)
    );
    // 3 initial, then -1 (fresh memory), -1 (same signal repeats), then held at the floor.
    assert_eq!(&levels[..4], &[3, 2, 1, 1], "levels: {levels:?}");
    assert_eq!(observed.adjustments[3].reason, DecisionReason::HeldAtMin);

    // Each report records the difficulty the wave actually ran at.
    let ran_at: Vec<u8> = observed
        .reports
        .iter()
        .map(|r| r.difficulty.get())
        .collect();
    assert_eq!(&ran_at[..3], &[3, 2, 1], "ran at: {ran_at:?}");
}

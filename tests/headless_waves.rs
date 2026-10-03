//! Runs the whole simulation headless: once with nobody at the controls, and
//! once per weapon with a simple aim bot, to check the plugins wire together.

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::{ButtonState, InputPlugin};
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use flow_arena::FlowArenaPlugins;
use flow_arena::app_setup::SimSet;
use flow_arena::debug_render::DebugRenderPlugin;
use flow_arena::enemies::Enemy;
use flow_arena::flow_director::{DecisionReason, DifficultyAdjusted};
use flow_arena::player::{FireRequested, Player};
use flow_arena::waves::WaveReport;
use flow_arena::weapons::{ShotFired, WeaponKind};
use std::time::Duration;

#[derive(Resource, Default)]
struct Observed {
    adjustments: Vec<DifficultyAdjusted>,
    reports: Vec<WaveReport>,
    weapons_fired: Vec<WeaponKind>,
}

fn observe(
    mut observed: ResMut<Observed>,
    mut adjusted: MessageReader<DifficultyAdjusted>,
    mut reports: MessageReader<WaveReport>,
    mut shots: MessageReader<ShotFired>,
) {
    observed.adjustments.extend(adjusted.read().copied());
    observed.reports.extend(reports.read().copied());
    observed
        .weapons_fired
        .extend(shots.read().map(|s| s.weapon));
}

/// Fires at the nearest enemy every tick, standing still.
fn aim_bot(
    player: Query<(Entity, &Transform), With<Player>>,
    enemies: Query<&Transform, With<Enemy>>,
    mut fire: MessageWriter<FireRequested>,
) {
    for (shooter, transform) in &player {
        let origin = transform.translation.truncate();
        let nearest = enemies
            .iter()
            .map(|t| t.translation.truncate())
            .min_by(|a, b| {
                a.distance_squared(origin)
                    .total_cmp(&b.distance_squared(origin))
            });
        if let Some(target) = nearest {
            let direction = (target - origin).normalize_or_zero();
            if direction != Vec2::ZERO {
                fire.write(FireRequested {
                    shooter,
                    origin,
                    direction,
                });
            }
        }
    }
}

fn headless_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin))
        .add_plugins(FlowArenaPlugins.build().disable::<DebugRenderPlugin>())
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .init_resource::<Observed>()
        .add_systems(Update, observe);
    app
}

fn run_minutes(app: &mut App, minutes: u32) {
    for _ in 0..(60 * 60 * minutes) {
        app.update();
    }
}

fn press(app: &mut App, key_code: KeyCode, digit: &str) {
    for state in [ButtonState::Pressed, ButtonState::Released] {
        app.world_mut().write_message(KeyboardInput {
            key_code,
            logical_key: Key::Character(digit.into()),
            state,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
        app.update();
    }
}

#[test]
fn idle_player_retries_the_wave_and_difficulty_falls_to_floor() {
    let mut app = headless_app();
    run_minutes(&mut app, 5);

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

    // Death retries the same wave with the attempt counting up.
    assert!(observed.reports.iter().all(|r| r.index.0 == 1));
    let attempts: Vec<u32> = observed.reports.iter().map(|r| r.attempt).collect();
    assert_eq!(&attempts[..3], &[1, 2, 3]);

    let levels: Vec<f32> = observed
        .adjustments
        .iter()
        .map(|a| a.difficulty.level())
        .collect();
    assert_eq!(
        observed.adjustments.first().map(|a| a.reason),
        Some(DecisionReason::Initial)
    );
    // 3 initial, then -1 per death, then held at the floor.
    assert_eq!(&levels[..4], &[3.0, 2.0, 1.0, 1.0], "levels: {levels:?}");
    assert_eq!(observed.adjustments[3].reason, DecisionReason::HeldAtMin);

    let ran_at: Vec<f32> = observed
        .reports
        .iter()
        .map(|r| r.difficulty.level())
        .collect();
    assert_eq!(&ran_at[..3], &[3.0, 2.0, 1.0], "ran at: {ran_at:?}");
}

fn bot_clears_waves(key: KeyCode, digit: &str, weapon: WeaponKind) {
    let mut app = headless_app();
    app.add_systems(FixedUpdate, aim_bot.in_set(SimSet::Intent));
    press(&mut app, key, digit);
    run_minutes(&mut app, 3);
    let observed = app.world().resource::<Observed>();
    assert!(!observed.weapons_fired.is_empty());
    assert!(
        observed.weapons_fired.iter().all(|w| *w == weapon),
        "{digit}: wrong weapon fired"
    );
    let reports = &observed.reports;
    eprintln!(
        "{weapon}: {:?}",
        reports
            .iter()
            .map(|r| (
                r.index.0,
                r.difficulty.level(),
                r.cleared(),
                r.damage_taken,
                r.pickups_collected
            ))
            .collect::<Vec<_>>()
    );
    let cleared = reports.iter().filter(|r| r.cleared()).count();
    assert!(
        cleared >= 2,
        "{digit}: expected cleared waves, got {reports:#?}"
    );
    assert!(reports.iter().all(|r| r.shots_hit <= r.shots_fired));
    assert!(reports.iter().any(|r| r.shots_hit > 0));
}

#[test]
fn aim_bot_clears_waves_with_projectile() {
    bot_clears_waves(KeyCode::Digit1, "1", WeaponKind::Projectile);
}

#[test]
fn aim_bot_clears_waves_with_hitscan() {
    bot_clears_waves(KeyCode::Digit2, "2", WeaponKind::Hitscan);
}

#[test]
fn aim_bot_clears_waves_with_melee() {
    bot_clears_waves(KeyCode::Digit3, "3", WeaponKind::Melee);
}

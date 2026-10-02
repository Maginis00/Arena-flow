//! Player spawn, movement, mouse aim and fire intent, and a full refill after
//! each cleared wave.
//!
//! Input is sampled in `Update` into [`PlayerInput`]; the simulation reads that
//! snapshot in `FixedUpdate`.

pub mod api;

use crate::app_setup::api::SimSet;
use crate::arena::api::ArenaBounds;
use crate::combat::api::{HealGranted, Health, Hitbox, PlayerDied, Team};
use crate::pickups::api::{Effects, EffectsChanged};
use crate::waves::api::{WaveCleared, WaveStarted};
use api::{FireRequested, Player, PlayerSpawned};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// PLACEHOLDER: player movement speed in world units per second.
const PLAYER_SPEED: f32 = 280.0;
/// PLACEHOLDER: player max hit points.
const PLAYER_MAX_HP: u32 = 100;
/// PLACEHOLDER: player box half-size.
const PLAYER_HALF_SIZE: f32 = 14.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlayerSpawned>()
            .add_message::<FireRequested>()
            .init_resource::<PlayerInput>()
            .init_resource::<MovementEffects>()
            .add_systems(Startup, spawn_player)
            .add_systems(Update, sample_input)
            .add_systems(
                FixedUpdate,
                (track_effects, request_fire).in_set(SimSet::Intent),
            )
            .add_systems(FixedUpdate, move_player.in_set(SimSet::Movement))
            .add_systems(
                FixedUpdate,
                (
                    despawn_on_death,
                    respawn_on_wave_start,
                    refill_on_wave_cleared,
                )
                    .chain()
                    .in_set(SimSet::Cleanup),
            );
    }
}

/// Latest input snapshot, written in `Update`, read in `FixedUpdate`.
#[derive(Resource, Debug, Default, Clone, Copy)]
struct PlayerInput {
    /// Normalised movement direction, or zero.
    movement: Vec2,
    /// Cursor position in world space, if the cursor is over the window.
    aim_world: Option<Vec2>,
    fire_held: bool,
}

/// Latest pickup effects; the player only uses `move_speed`.
#[derive(Resource, Debug, Default, Clone, Copy)]
struct MovementEffects(Effects);

fn track_effects(mut changed: MessageReader<EffectsChanged>, mut effects: ResMut<MovementEffects>) {
    if let Some(latest) = changed.read().last() {
        effects.0 = latest.effects;
    }
}

fn player_bundle() -> impl Bundle {
    (
        Player,
        Team::Player,
        Health::full(PLAYER_MAX_HP),
        Hitbox::square(PLAYER_HALF_SIZE),
        // Drawn above enemies (0.5), below projectiles (1.0).
        Transform::from_xyz(0.0, 0.0, 0.8),
    )
}

fn spawn_player(mut commands: Commands, mut spawned: MessageWriter<PlayerSpawned>) {
    let player = commands.spawn(player_bundle()).id();
    spawned.write(PlayerSpawned {
        player,
        max_hp: PLAYER_MAX_HP,
    });
}

fn sample_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    camera: Option<Single<(&Camera, &GlobalTransform), With<Camera2d>>>,
    mut input: ResMut<PlayerInput>,
) {
    let axis = |neg: KeyCode, pos: KeyCode| {
        f32::from(u8::from(keys.pressed(pos))) - f32::from(u8::from(keys.pressed(neg)))
    };
    input.movement = Vec2::new(
        axis(KeyCode::KeyA, KeyCode::KeyD),
        axis(KeyCode::KeyS, KeyCode::KeyW),
    )
    .normalize_or_zero();
    input.fire_held = mouse.pressed(MouseButton::Left);
    input.aim_world = match (window, camera) {
        (Some(window), Some(camera)) => {
            let (camera, camera_transform) = *camera;
            window
                .cursor_position()
                .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
        }
        _ => None,
    };
}

fn move_player(
    time: Res<Time>,
    input: Res<PlayerInput>,
    effects: Res<MovementEffects>,
    bounds: Res<ArenaBounds>,
    mut player: Query<(&mut Transform, &Hitbox), With<Player>>,
) {
    for (mut transform, hitbox) in &mut player {
        let next = transform.translation.truncate()
            + input.movement * PLAYER_SPEED * effects.0.move_speed * time.delta_secs();
        let clamped = bounds.clamp(next, hitbox.half_extents);
        transform.translation = clamped.extend(transform.translation.z);
    }
}

fn request_fire(
    input: Res<PlayerInput>,
    player: Query<(Entity, &Transform, &Health), With<Player>>,
    mut fire: MessageWriter<FireRequested>,
) {
    if !input.fire_held {
        return;
    }
    let Some(aim) = input.aim_world else {
        return;
    };
    for (shooter, transform, health) in &player {
        if health.is_dead() {
            continue;
        }
        let origin = transform.translation.truncate();
        let direction = (aim - origin).normalize_or_zero();
        if direction != Vec2::ZERO {
            fire.write(FireRequested {
                shooter,
                origin,
                direction,
            });
        }
    }
}

fn despawn_on_death(mut commands: Commands, mut died: MessageReader<PlayerDied>) {
    for death in died.read() {
        commands.entity(death.player).try_despawn();
    }
}

/// A cleared wave refills the player's hp for the next one, so every wave
/// starts from the same footing and its risk is its own.
fn refill_on_wave_cleared(
    mut cleared: MessageReader<WaveCleared>,
    player: Query<Entity, With<Player>>,
    mut heals: MessageWriter<HealGranted>,
) {
    // Only the fact that a wave was cleared matters, not how many.
    if cleared.read().count() == 0 {
        return;
    }
    for target in &player {
        heals.write(HealGranted {
            target,
            amount: PLAYER_MAX_HP,
        });
    }
}

/// After intermission the next wave starts; a dead player comes back at centre.
fn respawn_on_wave_start(
    mut commands: Commands,
    mut started: MessageReader<WaveStarted>,
    player: Query<(), With<Player>>,
    mut spawned: MessageWriter<PlayerSpawned>,
) {
    // Only the fact that a wave started matters, not how many.
    if started.read().count() == 0 || !player.is_empty() {
        return;
    }
    let player = commands.spawn(player_bundle()).id();
    spawned.write(PlayerSpawned {
        player,
        max_hp: PLAYER_MAX_HP,
    });
}

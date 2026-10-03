//! Sound and particles for a person playing in a window. Built only with the
//! `fx` feature and added only by `main.rs`, so bots, headless runs and watched
//! bot windows never see it. It reads simulation messages in `Update` and
//! writes nothing back: the simulation is the same with or without it.

pub mod api;
mod particles;
mod sounds;

use bevy::prelude::*;

/// Toggles all sound effects.
const MUTE_KEY: KeyCode = KeyCode::KeyM;

pub struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Muted>()
            .init_resource::<particles::EnemyColors>()
            .add_systems(Startup, (sounds::load_sounds, particles::spawn_hurt_flash))
            .add_systems(
                Update,
                (
                    toggle_mute,
                    sounds::play_sounds,
                    sounds::play_charge_tells,
                    particles::remember_enemy_colors,
                    particles::burst_on_kill.after(particles::remember_enemy_colors),
                    particles::spark_on_hit,
                    particles::flash_on_player_hit,
                    particles::move_particles,
                    particles::fade_hurt_flash,
                ),
            );
    }
}

/// Sound effects are silent while this is set.
#[derive(Resource, Debug, Default)]
struct Muted(bool);

fn toggle_mute(keys: Res<ButtonInput<KeyCode>>, mut muted: ResMut<Muted>) {
    if keys.just_pressed(MUTE_KEY) {
        muted.0 = !muted.0;
    }
}

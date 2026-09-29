//! Placeholder visuals: solid color boxes attached to simulation entities, plus
//! the arena border. Gameplay plugins never add sprites themselves.

pub mod api;

use crate::arena::api::ArenaBounds;
use crate::combat::api::{Hitbox, Projectile};
use crate::enemies::api::Enemy;
use crate::player::api::Player;
use bevy::prelude::*;

const PLAYER_COLOR: Color = Color::srgb(0.0, 0.9, 0.9);
const ENEMY_COLOR: Color = Color::srgb(0.9, 0.15, 0.15);
const PROJECTILE_COLOR: Color = Color::srgb(1.0, 0.9, 0.1);
const BORDER_COLOR: Color = Color::srgb(0.3, 0.3, 0.32);
const BORDER_THICKNESS: f32 = 4.0;

pub struct DebugRenderPlugin;

impl Plugin for DebugRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_border)
            .add_systems(Update, attach_boxes);
    }
}

type NewVisible = (
    Or<(Added<Player>, Added<Enemy>, Added<Projectile>)>,
    Without<Sprite>,
);

fn attach_boxes(
    mut commands: Commands,
    added: Query<(Entity, &Hitbox, Has<Player>, Has<Enemy>), NewVisible>,
) {
    for (entity, hitbox, is_player, is_enemy) in &added {
        let color = if is_player {
            PLAYER_COLOR
        } else if is_enemy {
            ENEMY_COLOR
        } else {
            PROJECTILE_COLOR
        };
        // The entity may have been despawned by a fixed tick earlier this frame.
        if let Ok(mut e) = commands.get_entity(entity) {
            e.insert(Sprite::from_color(color, hitbox.half_extents * 2.0));
        }
    }
}

fn spawn_border(mut commands: Commands, bounds: Res<ArenaBounds>) {
    let h = bounds.half_extents();
    let t = BORDER_THICKNESS;
    let horizontal = Vec2::new(h.x * 2.0 + t * 2.0, t);
    let vertical = Vec2::new(t, h.y * 2.0 + t * 2.0);
    let edges = [
        (Vec2::new(0.0, h.y + t / 2.0), horizontal),
        (Vec2::new(0.0, -h.y - t / 2.0), horizontal),
        (Vec2::new(h.x + t / 2.0, 0.0), vertical),
        (Vec2::new(-h.x - t / 2.0, 0.0), vertical),
    ];
    for (at, size) in edges {
        commands.spawn((
            Sprite::from_color(BORDER_COLOR, size),
            Transform::from_translation(at.extend(-1.0)),
        ));
    }
}

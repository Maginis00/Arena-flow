//! Placeholder visuals: solid color boxes attached to simulation entities, the
//! arena border, and brief line flashes for hitscan and melee shots. Gameplay
//! plugins never add visuals themselves.

pub mod api;

use crate::arena::api::ArenaBounds;
use crate::combat::api::{Hitbox, Projectile};
use crate::enemies::api::Enemy;
use crate::pickups::api::PickupKind;
use crate::player::api::Player;
use crate::weapons::api::{Delivery, ShotFired};
use bevy::prelude::*;

const PLAYER_COLOR: Color = Color::srgb(0.0, 0.9, 0.9);
const ENEMY_COLOR: Color = Color::srgb(0.9, 0.15, 0.15);
const PROJECTILE_COLOR: Color = Color::srgb(1.0, 0.9, 0.1);
const BORDER_COLOR: Color = Color::srgb(0.3, 0.3, 0.32);
const BORDER_THICKNESS: f32 = 4.0;
/// Pickups are green; the shade tells them apart.
const OVERDRIVE_COLOR: Color = Color::srgb(0.2, 1.0, 0.2);
const HEAVY_COLOR: Color = Color::srgb(0.05, 0.5, 0.1);
const MEND_COLOR: Color = Color::srgb(0.7, 1.0, 0.7);
/// How long a hitscan tracer or melee swing stays on screen.
const FLASH_SECS: f32 = 0.08;
/// Line segments used to draw a melee arc.
const ARC_SEGMENTS: u16 = 12;

pub struct DebugRenderPlugin;

impl Plugin for DebugRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Flashes>()
            .add_systems(Startup, spawn_border)
            .add_systems(Update, (attach_boxes, attach_pickup_boxes, draw_flashes));
    }
}

#[derive(Debug, Clone, Copy)]
struct Flash {
    origin: Vec2,
    direction: Vec2,
    delivery: Delivery,
    remaining_secs: f32,
}

#[derive(Resource, Debug, Default)]
struct Flashes(Vec<Flash>);

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
        insert_box(&mut commands, entity, color, hitbox);
    }
}

fn attach_pickup_boxes(
    mut commands: Commands,
    added: Query<(Entity, &Hitbox, &PickupKind), (Added<PickupKind>, Without<Sprite>)>,
) {
    for (entity, hitbox, kind) in &added {
        let color = match kind {
            PickupKind::Overdrive => OVERDRIVE_COLOR,
            PickupKind::Heavy => HEAVY_COLOR,
            PickupKind::Mend => MEND_COLOR,
        };
        insert_box(&mut commands, entity, color, hitbox);
    }
}

fn insert_box(commands: &mut Commands, entity: Entity, color: Color, hitbox: &Hitbox) {
    // The entity may have been despawned by a fixed tick earlier this frame.
    if let Ok(mut e) = commands.get_entity(entity) {
        e.insert(Sprite::from_color(color, hitbox.half_extents * 2.0));
    }
}

fn draw_flashes(
    time: Res<Time>,
    mut shots: MessageReader<ShotFired>,
    mut flashes: ResMut<Flashes>,
    mut gizmos: Gizmos,
) {
    flashes.0.extend(
        shots
            .read()
            .filter(|s| !matches!(s.delivery, Delivery::Projectile { .. }))
            .map(|s| Flash {
                origin: s.origin,
                direction: s.direction,
                delivery: s.delivery,
                remaining_secs: FLASH_SECS,
            }),
    );
    for flash in &flashes.0 {
        match flash.delivery {
            Delivery::Projectile { .. } => {}
            Delivery::Hitscan { range, .. } => {
                gizmos.line_2d(
                    flash.origin,
                    flash.origin + flash.direction * range,
                    PROJECTILE_COLOR,
                );
            }
            Delivery::Melee {
                radius, half_angle, ..
            } => {
                let point = |angle: f32| {
                    flash.origin + Vec2::from_angle(angle).rotate(flash.direction) * radius
                };
                gizmos.line_2d(flash.origin, point(-half_angle), PROJECTILE_COLOR);
                gizmos.line_2d(flash.origin, point(half_angle), PROJECTILE_COLOR);
                let step = 2.0 * half_angle / f32::from(ARC_SEGMENTS);
                for i in 0..ARC_SEGMENTS {
                    let a = -half_angle + step * f32::from(i);
                    gizmos.line_2d(point(a), point(a + step), PROJECTILE_COLOR);
                }
            }
        }
    }
    let dt = time.delta_secs();
    flashes.0.retain_mut(|flash| {
        flash.remaining_secs -= dt;
        flash.remaining_secs > 0.0
    });
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

//! Placeholder visuals: solid color boxes attached to simulation entities, the
//! arena border, and brief line flashes for hitscan and melee shots. Gameplay
//! plugins never add visuals themselves.

pub mod api;

use crate::arena::api::ArenaBounds;
use crate::combat::api::{Hitbox, Projectile};
use crate::enemies::api::{ChargeTell, EnemyBolt, EnemyKind};
use crate::pickups::api::{PickupKind, Shard, ShardsSpent};
use crate::player::api::Player;
use crate::weapons::api::{Delivery, ShotFired};
use bevy::prelude::*;

const PLAYER_COLOR: Color = Color::srgb(0.0, 0.9, 0.9);
/// Enemies are red to purple; the shade and size tell the kinds apart.
const GRUNT_COLOR: Color = Color::srgb(0.9, 0.15, 0.15);
const SHOOTER_COLOR: Color = Color::srgb(1.0, 0.45, 0.1);
const BRUTE_COLOR: Color = Color::srgb(0.55, 0.05, 0.05);
const CHARGER_COLOR: Color = Color::srgb(1.0, 0.3, 0.55);
const SUMMONER_COLOR: Color = Color::srgb(0.6, 0.2, 0.9);
/// A charger winding up flashes this colour: its tell.
const CHARGER_TELL_COLOR: Color = Color::srgb(1.0, 0.95, 0.95);
const BOLT_COLOR: Color = Color::srgb(1.0, 0.55, 0.2);
const PROJECTILE_COLOR: Color = Color::srgb(1.0, 0.9, 0.1);
const BORDER_COLOR: Color = Color::srgb(0.3, 0.3, 0.32);
const BORDER_THICKNESS: f32 = 4.0;
/// Pickups are green; the shade tells them apart.
const OVERDRIVE_COLOR: Color = Color::srgb(0.2, 1.0, 0.2);
const HEAVY_COLOR: Color = Color::srgb(0.05, 0.5, 0.1);
const MEND_COLOR: Color = Color::srgb(0.7, 1.0, 0.7);
/// Shards and their blast (pickup prototype) are violet.
const SHARD_COLOR: Color = Color::srgb(0.75, 0.35, 1.0);
/// How long a shard blast ring stays on screen.
const BLAST_SECS: f32 = 0.2;
/// How long a hitscan tracer or melee swing stays on screen.
const FLASH_SECS: f32 = 0.08;
/// Line segments used to draw a melee arc.
const ARC_SEGMENTS: u16 = 12;

pub struct DebugRenderPlugin;

impl Plugin for DebugRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Flashes>()
            .add_systems(Startup, spawn_border)
            .add_systems(
                Update,
                (
                    attach_boxes,
                    attach_pickup_boxes,
                    attach_shard_boxes,
                    show_charge_tells,
                    draw_flashes,
                    draw_blasts,
                ),
            );
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
    Or<(
        Added<Player>,
        Added<EnemyKind>,
        Added<Projectile>,
        Added<EnemyBolt>,
    )>,
    Without<Sprite>,
);

fn attach_boxes(
    mut commands: Commands,
    added: Query<
        (
            Entity,
            &Hitbox,
            Has<Player>,
            Option<&EnemyKind>,
            Has<EnemyBolt>,
        ),
        NewVisible,
    >,
) {
    for (entity, hitbox, is_player, kind, is_bolt) in &added {
        let color = match (is_player, kind, is_bolt) {
            (true, ..) => PLAYER_COLOR,
            (_, Some(EnemyKind::Grunt), _) => GRUNT_COLOR,
            (_, Some(EnemyKind::Shooter), _) => SHOOTER_COLOR,
            (_, Some(EnemyKind::Brute), _) => BRUTE_COLOR,
            (_, Some(EnemyKind::Charger), _) => CHARGER_COLOR,
            (_, Some(EnemyKind::Summoner), _) => SUMMONER_COLOR,
            (_, None, true) => BOLT_COLOR,
            (_, None, false) => PROJECTILE_COLOR,
        };
        insert_box(&mut commands, entity, color, hitbox);
    }
}

/// A charger turns pale while it winds up, so the dash can be read.
fn show_charge_tells(mut chargers: Query<(&ChargeTell, &mut Sprite), Changed<ChargeTell>>) {
    for (tell, mut sprite) in &mut chargers {
        sprite.color = if tell.aim.is_some() {
            CHARGER_TELL_COLOR
        } else {
            CHARGER_COLOR
        };
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

fn attach_shard_boxes(
    mut commands: Commands,
    added: Query<(Entity, &Hitbox), (Added<Shard>, Without<Sprite>)>,
) {
    for (entity, hitbox) in &added {
        insert_box(&mut commands, entity, SHARD_COLOR, hitbox);
    }
}

/// A ring for each shard blast: centre, radius, seconds left.
fn draw_blasts(
    time: Res<Time>,
    mut spent: MessageReader<ShardsSpent>,
    mut rings: Local<Vec<(Vec2, f32, f32)>>,
    mut gizmos: Gizmos,
) {
    rings.extend(spent.read().map(|s| (s.at, s.radius, BLAST_SECS)));
    for (at, radius, _) in rings.iter() {
        gizmos.circle_2d(*at, *radius, SHARD_COLOR);
    }
    let dt = time.delta_secs();
    rings.retain_mut(|(_, _, left)| {
        *left -= dt;
        *left > 0.0
    });
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

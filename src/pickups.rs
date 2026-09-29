//! Trade-off pickups. Enemies drop one every few kills; each gives an upside
//! and a downside so taking it is always a decision.

pub mod api;
mod table;

use crate::app_setup::api::SimSet;
use crate::combat::api::{EnemyKilled, HealGranted, Hitbox, PlayerDied};
use crate::player::api::Player;
use crate::waves::api::WaveFailed;
use api::{EffectsChanged, PickupCollected, PickupKind};
use bevy::prelude::*;
use table::{Active, def};

/// PLACEHOLDER: one pickup drops every this many kills.
const KILLS_PER_DROP: u32 = 5;
/// PLACEHOLDER: seconds a dropped pickup stays before vanishing.
const PICKUP_LIFETIME_SECS: f32 = 8.0;
/// PLACEHOLDER: pickup box half-size.
const PICKUP_HALF_SIZE: f32 = 7.0;

pub struct PickupsPlugin;

impl Plugin for PickupsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<EffectsChanged>()
            .add_message::<PickupCollected>()
            .init_resource::<Drops>()
            .init_resource::<ActiveEffects>()
            .add_systems(
                FixedUpdate,
                (drop_on_kills, age_pickups).in_set(SimSet::Spawn),
            )
            .add_systems(FixedUpdate, collect.in_set(SimSet::Detect))
            .add_systems(
                FixedUpdate,
                (clear_on_death, tick_effects)
                    .chain()
                    .in_set(SimSet::Cleanup),
            );
    }
}

#[derive(Resource, Debug, Default)]
struct Drops {
    kills_since_drop: u32,
    /// Index into `PickupKind::ALL`; drops cycle deterministically.
    next_kind: usize,
}

#[derive(Resource, Debug, Default)]
struct ActiveEffects {
    active: Active,
    /// Last value written in `EffectsChanged`.
    published: api::Effects,
}

#[derive(Component, Debug, Clone, Copy)]
struct Lifetime {
    remaining_secs: f32,
}

fn drop_on_kills(
    mut commands: Commands,
    mut killed: MessageReader<EnemyKilled>,
    mut drops: ResMut<Drops>,
) {
    for kill in killed.read() {
        drops.kills_since_drop += 1;
        if drops.kills_since_drop < KILLS_PER_DROP {
            continue;
        }
        drops.kills_since_drop = 0;
        let kind = PickupKind::ALL[drops.next_kind % PickupKind::ALL.len()];
        drops.next_kind = drops.next_kind.wrapping_add(1);
        commands.spawn((
            kind,
            Hitbox::square(PICKUP_HALF_SIZE),
            Lifetime {
                remaining_secs: PICKUP_LIFETIME_SECS,
            },
            Transform::from_translation(kill.at.extend(0.3)),
        ));
    }
}

fn age_pickups(
    mut commands: Commands,
    time: Res<Time>,
    mut pickups: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in &mut pickups {
        lifetime.remaining_secs -= time.delta_secs();
        if lifetime.remaining_secs <= 0.0 {
            commands.entity(entity).try_despawn();
        }
    }
}

fn collect(
    mut commands: Commands,
    player: Option<Single<(Entity, &Transform, &Hitbox), With<Player>>>,
    pickups: Query<(Entity, &Transform, &Hitbox, &PickupKind)>,
    mut effects: ResMut<ActiveEffects>,
    mut heals: MessageWriter<HealGranted>,
    mut collected: MessageWriter<PickupCollected>,
) {
    let Some(player) = player else {
        return;
    };
    let (player_entity, player_transform, player_box) = *player;
    let player_at = player_transform.translation.truncate();
    for (entity, transform, hitbox, kind) in &pickups {
        if !hitbox.overlaps(transform.translation.truncate(), *player_box, player_at) {
            continue;
        }
        commands.entity(entity).try_despawn();
        effects.active.activate(*kind);
        let heal = def(*kind).heal;
        if heal > 0 {
            heals.write(HealGranted {
                target: player_entity,
                amount: heal,
            });
        }
        collected.write(PickupCollected { kind: *kind });
    }
}

/// Death wipes active effects; a failed wave also clears pickups on the floor
/// so the retry starts clean.
fn clear_on_death(
    mut commands: Commands,
    mut died: MessageReader<PlayerDied>,
    mut failed: MessageReader<WaveFailed>,
    mut effects: ResMut<ActiveEffects>,
    pickups: Query<Entity, With<PickupKind>>,
) {
    if died.read().count() > 0 {
        effects.active.clear();
    }
    if failed.read().count() > 0 {
        for entity in &pickups {
            commands.entity(entity).try_despawn();
        }
    }
}

fn tick_effects(
    time: Res<Time>,
    mut effects: ResMut<ActiveEffects>,
    mut changed: MessageWriter<EffectsChanged>,
) {
    effects.active.tick(time.delta_secs());
    let now = effects.active.effects();
    if now != effects.published {
        effects.published = now;
        changed.write(EffectsChanged { effects: now });
    }
}

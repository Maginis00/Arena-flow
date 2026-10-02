//! Trade-off pickups. Enemies drop one every few kills; each gives an upside
//! and a downside so taking it is always a decision.
//!
//! [`PickupRules`] picks the rule set. `Classic` is the default; `Far` and the
//! shard rules are prototypes for making pickups a real decision.

pub mod api;
mod placement;
mod shards;
mod table;

use crate::app_setup::api::SimSet;
use crate::arena::api::ArenaBounds;
use crate::combat::api::{EnemyKilled, HealGranted, Hit, HitSource, Hitbox, PlayerDied};
use crate::enemies::api::Enemy;
use crate::player::api::Player;
use crate::waves::api::{WaveCleared, WaveFailed};
use api::{
    EffectsChanged, PickupCollected, PickupKind, PickupRules, Shard, ShardsChanged, ShardsSpent,
};
use bevy::prelude::*;
use table::{Active, def};

/// PLACEHOLDER: one pickup drops every this many kills.
const KILLS_PER_DROP: u32 = 5;
/// PLACEHOLDER: seconds a dropped pickup stays before vanishing.
const PICKUP_LIFETIME_SECS: f32 = 8.0;
/// PLACEHOLDER: pickup box half-size.
const PICKUP_HALF_SIZE: f32 = 7.0;

/// `rules` defaults to `ARENA_PICKUPS` (unset: classic).
pub struct PickupsPlugin {
    pub rules: PickupRules,
}

impl Default for PickupsPlugin {
    fn default() -> Self {
        Self {
            rules: PickupRules::from_env(),
        }
    }
}

impl Plugin for PickupsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<EffectsChanged>()
            .add_message::<PickupCollected>()
            .add_message::<ShardsChanged>()
            .add_message::<ShardsSpent>()
            .insert_resource(self.rules)
            .init_resource::<Drops>()
            .init_resource::<ActiveEffects>()
            .init_resource::<SpendInput>()
            .add_systems(Update, sample_spend)
            .add_systems(
                FixedUpdate,
                (drop_on_kills, age_pickups).in_set(SimSet::Spawn),
            )
            .add_systems(
                FixedUpdate,
                (collect, collect_shards, spend_shards)
                    .chain()
                    .in_set(SimSet::Detect),
            )
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
    /// Shards the player holds (shard rules only).
    held_shards: u32,
    /// Last value written in `EffectsChanged`.
    published: api::Effects,
}

/// Space was pressed: spend the held shards. Sampled in `Update`.
#[derive(Resource, Debug, Default)]
struct SpendInput {
    requested: bool,
}

#[derive(Component, Debug, Clone, Copy)]
struct Lifetime {
    remaining_secs: f32,
}

fn drop_on_kills(
    mut commands: Commands,
    rules: Res<PickupRules>,
    bounds: Res<ArenaBounds>,
    player: Option<Single<&Transform, With<Player>>>,
    mut killed: MessageReader<EnemyKilled>,
    mut drops: ResMut<Drops>,
) {
    let player_at = player.map(|t| t.translation.truncate());
    for kill in killed.read() {
        if *rules == PickupRules::ShardsRange
            && player_at.is_some_and(|p| p.distance(kill.at) < shards::RANGE_MIN_KILL_DISTANCE)
        {
            continue;
        }
        drops.kills_since_drop += 1;
        let every = if rules.uses_shards() {
            shards::KILLS_PER_SHARD
        } else {
            KILLS_PER_DROP
        };
        if drops.kills_since_drop < every {
            continue;
        }
        drops.kills_since_drop = 0;
        if rules.uses_shards() {
            commands.spawn((
                Shard,
                Hitbox::square(shards::SHARD_HALF_SIZE),
                Lifetime {
                    remaining_secs: shards::SHARD_LIFETIME_SECS,
                },
                Transform::from_translation(kill.at.extend(0.3)),
            ));
            continue;
        }
        let kind = PickupKind::ALL[drops.next_kind % PickupKind::ALL.len()];
        drops.next_kind = drops.next_kind.wrapping_add(1);
        let (at, lifetime) = match (*rules, player_at) {
            (PickupRules::Far, Some(p)) => (
                placement::thrown(kill.at, p, bounds.half_extents()),
                placement::FAR_LIFETIME_SECS,
            ),
            _ => (kill.at, PICKUP_LIFETIME_SECS),
        };
        commands.spawn((
            kind,
            Hitbox::square(PICKUP_HALF_SIZE),
            Lifetime {
                remaining_secs: lifetime,
            },
            Transform::from_translation(at.extend(0.3)),
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

fn collect_shards(
    mut commands: Commands,
    player: Option<Single<(&Transform, &Hitbox), With<Player>>>,
    floor: Query<(Entity, &Transform, &Hitbox), With<Shard>>,
    mut effects: ResMut<ActiveEffects>,
    mut changed: MessageWriter<ShardsChanged>,
) {
    let Some(player) = player else {
        return;
    };
    let (player_transform, player_box) = *player;
    let player_at = player_transform.translation.truncate();
    let before = effects.held_shards;
    for (entity, transform, hitbox) in &floor {
        if effects.held_shards >= shards::MAX_HELD
            || !hitbox.overlaps(transform.translation.truncate(), *player_box, player_at)
        {
            continue;
        }
        commands.entity(entity).try_despawn();
        effects.held_shards += 1;
    }
    if effects.held_shards != before {
        changed.write(ShardsChanged {
            held: effects.held_shards,
        });
    }
}

fn sample_spend(keys: Res<ButtonInput<KeyCode>>, mut input: ResMut<SpendInput>) {
    if keys.just_pressed(KeyCode::Space) {
        input.requested = true;
    }
}

/// Spend every held shard on a blast around the player, if there are enough.
#[allow(clippy::too_many_arguments)] // one reader or writer per fact
fn spend_shards(
    mut input: ResMut<SpendInput>,
    rules: Res<PickupRules>,
    player: Option<Single<&Transform, With<Player>>>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    mut effects: ResMut<ActiveEffects>,
    mut hits: MessageWriter<Hit>,
    mut changed: MessageWriter<ShardsChanged>,
    mut spent: MessageWriter<ShardsSpent>,
) {
    if !std::mem::take(&mut input.requested) || !rules.uses_shards() {
        return;
    }
    let Some(player) = player else {
        return;
    };
    let count = effects.held_shards;
    let Some(radius) = shards::blast_radius(count) else {
        return;
    };
    let at = player.translation.truncate();
    for (enemy, transform) in &enemies {
        if transform.translation.truncate().distance(at) <= radius {
            hits.write(Hit {
                target: enemy,
                damage: shards::BLAST_DAMAGE,
                source: HitSource::Blast,
            });
        }
    }
    effects.held_shards = 0;
    changed.write(ShardsChanged { held: 0 });
    spent.write(ShardsSpent { count, at, radius });
}

/// Death wipes active effects and held shards; a failed wave also clears
/// pickups on the floor so the retry starts clean. Under `Far` rules a
/// cleared wave does the same, so nothing carries into the next wave.
#[allow(clippy::too_many_arguments)] // one reader or writer per fact
fn clear_on_death(
    mut commands: Commands,
    rules: Res<PickupRules>,
    mut died: MessageReader<PlayerDied>,
    mut failed: MessageReader<WaveFailed>,
    mut cleared: MessageReader<WaveCleared>,
    mut effects: ResMut<ActiveEffects>,
    floor: Query<Entity, Or<(With<PickupKind>, With<Shard>)>>,
    mut changed: MessageWriter<ShardsChanged>,
) {
    let wave_ended_clean = cleared.read().count() > 0 && *rules == PickupRules::Far;
    if died.read().count() > 0 || wave_ended_clean {
        effects.active.clear();
        if effects.held_shards > 0 {
            effects.held_shards = 0;
            changed.write(ShardsChanged { held: 0 });
        }
    }
    if failed.read().count() > 0 || wave_ended_clean {
        for entity in &floor {
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
    let now = effects
        .active
        .effects()
        .combine(shards::held_effects(effects.held_shards));
    if now != effects.published {
        effects.published = now;
        changed.write(EffectsChanged { effects: now });
    }
}

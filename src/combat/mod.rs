//! Hits, health, damage application.
//!
//! Detects projectile overlaps, applies every [`Hit`](api::Hit) (from
//! projectiles or enemy contact), and reports the outcome as facts.

pub mod api;

use crate::app_setup::api::SimSet;
use api::{
    EnemyKilled, Health, Hit, HitSource, Hitbox, PlayerDamaged, PlayerDied, Projectile, Team,
};
use bevy::prelude::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<Hit>()
            .add_message::<PlayerDamaged>()
            .add_message::<PlayerDied>()
            .add_message::<EnemyKilled>()
            .add_systems(FixedUpdate, detect_projectile_hits.in_set(SimSet::Detect))
            .add_systems(FixedUpdate, apply_hits.in_set(SimSet::Resolve));
    }
}

/// Each projectile hits at most one living target of another team per tick.
fn detect_projectile_hits(
    projectiles: Query<(Entity, &Transform, &Hitbox, &Projectile)>,
    targets: Query<(Entity, &Transform, &Hitbox, &Health, &Team)>,
    mut hits: MessageWriter<Hit>,
) {
    for (projectile, p_transform, p_box, shot) in &projectiles {
        let p_at = p_transform.translation.truncate();
        let struck = targets.iter().find(|(_, t, hitbox, health, team)| {
            **team != shot.team
                && !health.is_dead()
                && p_box.overlaps(p_at, **hitbox, t.translation.truncate())
        });
        if let Some((target, ..)) = struck {
            hits.write(Hit {
                target,
                damage: shot.damage,
                source: HitSource::Projectile(projectile),
            });
        }
    }
}

fn apply_hits(
    mut commands: Commands,
    mut hits: MessageReader<Hit>,
    mut targets: Query<(&mut Health, &Team)>,
    mut player_damaged: MessageWriter<PlayerDamaged>,
    mut player_died: MessageWriter<PlayerDied>,
    mut enemy_killed: MessageWriter<EnemyKilled>,
) {
    for hit in hits.read() {
        if let HitSource::Projectile(projectile) = hit.source {
            commands.entity(projectile).try_despawn();
        }
        // The target may already be gone (killed earlier this tick, wave ended).
        let Ok((mut health, team)) = targets.get_mut(hit.target) else {
            continue;
        };
        if health.is_dead() {
            continue;
        }
        let applied = health.take(hit.damage);
        match team {
            Team::Player => {
                if applied > 0 {
                    player_damaged.write(PlayerDamaged {
                        amount: applied,
                        remaining: health.current(),
                        max: health.max(),
                    });
                }
                if health.is_dead() {
                    // The player plugin owns the player's lifecycle and despawns it.
                    player_died.write(PlayerDied { player: hit.target });
                }
            }
            Team::Enemy => {
                if health.is_dead() {
                    enemy_killed.write(EnemyKilled { enemy: hit.target });
                    commands.entity(hit.target).try_despawn();
                }
            }
        }
    }
}

//! Hits, health, damage application.
//!
//! Resolves every weapon shot into [`Hit`]s (projectile overlaps,
//! hitscan rays, melee arcs), applies all hits and heals, and reports the
//! outcome as facts.

mod health;
mod hits;

pub use health::{Health, Hitbox, Team};
pub use hits::{
    EnemyKilled, HealGranted, Hit, HitSource, PlayerDamaged, PlayerDied, PlayerHealed, Projectile,
    ShotId,
};

use crate::app_setup::SimSet;
use crate::pickups::{Effects, EffectsChanged, scale_damage};
use crate::weapons::{Delivery, ShotFired};
use bevy::prelude::*;

/// PLACEHOLDER: seconds after taking damage in which the player can't be hurt
/// again. A crowd then costs a few hits and a chance to break out instead of
/// the whole bar at once, so a wave can end half-way to dying.
const GRACE_SECS: f32 = 0.75;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<Hit>()
            .add_message::<PlayerDamaged>()
            .add_message::<PlayerDied>()
            .add_message::<EnemyKilled>()
            .add_message::<HealGranted>()
            .add_message::<PlayerHealed>()
            .init_resource::<IncomingEffects>()
            .init_resource::<Grace>()
            .add_systems(FixedUpdate, track_effects.in_set(SimSet::Intent))
            .add_systems(
                FixedUpdate,
                (detect_projectile_hits, resolve_instant_shots).in_set(SimSet::Detect),
            )
            .add_systems(
                FixedUpdate,
                (apply_heals, apply_hits).chain().in_set(SimSet::Resolve),
            );
    }
}

/// Time left on the player's grace period after the last damage taken.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
struct Grace {
    remaining_secs: f32,
}

impl Grace {
    fn tick(&mut self, dt: f32) {
        self.remaining_secs = (self.remaining_secs - dt).max(0.0);
    }

    fn protects(self) -> bool {
        self.remaining_secs > 0.0
    }

    fn start(&mut self) {
        self.remaining_secs = GRACE_SECS;
    }
}

/// Latest pickup effects; combat only uses `incoming_damage`.
#[derive(Resource, Debug, Default, Clone, Copy)]
struct IncomingEffects(Effects);

type Targets<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Transform,
        &'static Hitbox,
        &'static Health,
        &'static Team,
    ),
>;

fn track_effects(mut changed: MessageReader<EffectsChanged>, mut effects: ResMut<IncomingEffects>) {
    if let Some(latest) = changed.read().last() {
        effects.0 = latest.effects;
    }
}

/// Each projectile hits at most one living target of another team per tick.
fn detect_projectile_hits(
    projectiles: Query<(Entity, &Transform, &Hitbox, &Projectile)>,
    targets: Targets,
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
                source: HitSource::Shot {
                    shot: shot.shot,
                    projectile: Some(projectile),
                },
            });
        }
    }
}

/// Hitscan: first target along the ray. Melee: every target inside the arc.
fn resolve_instant_shots(
    mut shots: MessageReader<ShotFired>,
    targets: Targets,
    mut hits: MessageWriter<Hit>,
) {
    for fired in shots.read() {
        let source = HitSource::Shot {
            shot: fired.shot,
            projectile: None,
        };
        let hostile = targets
            .iter()
            .filter(|(.., health, team)| **team != fired.team && !health.is_dead());
        match fired.delivery {
            Delivery::Projectile { .. } => {}
            Delivery::Hitscan { range, damage } => {
                let nearest = hostile
                    .filter_map(|(entity, t, hitbox, ..)| {
                        hitbox
                            .ray_distance(
                                t.translation.truncate(),
                                fired.origin,
                                fired.direction,
                                range,
                            )
                            .map(|d| (entity, d))
                    })
                    .min_by(|a, b| a.1.total_cmp(&b.1));
                if let Some((target, _)) = nearest {
                    hits.write(Hit {
                        target,
                        damage,
                        source,
                    });
                }
            }
            Delivery::Melee {
                radius,
                half_angle,
                damage,
            } => {
                for (target, t, hitbox, ..) in hostile {
                    let offset = t.translation.truncate() - fired.origin;
                    // Approximate the target box by its bounding circle.
                    let reach = radius + hitbox.half_extents.length();
                    let in_reach = offset.length_squared() <= reach * reach;
                    let in_arc = offset.length_squared() <= f32::EPSILON
                        || fired.direction.angle_to(offset).abs() <= half_angle;
                    if in_reach && in_arc {
                        hits.write(Hit {
                            target,
                            damage,
                            source,
                        });
                    }
                }
            }
        }
    }
}

fn apply_heals(
    mut heals: MessageReader<HealGranted>,
    mut targets: Query<(&mut Health, &Team)>,
    mut healed: MessageWriter<PlayerHealed>,
) {
    for heal in heals.read() {
        let Ok((mut health, team)) = targets.get_mut(heal.target) else {
            continue;
        };
        if health.is_dead() {
            continue;
        }
        let applied = health.restore(heal.amount);
        if applied > 0 && *team == Team::Player {
            healed.write(PlayerHealed {
                amount: applied,
                remaining: health.current(),
                max: health.max(),
            });
        }
    }
}

#[allow(clippy::too_many_arguments)] // one writer per resolved fact
fn apply_hits(
    mut commands: Commands,
    time: Res<Time>,
    mut grace: ResMut<Grace>,
    effects: Res<IncomingEffects>,
    mut hits: MessageReader<Hit>,
    mut targets: Query<(&mut Health, &Team, &Transform)>,
    mut player_damaged: MessageWriter<PlayerDamaged>,
    mut player_died: MessageWriter<PlayerDied>,
    mut enemy_killed: MessageWriter<EnemyKilled>,
) {
    grace.tick(time.delta_secs());
    for hit in hits.read() {
        if let HitSource::Shot {
            projectile: Some(projectile),
            ..
        } = hit.source
        {
            commands.entity(projectile).try_despawn();
        }
        // The target may already be gone (killed earlier this tick, wave ended).
        let Ok((mut health, team, transform)) = targets.get_mut(hit.target) else {
            continue;
        };
        if health.is_dead() {
            continue;
        }
        match team {
            Team::Player => {
                // Also stops several hits in the same tick from stacking.
                if grace.protects() {
                    continue;
                }
                let applied = health.take(scale_damage(hit.damage, effects.0.incoming_damage));
                if applied > 0 {
                    grace.start();
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
                health.take(hit.damage);
                if health.is_dead() {
                    enemy_killed.write(EnemyKilled {
                        enemy: hit.target,
                        at: transform.translation.truncate(),
                    });
                    commands.entity(hit.target).try_despawn();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grace_protects_until_it_runs_out() {
        let mut grace = Grace::default();
        assert!(!grace.protects());
        grace.start();
        assert!(grace.protects());
        grace.tick(GRACE_SECS - 0.01);
        assert!(grace.protects());
        grace.tick(0.02);
        assert!(!grace.protects());
        grace.tick(1.0);
        assert_eq!(grace.remaining_secs, 0.0);
    }
}

//! How near enemies come to the player during the wave in play. Every tick
//! reads positions and contact hits; [`Danger`] in the report sums it up.

use super::api::Danger;
use super::machine::{Approach, WaveMachine};
use crate::combat::api::{Hit, HitSource, Hitbox};
use crate::enemies::api::{Enemy, EnemyBolt};
use crate::player::api::Player;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

/// PLACEHOLDER: an enemy whose box comes this close to the player's box is
/// on a close call.
const CLOSE_CALL_GAP: f32 = 24.0;
/// PLACEHOLDER: the approach is over once the enemy is this far again (or gone).
const CLOSE_CALL_EXIT_GAP: f32 = 40.0;
/// PLACEHOLDER: under threat while some enemy is less than this from contact.
const THREAT_SECS: f32 = 0.4;
/// PLACEHOLDER: crowding counts the enemies less than this from contact.
const CROWD_SECS: f32 = 1.0;
/// PLACEHOLDER: calm while no enemy is less than this from contact.
const CALM_SECS: f32 = 2.0;

pub(super) fn measure_danger(
    time: Res<Time>,
    mut machine: ResMut<WaveMachine>,
    mut hits: MessageReader<Hit>,
    player: Option<Single<(&Transform, &Hitbox), With<Player>>>,
    // Enemy bolts count like enemies: a bolt that grazes past is a close call.
    enemies: Query<(Entity, &Transform, &Hitbox), Or<(With<Enemy>, With<EnemyBolt>)>>,
) {
    let contacts: Vec<Entity> = hits
        .read()
        .filter_map(|hit| match hit.source {
            HitSource::Contact(enemy) | HitSource::EnemyShot(enemy) => Some(enemy),
            HitSource::Shot { .. } => None,
        })
        .collect();
    if !machine.in_play() {
        return;
    }
    let (Some(spec), Some(player)) = (machine.current, player) else {
        return;
    };
    let (player_transform, player_box) = *player;
    let at = player_transform.translation.truncate();
    let gaps: Vec<(Entity, f32)> = enemies
        .iter()
        .map(|(enemy, transform, hitbox)| {
            let gap = gap(at, *player_box, transform.translation.truncate(), *hitbox);
            (enemy, gap)
        })
        .collect();
    let stats = &mut machine.stats;
    tick(
        &mut stats.danger,
        &mut stats.approaches,
        &gaps,
        &contacts,
        spec.enemy_speed,
        time.delta_secs(),
    );
}

/// Space between two boxes along the axis where they are furthest apart;
/// zero once they touch.
fn gap(a: Vec2, a_box: Hitbox, b: Vec2, b_box: Hitbox) -> f32 {
    let apart = (a - b).abs() - (a_box.half_extents + b_box.half_extents);
    apart.max_element().max(0.0)
}

/// One tick: `gaps` is every live enemy's gap to the player, `contacts` the
/// enemies that landed a contact hit this tick.
fn tick(
    danger: &mut Danger,
    approaches: &mut HashMap<Entity, Approach>,
    gaps: &[(Entity, f32)],
    contacts: &[Entity],
    enemy_speed: f32,
    dt: f32,
) {
    for &(enemy, gap) in gaps.iter().filter(|(_, gap)| *gap <= CLOSE_CALL_GAP) {
        let approach = approaches.entry(enemy).or_insert(Approach {
            nearest_gap: gap,
            hit: false,
        });
        approach.nearest_gap = approach.nearest_gap.min(gap);
    }
    for &enemy in contacts {
        let approach = approaches.entry(enemy).or_insert(Approach {
            nearest_gap: 0.0,
            hit: false,
        });
        approach.hit = true;
    }
    approaches.retain(|enemy, approach| {
        let still_close = gaps
            .iter()
            .any(|(e, gap)| e == enemy && *gap <= CLOSE_CALL_EXIT_GAP);
        if still_close {
            return true;
        }
        if approach.hit {
            danger.hit_approaches += 1;
        } else {
            danger.close_calls += 1;
            danger.close_call_weight += 1.0 - approach.nearest_gap / CLOSE_CALL_GAP;
        }
        false
    });

    let speed = enemy_speed.max(1.0);
    let secs_to_contact = |gap: f32| gap / speed;
    let nearest = gaps
        .iter()
        .map(|(_, gap)| secs_to_contact(*gap))
        .fold(f32::INFINITY, f32::min);
    if nearest < THREAT_SECS {
        danger.threat_secs += dt;
    }
    if nearest >= CALM_SECS {
        danger.calm_secs += dt;
    } else {
        danger.closest_secs = Some(danger.closest_secs.map_or(nearest, |c| c.min(nearest)));
    }
    let crowd = gaps
        .iter()
        .filter(|(_, gap)| secs_to_contact(*gap) < CROWD_SECS)
        .count();
    danger.peak_crowd = danger
        .peak_crowd
        .max(u32::try_from(crowd).unwrap_or(u32::MAX));
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    fn enemy(n: u32) -> Entity {
        Entity::from_raw_u32(n).expect("valid test entity")
    }

    struct Run {
        danger: Danger,
        approaches: HashMap<Entity, Approach>,
    }

    impl Run {
        fn new() -> Self {
            Self {
                danger: Danger::default(),
                approaches: HashMap::default(),
            }
        }

        fn tick(&mut self, gaps: &[(Entity, f32)], contacts: &[Entity]) {
            tick(
                &mut self.danger,
                &mut self.approaches,
                gaps,
                contacts,
                100.0,
                DT,
            );
        }
    }

    #[test]
    fn gap_is_zero_when_boxes_touch() {
        let b = Hitbox::square(10.0);
        assert_eq!(gap(Vec2::ZERO, b, Vec2::new(15.0, 0.0), b), 0.0);
        assert_eq!(gap(Vec2::ZERO, b, Vec2::new(50.0, 5.0), b), 30.0);
        assert_eq!(gap(Vec2::ZERO, b, Vec2::new(-30.0, 60.0), b), 40.0);
    }

    #[test]
    fn an_enemy_that_comes_close_and_leaves_is_a_close_call() {
        let mut run = Run::new();
        let e = enemy(1);
        run.tick(&[(e, 100.0)], &[]);
        run.tick(&[(e, 12.0)], &[]);
        run.tick(&[(e, 30.0)], &[]);
        assert_eq!(run.danger.close_calls, 0, "still inside the exit gap");
        run.tick(&[(e, 60.0)], &[]);
        assert_eq!(run.danger.close_calls, 1);
        assert!((run.danger.close_call_weight - 0.5).abs() < 1e-6);
    }

    #[test]
    fn an_enemy_killed_while_close_is_a_close_call() {
        let mut run = Run::new();
        let e = enemy(1);
        run.tick(&[(e, 0.0)], &[]);
        run.tick(&[], &[]);
        assert_eq!(run.danger.close_calls, 1);
        assert_eq!(run.danger.close_call_weight, 1.0);
    }

    #[test]
    fn an_approach_that_lands_a_hit_is_not_a_close_call() {
        let mut run = Run::new();
        let e = enemy(1);
        run.tick(&[(e, 5.0)], &[]);
        run.tick(&[(e, 0.0)], &[e]);
        run.tick(&[(e, 0.0)], &[e]);
        run.tick(&[], &[]);
        assert_eq!(run.danger.close_calls, 0);
        assert_eq!(run.danger.hit_approaches, 1);
    }

    #[test]
    fn time_is_split_by_the_nearest_enemy() {
        let mut run = Run::new();
        let (a, b) = (enemy(1), enemy(2));
        // Speed 100: 30 is 0.3 s from contact, 500 is 5 s.
        run.tick(&[(a, 30.0), (b, 500.0)], &[]);
        run.tick(&[(a, 500.0), (b, 500.0)], &[]);
        run.tick(&[], &[]);
        assert!((run.danger.threat_secs - DT).abs() < 1e-6);
        assert!((run.danger.calm_secs - 2.0 * DT).abs() < 1e-6);
        assert_eq!(run.danger.closest_secs, Some(0.3));
    }

    #[test]
    fn closest_stays_unknown_while_everything_is_far() {
        let mut run = Run::new();
        run.tick(&[(enemy(1), 900.0)], &[]);
        assert_eq!(run.danger.closest_secs, None);
    }

    #[test]
    fn peak_crowd_counts_enemies_within_a_second() {
        let mut run = Run::new();
        let gaps: Vec<_> = (1..=5).map(|n| (enemy(n), n as f32 * 30.0)).collect();
        run.tick(&gaps, &[]);
        assert_eq!(run.danger.peak_crowd, 3);
        run.tick(&[], &[]);
        assert_eq!(run.danger.peak_crowd, 3);
    }
}

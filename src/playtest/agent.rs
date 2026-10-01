//! An outside agent plays the real game in steps. The game is deterministic,
//! so a run is just its list of orders: [`replay`] plays them from the start
//! in a headless app and returns what the agent sees after the last one.
//! The agent decides between steps, while the simulation is not running; its
//! "hands" carry each order out with one bot tier's reaction time and aim.

use super::agent_order::{Aim, Move, Order};
use super::agent_view::{FinishedWave, Frame, StepEvents};
use super::choices::reach;
use super::perception::{DelayedView, Rng, Snapshot};
use super::steering::{eight_way, nearest, rotate};
use super::tier::{SkillTier, TierParams};
use crate::FlowArenaPlugins;
use crate::app_setup::api::{FIXED_HZ, SimSet};
use crate::arena::api::ArenaBounds;
use crate::combat::api::{EnemyKilled, Health, PlayerDamaged, PlayerDied, PlayerHealed, Team};
use crate::debug_render::DebugRenderPlugin;
use crate::enemies::api::{Enemy, EnemySpawned};
use crate::flow_director::api::DifficultyAdjusted;
use crate::pickups::api::{Effects, EffectsChanged, PickupCollected, PickupKind};
use crate::player::api::{FireRequested, Player};
use crate::telemetry::api::SessionRecord;
use crate::waves::api::{WaveReport, WaveSpec, WaveStarted};
use crate::weapons::api::{ShotFired, WeaponKind, WeaponSwitched};
use bevy::ecs::schedule::SingleThreadedExecutor;
use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use std::time::Duration;

/// PLACEHOLDER: `fire densest` counts neighbours this close to a target.
const DENSE_RADIUS: f32 = 100.0;
/// `to X Y` stops this close to the point.
const ARRIVE_RADIUS: f32 = 8.0;
/// Fixed seed for the aim wobble, so a run replays exactly.
const HAND_SEED: u32 = 1;

/// A finished replay: what the agent sees now, plus telemetry's record of the
/// whole run for the end-of-run summary.
pub struct AgentRun {
    pub frame: Frame,
    pub record: SessionRecord,
}

/// Play `earlier` orders silently, then `latest` while recording what
/// happens; the returned frame describes only the `latest` part. `hands`
/// sets the reaction time and aim error the orders are carried out with.
pub fn replay(hands: SkillTier, earlier: &[Order], latest: &[Order]) -> AgentRun {
    let orders: Vec<Order> = earlier.iter().chain(latest).copied().collect();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin))
        .add_plugins(FlowArenaPlugins.build().disable::<DebugRenderPlugin>())
        .insert_resource(Hands::new(hands.params(), orders))
        .init_resource::<Watched>()
        .add_systems(FixedUpdate, act.in_set(SimSet::Intent))
        .add_systems(Update, watch)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / FIXED_HZ,
        )));
    // One thread keeps message order, and so every replay, identical.
    app.edit_schedule(FixedUpdate, |schedule| {
        schedule.set_executor(SingleThreadedExecutor::new());
    })
    .edit_schedule(Update, |schedule| {
        schedule.set_executor(SingleThreadedExecutor::new());
    });

    // One app update is one simulation tick at this clock; the first update
    // runs Startup.
    let earlier_ticks: u64 = earlier.iter().map(|o| ticks(o.secs) as u64).sum();
    let latest_ticks: u64 = latest.iter().map(|o| ticks(o.secs) as u64).sum();
    app.update();
    for _ in 0..earlier_ticks {
        app.update();
    }
    {
        let mut watched = app.world_mut().resource_mut::<Watched>();
        watched.step = StepEvents::default();
        watched.finished.clear();
    }
    for _ in 0..latest_ticks {
        app.update();
    }
    let secs = (earlier_ticks + latest_ticks) as f32 / FIXED_HZ as f32;
    let frame = frame(app.world_mut(), secs);
    let record = app
        .world_mut()
        .remove_resource::<SessionRecord>()
        .unwrap_or_default();
    AgentRun { frame, record }
}

fn ticks(secs: f32) -> u32 {
    // Orders are bounded to a few seconds when parsed.
    ((secs * FIXED_HZ as f32).round() as u32).max(1)
}

#[derive(Resource, Debug)]
struct Hands {
    orders: Vec<Order>,
    index: usize,
    ticks_left: u32,
    started: bool,
    params: TierParams,
    view: DelayedView,
    rng: Rng,
    until_wobble_secs: f32,
    aim_offset_rad: f32,
}

impl Hands {
    fn new(params: TierParams, orders: Vec<Order>) -> Self {
        // Rounding a positive, small reaction time to whole ticks.
        let delay = (params.reaction_secs * FIXED_HZ as f32).round().max(0.0) as usize;
        let ticks_left = orders.first().map_or(0, |o| ticks(o.secs));
        Self {
            orders,
            index: 0,
            ticks_left,
            started: false,
            params,
            view: DelayedView::new(delay),
            rng: Rng::new(HAND_SEED),
            until_wobble_secs: 0.0,
            aim_offset_rad: 0.0,
        }
    }
}

/// Everything the agent is told about, gathered from the game's messages.
#[derive(Resource, Debug, Default)]
struct Watched {
    wave: Option<WaveSpec>,
    in_wave: bool,
    spawned: u32,
    weapon: WeaponKind,
    effects: Effects,
    step: StepEvents,
    finished: Vec<FinishedWave>,
}

#[allow(clippy::too_many_arguments)] // what a player's hands and eyes touch
fn act(
    time: Res<Time>,
    bounds: Res<ArenaBounds>,
    mut hands: ResMut<Hands>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    player: Query<(Entity, &Transform, &Health), With<Player>>,
    enemies: Query<&Transform, With<Enemy>>,
    pickups: Query<(&Transform, &PickupKind)>,
    watched: Res<Watched>,
    mut fire: MessageWriter<FireRequested>,
) {
    let hands = &mut *hands;
    hands.view.push(Snapshot {
        enemies: enemies.iter().map(|t| t.translation.truncate()).collect(),
        pickups: pickups
            .iter()
            .map(|(t, kind)| (t.translation.truncate(), *kind))
            .collect(),
    });
    let Some(order) = hands.orders.get(hands.index).copied() else {
        hold_keys(&mut keys, (0, 0));
        return;
    };
    if !hands.started {
        hands.started = true;
        if let Some(weapon) = order.weapon {
            let key = weapon_key(weapon);
            keys.release(key);
            keys.press(key);
        }
    }
    // The hand drifts off target as often as the tier's bot re-aims.
    hands.until_wobble_secs -= time.delta_secs();
    if hands.until_wobble_secs <= 0.0 {
        hands.until_wobble_secs += hands.params.decision_secs;
        hands.aim_offset_rad = hands.rng.wobble() * hands.params.aim_error_deg.to_radians();
    }
    hands.ticks_left = hands.ticks_left.saturating_sub(1);
    if hands.ticks_left == 0 {
        hands.index += 1;
        hands.started = false;
        hands.ticks_left = hands.orders.get(hands.index).map_or(0, |o| ticks(o.secs));
    }

    let alive = player.iter().find(|(.., health)| !health.is_dead());
    let Some((shooter, transform, _)) = alive else {
        hold_keys(&mut keys, (0, 0));
        return;
    };
    let own = transform.translation.truncate();
    let wish = match order.movement {
        Move::Keys(x, y) => Vec2::new(f32::from(x), f32::from(y)),
        Move::To(at) => {
            let at = at.clamp(-bounds.half_extents(), bounds.half_extents());
            if at.distance(own) > ARRIVE_RADIUS {
                at - own
            } else {
                Vec2::ZERO
            }
        }
    };
    hold_keys(&mut keys, eight_way(wish));

    let Some(seen) = hands.view.seen() else {
        return;
    };
    let direction = match order.aim {
        Aim::Hold => return,
        Aim::Angle(deg) => Vec2::from_angle(deg.to_radians()),
        Aim::Nearest | Aim::Densest => {
            let target = if order.aim == Aim::Nearest {
                nearest(own, seen.enemies.iter().copied())
            } else {
                densest(&seen.enemies)
            };
            let Some(target) = target else {
                return;
            };
            if target.distance(own) > reach(watched.weapon) {
                return;
            }
            rotate((target - own).normalize_or_zero(), hands.aim_offset_rad)
        }
    };
    if direction != Vec2::ZERO {
        fire.write(FireRequested {
            shooter,
            origin: own,
            direction,
        });
    }
}

/// The enemy with the most others within [`DENSE_RADIUS`].
fn densest(enemies: &[Vec2]) -> Option<Vec2> {
    enemies.iter().copied().max_by_key(|e| {
        enemies
            .iter()
            .filter(|o| o.distance(*e) < DENSE_RADIUS)
            .count()
    })
}

#[allow(clippy::too_many_arguments)] // one reader per observed fact
fn watch(
    mut watched: ResMut<Watched>,
    mut started: MessageReader<WaveStarted>,
    mut spawned: MessageReader<EnemySpawned>,
    mut reports: MessageReader<WaveReport>,
    mut adjusted: MessageReader<DifficultyAdjusted>,
    mut damaged: MessageReader<PlayerDamaged>,
    mut healed: MessageReader<PlayerHealed>,
    mut died: MessageReader<PlayerDied>,
    mut shots: MessageReader<ShotFired>,
    mut killed: MessageReader<EnemyKilled>,
    mut collected: MessageReader<PickupCollected>,
    mut switched: MessageReader<WeaponSwitched>,
    mut effects: MessageReader<EffectsChanged>,
) {
    let w = &mut *watched;
    for m in started.read() {
        w.wave = Some(m.spec);
        w.in_wave = true;
        w.spawned = 0;
    }
    w.spawned += spawned.read().count() as u32;
    for m in reports.read() {
        w.in_wave = false;
        w.finished.push(FinishedWave {
            report: *m,
            next: None,
        });
    }
    for m in adjusted.read() {
        let unanswered = w.finished.iter_mut().rev().find(|f| f.next.is_none());
        if let (Some(finished), Some(_)) = (unanswered, m.risk) {
            finished.next = Some((m.difficulty.level(), m.reason.to_string()));
        }
    }
    for m in damaged.read() {
        w.step.damage_taken += m.amount;
        w.step.hits_taken += 1;
    }
    for m in healed.read() {
        w.step.healed += m.amount;
    }
    w.step.died |= died.read().count() > 0;
    w.step.shots += shots.read().filter(|s| s.team == Team::Player).count() as u32;
    w.step.kills += killed.read().count() as u32;
    w.step.pickups.extend(collected.read().map(|m| m.kind));
    if let Some(m) = switched.read().last() {
        w.weapon = m.weapon;
    }
    if let Some(m) = effects.read().last() {
        w.effects = m.effects;
    }
}

fn frame(world: &mut World, secs: f32) -> Frame {
    let half_extents = world.resource::<ArenaBounds>().half_extents();
    let player = world
        .query_filtered::<(&Transform, &Health), With<Player>>()
        .iter(world)
        .find(|(_, health)| !health.is_dead())
        .map(|(t, h)| (t.translation.truncate(), h.current(), h.max()));
    let enemies = world
        .query_filtered::<&Transform, With<Enemy>>()
        .iter(world)
        .map(|t| t.translation.truncate())
        .collect();
    let pickups = world
        .query::<(&Transform, &PickupKind)>()
        .iter(world)
        .map(|(t, kind)| (t.translation.truncate(), *kind))
        .collect();
    let w = world.resource::<Watched>();
    let still_to_spawn = match (w.wave, w.in_wave) {
        (Some(spec), true) => spec.enemy_count.saturating_sub(w.spawned),
        _ => 0,
    };
    Frame {
        secs,
        half_extents,
        wave: w.wave,
        in_wave: w.in_wave,
        still_to_spawn,
        player,
        weapon: w.weapon,
        effects: w.effects,
        enemies,
        pickups,
        step: w.step.clone(),
        finished: w.finished.clone(),
    }
}

fn hold_keys(keys: &mut ButtonInput<KeyCode>, (x, y): (i8, i8)) {
    for (key, held) in [
        (KeyCode::KeyD, x > 0),
        (KeyCode::KeyA, x < 0),
        (KeyCode::KeyW, y > 0),
        (KeyCode::KeyS, y < 0),
    ] {
        if held {
            keys.press(key);
        } else {
            keys.release(key);
        }
    }
}

const fn weapon_key(weapon: WeaponKind) -> KeyCode {
    match weapon {
        WeaponKind::Projectile => KeyCode::Digit1,
        WeaponKind::Hitscan => KeyCode::Digit2,
        WeaponKind::Melee => KeyCode::Digit3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn orders(lines: &[&str]) -> Vec<Order> {
        lines
            .iter()
            .map(|l| l.parse().expect("valid order"))
            .collect()
    }

    #[test]
    fn a_replay_is_identical_every_time() {
        let earlier = orders(&["e 2 for 1", "n for 1", "w 3 fire densest for 1"]);
        let latest = orders(&["s 1 for 1"]);
        let a = replay(SkillTier::Skilled, &earlier, &latest).frame;
        let b = replay(SkillTier::Skilled, &earlier, &latest).frame;
        assert_eq!(a, b);
        assert!(a.wave.is_some(), "the first wave starts within 4 s");
        assert_eq!(a.weapon, WeaponKind::Projectile);
    }

    #[test]
    fn densest_picks_the_crowded_enemy() {
        let lone = Vec2::new(500.0, 0.0);
        let crowd = [Vec2::ZERO, Vec2::X * 10.0, Vec2::Y * 10.0];
        let mut all = vec![lone];
        all.extend(crowd);
        assert!(crowd.contains(&densest(&all).expect("enemies exist")));
    }
}

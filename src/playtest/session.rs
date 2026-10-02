//! One headless playtest session: the real game plugins, one bot, a fixed
//! clock, and telemetry's record of what happened.

use super::bot::PlaytestBotPlugin;
use super::tier::{PickupPolicy, SkillTier, SpendPolicy, WeaponPolicy};
use crate::FlowArenaPlugins;
use crate::app_setup::api::FIXED_HZ;
use crate::debug_render::DebugRenderPlugin;
use crate::flow_director::api::Difficulty;
use crate::flow_director::{DirectorConfig, FlowDirectorPlugin};
use crate::pickups::PickupsPlugin;
use crate::pickups::api::PickupRules;
use crate::telemetry::api::SessionRecord;
use crate::weapons::api::WeaponKind;
use bevy::ecs::schedule::SingleThreadedExecutor;
use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use std::time::Duration;

/// What to play.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SessionConfig {
    pub tier: SkillTier,
    pub seed: u32,
    /// Simulated minutes, not wall-clock.
    pub minutes: f32,
    /// Hold only this weapon instead of the tier's own weapon choice.
    pub weapon_lock: Option<WeaponKind>,
    /// Play every wave at this difficulty instead of letting the director steer.
    pub pinned: Option<Difficulty>,
    /// Pickup rule set (prototypes); `Classic` is the game as it ships.
    pub pickup_rules: PickupRules,
    /// Walk to pickups this way instead of the tier's own policy.
    pub pickup_policy: Option<PickupPolicy>,
    /// Spend shards this way instead of the tier's own policy.
    pub spend: Option<SpendPolicy>,
}

/// Run one session to completion and return its log.
pub fn play(config: SessionConfig) -> SessionRecord {
    let mut params = config.tier.params();
    if let Some(weapon) = config.weapon_lock {
        params.weapon = WeaponPolicy::Fixed(weapon);
    }
    if let Some(policy) = config.pickup_policy {
        params.pickups = policy;
        // An ignoring tier has no reach; give an overriding policy the casual one.
        params.pickup_reach = params
            .pickup_reach
            .max(SkillTier::Casual.params().pickup_reach);
    }
    if let Some(spend) = config.spend {
        params.spend = spend;
    }
    let director = match config.pinned {
        Some(start) => FlowDirectorPlugin {
            start,
            config: DirectorConfig::pinned(),
        },
        None => FlowDirectorPlugin::default(),
    };
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin))
        .add_plugins(
            FlowArenaPlugins
                .build()
                .disable::<DebugRenderPlugin>()
                .set(director)
                .set(PickupsPlugin {
                    rules: config.pickup_rules,
                }),
        )
        .add_plugins(PlaytestBotPlugin {
            params,
            seed: config.seed,
        })
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / FIXED_HZ,
        )));

    // Many sessions run side by side, one per core; each runs its systems on
    // its own thread instead of sharing Bevy's task pool with the others.
    for (_, schedule) in app.world_mut().resource_mut::<Schedules>().iter_mut() {
        schedule.set_executor(SingleThreadedExecutor::new());
    }

    // One app update is one simulation tick at this clock.
    let ticks = (f64::from(config.minutes.max(0.0)) * 60.0 * FIXED_HZ).round() as u64;
    for _ in 0..ticks {
        app.update();
    }
    // Telemetry records every session; take its record.
    app.world_mut()
        .remove_resource::<SessionRecord>()
        .unwrap_or_default()
}

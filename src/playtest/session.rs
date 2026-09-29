//! One headless playtest session: the real game plugins, one bot, a fixed
//! clock, and telemetry's record of what happened.

use super::bot::PlaytestBotPlugin;
use super::tier::{SkillTier, WeaponPolicy};
use crate::FlowArenaPlugins;
use crate::app_setup::api::FIXED_HZ;
use crate::debug_render::DebugRenderPlugin;
use crate::telemetry::api::SessionRecord;
use crate::weapons::api::WeaponKind;
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
}

/// Run one session to completion and return its log.
pub fn play(config: SessionConfig) -> SessionRecord {
    let mut params = config.tier.params();
    if let Some(weapon) = config.weapon_lock {
        params.weapon = WeaponPolicy::Fixed(weapon);
    }
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin))
        .add_plugins(FlowArenaPlugins.build().disable::<DebugRenderPlugin>())
        .add_plugins(PlaytestBotPlugin {
            params,
            seed: config.seed,
        })
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / FIXED_HZ,
        )));

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

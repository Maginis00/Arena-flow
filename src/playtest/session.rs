//! One headless playtest session: the real game plugins, one bot, a fixed
//! clock, and a log of what happened.

use super::bot::PlaytestBotPlugin;
use super::tier::SkillTier;
use crate::FlowArenaPlugins;
use crate::app_setup::api::FIXED_HZ;
use crate::debug_render::DebugRenderPlugin;
use crate::flow_director::api::DifficultyAdjusted;
use crate::pickups::api::{PickupCollected, PickupKind};
use crate::waves::api::WaveReport;
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
}

/// Everything the session observed, in order.
#[derive(Resource, Debug, Clone, Default)]
pub struct SessionLog {
    pub reports: Vec<WaveReport>,
    pub decisions: Vec<DifficultyAdjusted>,
    /// Collected pickups, indexed like [`PickupKind::ALL`].
    pub pickups: [u32; 3],
}

impl SessionLog {
    pub fn pickups_of(&self, kind: PickupKind) -> u32 {
        PickupKind::ALL
            .iter()
            .position(|k| *k == kind)
            .and_then(|i| self.pickups.get(i).copied())
            .unwrap_or(0)
    }
}

/// Run one session to completion and return its log.
pub fn play(config: SessionConfig) -> SessionLog {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin))
        .add_plugins(FlowArenaPlugins.build().disable::<DebugRenderPlugin>())
        .add_plugins(PlaytestBotPlugin {
            tier: config.tier,
            seed: config.seed,
        })
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / FIXED_HZ,
        )))
        .init_resource::<SessionLog>()
        .add_systems(Update, record);

    // One app update is one simulation tick at this clock.
    let ticks = (f64::from(config.minutes.max(0.0)) * 60.0 * FIXED_HZ).round() as u64;
    for _ in 0..ticks {
        app.update();
    }
    app.world_mut()
        .remove_resource::<SessionLog>()
        .unwrap_or_default()
}

fn record(
    mut log: ResMut<SessionLog>,
    mut reports: MessageReader<WaveReport>,
    mut decisions: MessageReader<DifficultyAdjusted>,
    mut pickups: MessageReader<PickupCollected>,
) {
    log.reports.extend(reports.read().copied());
    log.decisions.extend(decisions.read().copied());
    for collected in pickups.read() {
        if let Some(i) = PickupKind::ALL.iter().position(|k| *k == collected.kind) {
            log.pickups[i] += 1;
        }
    }
}

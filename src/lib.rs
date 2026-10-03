//! flow_arena: a 2D wave-arena test harness for a flow-channel difficulty director.
//!
//! One module per domain, each exposing a `Plugin`. A domain's root file decides
//! what is public with `pub use`; its submodules stay private, so other domains
//! can only reach `crate::<domain>::Item`.

// Bevy system signatures are type-heavy by nature; Bevy itself recommends
// allowing this lint for ECS code.
#![allow(clippy::type_complexity)]

pub mod app_setup;
pub mod arena;
pub mod camera;
pub mod combat;
pub mod debug_render;
pub mod enemies;
pub mod flow_director;
#[cfg(feature = "fx")]
pub mod fx;
pub mod pickups;
pub mod player;
pub mod playtest;
pub mod telemetry;
pub mod waves;
pub mod weapons;

use bevy::app::{PluginGroup, PluginGroupBuilder};

/// Every gameplay plugin, in no particular order: ordering lives in
/// [`app_setup::SimSet`], not in registration order.
pub struct FlowArenaPlugins;

impl PluginGroup for FlowArenaPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(app_setup::AppSetupPlugin)
            .add(arena::ArenaPlugin)
            .add(camera::CameraPlugin)
            .add(player::PlayerPlugin)
            .add(combat::CombatPlugin)
            .add(weapons::WeaponsPlugin)
            .add(enemies::EnemiesPlugin)
            .add(pickups::PickupsPlugin::default())
            .add(waves::WavesPlugin)
            .add(flow_director::FlowDirectorPlugin::default())
            .add(telemetry::TelemetryPlugin)
            .add(debug_render::DebugRenderPlugin)
    }
}

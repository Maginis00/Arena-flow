//! flow_arena: a 2D wave-arena test harness for a flow-channel difficulty director.
//!
//! One module per domain, each exposing a `Plugin` and an `api` module. Other
//! domains may only use items from `<domain>::api` (plus the plugin type itself).
//! Everything else in a domain is private to it.

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
pub mod pickups;
pub mod player;
pub mod telemetry;
pub mod waves;
pub mod weapons;

use bevy::app::{PluginGroup, PluginGroupBuilder};

/// Every gameplay plugin, in no particular order: ordering lives in
/// [`app_setup::api::SimSet`], not in registration order.
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
            .add(pickups::PickupsPlugin)
            .add(waves::WavesPlugin)
            .add(flow_director::FlowDirectorPlugin)
            .add(telemetry::TelemetryPlugin)
            .add(debug_render::DebugRenderPlugin)
    }
}

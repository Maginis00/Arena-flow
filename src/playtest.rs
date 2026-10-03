//! Simulated playtesters at several skill tiers. Each bot plays through the
//! real game plugins headless and deterministically, so balance changes can be
//! checked against every tier before they ship.
//!
//! Run the report with `cargo run --release --example playtest`, or watch one
//! bot play in the game window with `cargo run --example playtest -- --watch`,
//! or all four tiers side by side with `-- --watch-all`.
//!
//! `agent` lets an outside agent (an LLM, or a person) play in steps instead:
//! see `examples/agent_play.rs`.

mod agent;
mod agent_order;
mod agent_view;
mod bot;
mod choices;
mod perception;
mod report;
mod session;
mod steering;
mod strategy;
mod summary;
mod tier;
mod tiling;
mod watch;
mod work_area;

pub use agent::{AgentRun, replay};
pub use agent_order::Order;
pub use agent_view::{Feel, journal, render};
pub use bot::PlaytestBotPlugin;
pub use report::{Named, flow_table, pickup_table, table, weapon_table};
pub use session::{SessionConfig, play, play_many};
pub use strategy::{Outcome, standing_table, standings, verdict_table, verdicts};
pub use summary::Summary;
pub use tier::{
    HandSkill, PickupPolicy, SkillTier, SpendPolicy, TierParams, WeaponPolicy, hand_skill,
};
pub use tiling::Quadrant;
pub use watch::{WatchConfig, watch};

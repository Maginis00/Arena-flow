//! Simulated playtesters at several skill tiers. Each bot plays through the
//! real game plugins headless and deterministically, so balance changes can be
//! checked against every tier before they ship.
//!
//! Run the report with `cargo run --release --example playtest`, or watch one
//! bot play in the game window with `cargo run --example playtest -- --watch`,
//! or all four tiers side by side with `-- --watch-all`.

mod bot;
mod choices;
mod perception;
mod report;
mod session;
mod steering;
mod summary;
mod tier;
mod tiling;
mod watch;
mod work_area;

pub use bot::PlaytestBotPlugin;
pub use report::{Named, pickup_table, table, weapon_table};
pub use session::{SessionConfig, play};
pub use summary::Summary;
pub use tier::{PickupPolicy, SkillTier, TierParams, WeaponPolicy};
pub use tiling::Quadrant;
pub use watch::{WatchConfig, watch};

//! Simulated playtesters at several skill tiers. Each bot plays through the
//! real game plugins headless and deterministically, so balance changes can be
//! checked against every tier before they ship.
//!
//! Run the report with `cargo run --release --example playtest`.

mod bot;
mod choices;
mod perception;
mod session;
mod steering;
mod summary;
mod tier;
mod weapon_use;

pub use bot::PlaytestBotPlugin;
pub use session::{SessionConfig, SessionLog, play};
pub use summary::{Summary, table, weapon_table};
pub use tier::{PickupPolicy, SkillTier, TierParams, WeaponPolicy};
pub use weapon_use::{WEAPONS, WeaponTally, WeaponUse};

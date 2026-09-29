use bevy::prelude::*;

/// Simulation tick rate. All gameplay simulation runs in `FixedUpdate`.
pub const FIXED_HZ: f64 = 60.0;

/// Ordered phases of one simulation tick in `FixedUpdate`.
///
/// Messages written in a later set are read by earlier sets on the next tick;
/// Bevy keeps messages alive until `FixedUpdate` has run, so nothing is lost.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimSet {
    /// Turn sampled input into intents (`FireRequested`).
    Intent,
    /// Create entities: projectiles, enemies.
    Spawn,
    /// Integrate positions.
    Movement,
    /// Overlap tests; emit `Hit`.
    Detect,
    /// Apply `Hit`: damage, deaths, despawns.
    Resolve,
    /// Wave state machine and per-wave measurement.
    Progress,
    /// Flow director decisions.
    Direct,
    /// Out-of-bounds and end-of-wave cleanup.
    Cleanup,
}

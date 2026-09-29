//! Wave state machine: Idle -> Spawning -> Active -> Cleared -> Intermission -> Spawning ...
//!
//! A failed wave (player died) is retried: the next wave keeps the same index
//! with `attempt + 1`, at whatever difficulty the director picked.
//!
//! Runs entirely in `FixedUpdate` as a plain resource rather than Bevy `States`,
//! so transitions happen on simulation ticks instead of at frame boundaries.
//! `Cleared` covers both outcomes: the wave has left play, cleared or failed.
//!
//! The machine's data lives in `machine`, the counting in `measure`, and the
//! phase changes in `transitions`.

pub mod api;

mod machine;
mod measure;
mod transitions;

use crate::app_setup::api::SimSet;
use api::{WaveCleared, WaveFailed, WaveReport, WaveStarted};
use bevy::prelude::*;

pub struct WavesPlugin;

impl Plugin for WavesPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<WaveStarted>()
            .add_message::<WaveCleared>()
            .add_message::<WaveFailed>()
            .add_message::<WaveReport>()
            .init_resource::<machine::WaveMachine>()
            .add_systems(
                FixedUpdate,
                (
                    measure::record_director,
                    measure::track_hp,
                    measure::measure,
                    transitions::advance,
                )
                    .chain()
                    .in_set(SimSet::Progress),
            );
    }
}

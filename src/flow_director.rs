//! Flow director: reads `WaveReport`, writes `DifficultyAdjusted`.
//!
//! The decision itself is [`decide`], a pure function. The systems here only
//! do IO: read the report, call `decide`, store memory, write the result.
//! [`engagement`] places a wave's risk on the flow curve; the director's band
//! sits around its peak, and the playtest report scores sessions with it.

pub mod api;
mod curve;
mod decide;

pub use curve::{FLOW_PEAK, IN_FLOW, engagement};
pub use decide::{Decision, DirectorConfig, DirectorMemory, decide, levers_for, wave_risk};

use crate::app_setup::api::SimSet;
use crate::waves::api::WaveReport;
use api::{DecisionReason, Difficulty, DifficultyAdjusted};
use bevy::prelude::*;

/// PLACEHOLDER: difficulty of the first wave.
const STARTING_DIFFICULTY: Difficulty = match Difficulty::new(3) {
    Ok(d) => d,
    Err(_) => Difficulty::MIN,
};

/// The director, starting at `start` and steering by `config`. The default is
/// the game's; playtests pin the difficulty with [`DirectorConfig::pinned`].
pub struct FlowDirectorPlugin {
    pub start: Difficulty,
    pub config: DirectorConfig,
}

impl Default for FlowDirectorPlugin {
    fn default() -> Self {
        Self {
            start: STARTING_DIFFICULTY,
            config: DirectorConfig::default(),
        }
    }
}

impl Plugin for FlowDirectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DifficultyAdjusted>()
            .insert_resource(DirectorState {
                difficulty: self.start,
                memory: DirectorMemory::default(),
            })
            .insert_resource(Config(self.config))
            .add_systems(Startup, announce_initial)
            .add_systems(FixedUpdate, adjust_after_wave.in_set(SimSet::Direct));
    }
}

#[derive(Resource, Debug, Clone, Copy)]
struct DirectorState {
    difficulty: Difficulty,
    memory: DirectorMemory,
}

#[derive(Resource, Debug, Clone, Copy)]
struct Config(DirectorConfig);

fn announce_initial(state: Res<DirectorState>, mut adjusted: MessageWriter<DifficultyAdjusted>) {
    adjusted.write(DifficultyAdjusted {
        previous: state.difficulty,
        difficulty: state.difficulty,
        reason: DecisionReason::Initial,
        levers: levers_for(state.difficulty),
        risk: None,
    });
}

fn adjust_after_wave(
    mut reports: MessageReader<WaveReport>,
    config: Res<Config>,
    mut state: ResMut<DirectorState>,
    mut adjusted: MessageWriter<DifficultyAdjusted>,
) {
    for report in reports.read() {
        let previous = state.difficulty;
        let decision = decide(previous, report, state.memory, &config.0);
        state.difficulty = decision.difficulty;
        state.memory = decision.memory;
        adjusted.write(DifficultyAdjusted {
            previous,
            difficulty: decision.difficulty,
            reason: decision.reason,
            levers: levers_for(decision.difficulty),
            risk: Some(decision.risk),
        });
    }
}

//! Watch one bot play in the normal game window at real-time speed: the same
//! game plugins and bot as a headless session, plus a label saying who plays
//! and what the director just decided.

use super::bot::PlaytestBotPlugin;
use super::tier::{SkillTier, WeaponPolicy};
use crate::FlowArenaPlugins;
use crate::flow_director::api::{DecisionReason, Difficulty, DifficultyAdjusted};
use crate::telemetry::api::SessionFileEnabled;
use crate::weapons::api::WeaponKind;
use bevy::prelude::*;

/// Who to watch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WatchConfig {
    pub tier: SkillTier,
    pub seed: u32,
    /// Pins one weapon instead of the tier's weapon policy.
    pub weapon: Option<WeaponKind>,
}

/// Open the game window with a bot at the controls; returns when it closes.
pub fn watch(config: WatchConfig) -> AppExit {
    let mut params = config.tier.params();
    if let Some(weapon) = config.weapon {
        params.weapon = WeaponPolicy::Fixed(weapon);
    }
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: format!("flow_arena: {} bot", config.tier),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FlowArenaPlugins)
        .add_plugins(PlaytestBotPlugin {
            params,
            seed: config.seed,
        })
        .insert_resource(SessionFileEnabled(false))
        .add_plugins(WatchLabelPlugin(config))
        .run()
}

struct WatchLabelPlugin(WatchConfig);

impl Plugin for WatchLabelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Watched {
            config: self.0,
            difficulty: None,
            reason: None,
        })
        .add_systems(Startup, spawn_label)
        .add_systems(Update, update_label);
    }
}

#[derive(Resource, Debug)]
struct Watched {
    config: WatchConfig,
    difficulty: Option<Difficulty>,
    reason: Option<DecisionReason>,
}

#[derive(Component)]
struct WatchLabel;

fn spawn_label(mut commands: Commands) {
    // Bottom left, clear of the debug overlay in the top left.
    commands.spawn((
        WatchLabel,
        Text::new(""),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            bottom: Val::Px(8.0),
            ..default()
        },
    ));
}

fn update_label(
    mut watched: ResMut<Watched>,
    mut adjusted: MessageReader<DifficultyAdjusted>,
    mut label: Query<&mut Text, With<WatchLabel>>,
) {
    if let Some(a) = adjusted.read().last() {
        watched.difficulty = Some(a.difficulty);
        watched.reason = Some(a.reason);
    }
    let c = watched.config;
    let weapon = match (c.weapon, c.tier.params().weapon) {
        (Some(weapon), _) | (None, WeaponPolicy::Fixed(weapon)) => weapon.to_string(),
        (None, WeaponPolicy::Situational) => "situational".to_owned(),
    };
    let difficulty = watched
        .difficulty
        .map_or_else(|| "-".to_owned(), |d| d.to_string());
    let reason = watched
        .reason
        .map_or_else(|| "-".to_owned(), |r| r.to_string());
    for mut text in &mut label {
        text.0 = format!(
            "bot {} #{}  weapon {weapon}  next difficulty {difficulty}\ndirector: {reason}",
            c.tier, c.seed
        );
    }
}

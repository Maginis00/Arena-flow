//! Watch one bot play in the normal game window at real-time speed: the same
//! game plugins and bot as a headless session, plus a label saying who plays
//! and what the director just decided. With a quadrant set, the window drops
//! its frame and fills that quarter of the primary monitor's work area, so
//! four bots can be watched side by side.

use super::bot::PlaytestBotPlugin;
use super::tier::{SkillTier, WeaponPolicy};
use super::tiling::Quadrant;
use super::work_area::primary_work_area;
use crate::FlowArenaPlugins;
use crate::flow_director::api::{DecisionReason, Difficulty, DifficultyAdjusted};
use crate::telemetry::api::{OverlayEnabled, SessionFileEnabled};
use crate::waves::api::{WaveIndex, WaveStarted};
use crate::weapons::api::WeaponKind;
use bevy::prelude::*;
use bevy::window::{Monitor, PrimaryMonitor, PrimaryWindow, WindowPosition};

/// Who to watch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WatchConfig {
    pub tier: SkillTier,
    pub seed: u32,
    /// Pins one weapon instead of the tier's weapon policy.
    pub weapon: Option<WeaponKind>,
    /// Fill this quarter of the screen instead of a normal 1280x720 window.
    pub quadrant: Option<Quadrant>,
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
                // A tile has no frame, so the four tile the screen exactly. It
                // stays hidden until `place_in_quadrant` has moved it.
                decorations: config.quadrant.is_none(),
                visible: config.quadrant.is_none(),
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
        // A quarter screen has room for the bot's label only.
        .insert_resource(OverlayEnabled(config.quadrant.is_none()))
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
            wave: None,
        })
        .add_systems(Startup, spawn_label)
        .add_systems(Update, update_label);
        if let Some(quadrant) = self.0.quadrant {
            app.insert_resource(Placement(quadrant)).add_systems(
                Update,
                place_in_quadrant.run_if(resource_exists::<Placement>),
            );
        }
    }
}

#[derive(Resource, Debug)]
struct Watched {
    config: WatchConfig,
    difficulty: Option<Difficulty>,
    reason: Option<DecisionReason>,
    /// The wave in play and the difficulty it runs at (the compact label).
    wave: Option<(WaveIndex, Difficulty)>,
}

#[derive(Component)]
struct WatchLabel;

/// PLACEHOLDER: font size of the compact label in a quarter-screen window.
const COMPACT_FONT_SIZE: f32 = 14.0;

fn spawn_label(mut commands: Commands, watched: Res<Watched>) {
    let mut label = commands.spawn((WatchLabel, Text::new("")));
    if watched.config.quadrant.is_some() {
        // No overlay in a tile: a small label in the top left is all there is.
        label.insert((
            TextFont::from_font_size(COMPACT_FONT_SIZE),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(8.0),
                top: Val::Px(4.0),
                ..default()
            },
        ));
    } else {
        // Bottom left, clear of the debug overlay in the top left.
        label.insert(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            bottom: Val::Px(8.0),
            ..default()
        });
    }
}

fn update_label(
    mut watched: ResMut<Watched>,
    mut adjusted: MessageReader<DifficultyAdjusted>,
    mut started: MessageReader<WaveStarted>,
    mut label: Query<&mut Text, With<WatchLabel>>,
) {
    if let Some(a) = adjusted.read().last() {
        watched.difficulty = Some(a.difficulty);
        watched.reason = Some(a.reason);
    }
    if let Some(s) = started.read().last() {
        watched.wave = Some((s.spec.index, s.spec.difficulty));
    }
    let c = watched.config;
    if c.quadrant.is_some() {
        let wave = watched.wave.map_or_else(
            || "-".to_owned(),
            |(index, difficulty)| format!("{index}  difficulty {difficulty}"),
        );
        for mut text in &mut label {
            text.0 = format!("{}  wave {wave}", c.tier);
        }
        return;
    }
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

/// The quadrant the window still has to move to; removed once it has.
#[derive(Resource, Debug)]
struct Placement(Quadrant);

/// Moves the window into its quadrant once the primary monitor is known
/// (winit reports monitors only after the event loop starts), then shows it.
fn place_in_quadrant(
    mut commands: Commands,
    placement: Res<Placement>,
    monitor: Option<Single<&Monitor, With<PrimaryMonitor>>>,
    window: Option<Single<&mut Window, With<PrimaryWindow>>>,
) {
    let (Some(monitor), Some(mut window)) = (monitor, window) else {
        return;
    };
    let whole = IRect::from_corners(
        monitor.physical_position,
        monitor.physical_position + monitor.physical_size().as_ivec2(),
    );
    let area = primary_work_area().unwrap_or(whole);
    let tile = placement.0.rect(area);
    let size = tile.size().max(IVec2::ONE).as_uvec2();
    window.position = WindowPosition::At(tile.min);
    window.resolution.set_physical_resolution(size.x, size.y);
    window.visible = true;
    commands.remove_resource::<Placement>();
}

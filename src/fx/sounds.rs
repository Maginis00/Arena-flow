//! One sample per game fact. The files live in `assets/sfx/`; any CC0 sample
//! with the same name replaces one.

use super::Muted;
use crate::combat::{EnemyKilled, Hit, HitSource, PlayerDamaged, PlayerDied, Team};
use crate::enemies::ChargeTell;
use crate::pickups::PickupCollected;
use crate::waves::WaveCleared;
use crate::weapons::{ShotFired, WeaponKind};
use bevy::audio::Volume;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Sfx {
    ShotHitscan,
    ShotProjectile,
    Swing,
    EnemyHit,
    EnemyDie,
    PlayerHit,
    PlayerDie,
    ChargeTell,
    Pickup,
    WaveClear,
}

impl Sfx {
    const ALL: [Sfx; 10] = [
        Sfx::ShotHitscan,
        Sfx::ShotProjectile,
        Sfx::Swing,
        Sfx::EnemyHit,
        Sfx::EnemyDie,
        Sfx::PlayerHit,
        Sfx::PlayerDie,
        Sfx::ChargeTell,
        Sfx::Pickup,
        Sfx::WaveClear,
    ];

    fn path(self) -> &'static str {
        match self {
            Sfx::ShotHitscan => "sfx/shot_hitscan.ogg",
            Sfx::ShotProjectile => "sfx/shot_projectile.ogg",
            Sfx::Swing => "sfx/swing.ogg",
            Sfx::EnemyHit => "sfx/enemy_hit.ogg",
            Sfx::EnemyDie => "sfx/enemy_die.ogg",
            Sfx::PlayerHit => "sfx/player_hit.ogg",
            Sfx::PlayerDie => "sfx/player_die.ogg",
            Sfx::ChargeTell => "sfx/charge_tell.ogg",
            Sfx::Pickup => "sfx/pickup.ogg",
            Sfx::WaveClear => "sfx/wave_clear.ogg",
        }
    }

    /// PLACEHOLDER: mix levels. Shots repeat fast, so they sit lowest; what
    /// the player must notice (getting hit, a charger winding up) sits highest.
    fn volume(self) -> f32 {
        match self {
            Sfx::ShotHitscan | Sfx::ShotProjectile => 0.25,
            Sfx::Swing => 0.35,
            Sfx::EnemyHit => 0.3,
            Sfx::EnemyDie => 0.45,
            Sfx::PlayerHit | Sfx::PlayerDie => 0.8,
            Sfx::ChargeTell => 0.6,
            Sfx::Pickup | Sfx::WaveClear => 0.5,
        }
    }
}

#[derive(Resource, Debug)]
pub(super) struct Sounds(Vec<(Sfx, Handle<AudioSource>)>);

pub(super) fn load_sounds(mut commands: Commands, assets: Res<AssetServer>) {
    let handles = Sfx::ALL
        .iter()
        .map(|&sfx| (sfx, assets.load(sfx.path())))
        .collect();
    commands.insert_resource(Sounds(handles));
}

/// The simulation facts that have a sound.
#[derive(SystemParam)]
pub(super) struct Facts<'w, 's> {
    shots: MessageReader<'w, 's, ShotFired>,
    hits: MessageReader<'w, 's, Hit>,
    kills: MessageReader<'w, 's, EnemyKilled>,
    damaged: MessageReader<'w, 's, PlayerDamaged>,
    died: MessageReader<'w, 's, PlayerDied>,
    collected: MessageReader<'w, 's, PickupCollected>,
    cleared: MessageReader<'w, 's, WaveCleared>,
}

impl Facts<'_, '_> {
    /// Every sound due this frame, each at most once: a sword swing killing
    /// six enemies is one death sound, not six stacked ones.
    fn due(&mut self) -> Vec<Sfx> {
        let mut due: Vec<Sfx> = self
            .shots
            .read()
            .filter(|s| s.team == Team::Player)
            .map(|s| match s.weapon {
                WeaponKind::Hitscan => Sfx::ShotHitscan,
                WeaponKind::Projectile => Sfx::ShotProjectile,
                WeaponKind::Melee => Sfx::Swing,
            })
            .collect();
        let weapon_hit = self
            .hits
            .read()
            .any(|h| matches!(h.source, HitSource::Shot { .. } | HitSource::Blast));
        let flags = [
            (weapon_hit, Sfx::EnemyHit),
            (self.kills.read().count() > 0, Sfx::EnemyDie),
            (self.damaged.read().count() > 0, Sfx::PlayerHit),
            (self.died.read().count() > 0, Sfx::PlayerDie),
            (self.collected.read().count() > 0, Sfx::Pickup),
            (self.cleared.read().count() > 0, Sfx::WaveClear),
        ];
        due.extend(flags.iter().filter(|(any, _)| *any).map(|(_, sfx)| *sfx));
        due.sort_by_key(|sfx| Sfx::ALL.iter().position(|s| s == sfx));
        due.dedup();
        due
    }
}

pub(super) fn play_sounds(
    mut commands: Commands,
    sounds: Option<Res<Sounds>>,
    muted: Res<Muted>,
    mut facts: Facts,
) {
    // Read every frame, even muted, so facts don't pile up and play late.
    let due = facts.due();
    let (Some(sounds), false) = (sounds, muted.0) else {
        return;
    };
    for sfx in due {
        play(&mut commands, &sounds, sfx);
    }
}

/// A charger starting its wind-up is heard as well as seen.
pub(super) fn play_charge_tells(
    mut commands: Commands,
    sounds: Option<Res<Sounds>>,
    muted: Res<Muted>,
    tells: Query<&ChargeTell, Changed<ChargeTell>>,
) {
    let started = tells.iter().any(|tell| tell.aim.is_some());
    let (Some(sounds), false, true) = (sounds, muted.0, started) else {
        return;
    };
    play(&mut commands, &sounds, Sfx::ChargeTell);
}

fn play(commands: &mut Commands, sounds: &Sounds, sfx: Sfx) {
    let Some((_, handle)) = sounds.0.iter().find(|(s, _)| *s == sfx) else {
        return;
    };
    commands.spawn((
        AudioPlayer::new(handle.clone()),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(sfx.volume())),
    ));
}

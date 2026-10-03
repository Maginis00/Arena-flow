//! Shots, projectiles and the hit facts combat resolves and reports.

use super::Team;
use bevy::prelude::*;

/// Identifies one trigger pull of a weapon. A melee swing that strikes three
/// enemies is still one shot, so accuracy counts shots, not hits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ShotId(pub u32);

/// A damaging projectile. Despawned by combat after its first hit.
#[derive(Component, Debug, Clone, Copy)]
pub struct Projectile {
    pub damage: u32,
    pub team: Team,
    pub shot: ShotId,
}

/// What dealt a [`Hit`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitSource {
    /// A weapon shot. `projectile` is set when a projectile entity delivered it.
    Shot {
        shot: ShotId,
        projectile: Option<Entity>,
    },
    /// An enemy touching the player.
    Contact(Entity),
    /// An enemy bolt (the entity named) reaching the player.
    EnemyShot(Entity),
    /// The player's shard blast (pickup prototype).
    Blast,
}

/// A confirmed hit that should deal damage. Written by combat (weapon shots)
/// and enemies (contact, bolts); applied by combat.
#[derive(Message, Debug, Clone, Copy)]
pub struct Hit {
    pub target: Entity,
    pub damage: u32,
    pub source: HitSource,
}

/// The player lost health.
#[derive(Message, Debug, Clone, Copy)]
pub struct PlayerDamaged {
    pub amount: u32,
    pub remaining: u32,
    pub max: u32,
}

/// The player's health reached zero.
#[derive(Message, Debug, Clone, Copy)]
pub struct PlayerDied {
    pub player: Entity,
}

/// An enemy's health reached zero. Combat despawns it.
#[derive(Message, Debug, Clone, Copy)]
pub struct EnemyKilled {
    pub enemy: Entity,
    pub at: Vec2,
}

/// Restore hp to `target`, capped at max. Written by pickups; applied by combat.
#[derive(Message, Debug, Clone, Copy)]
pub struct HealGranted {
    pub target: Entity,
    pub amount: u32,
}

/// The player regained health.
#[derive(Message, Debug, Clone, Copy)]
pub struct PlayerHealed {
    pub amount: u32,
    pub remaining: u32,
    pub max: u32,
}

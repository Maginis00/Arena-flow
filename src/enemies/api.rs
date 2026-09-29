use bevy::prelude::*;

/// Marker for enemy entities.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Enemy;

/// An enemy from the current wave entered the arena.
#[derive(Message, Debug, Clone, Copy)]
pub struct EnemySpawned {
    pub enemy: Entity,
}

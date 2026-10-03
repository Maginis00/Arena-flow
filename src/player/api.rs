use bevy::prelude::*;

/// Marker for the player entity.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Player;

/// A (re)spawned player entered the arena.
#[derive(Message, Debug, Clone, Copy)]
pub struct PlayerSpawned {
    pub player: Entity,
    pub max_hp: u32,
}

/// The player is holding the fire input this tick. Weapons decide whether a
/// shot actually happens (cooldown); this is only the intent.
#[derive(Message, Debug, Clone, Copy)]
pub struct FireRequested {
    pub shooter: Entity,
    pub origin: Vec2,
    /// Unit vector toward the cursor.
    pub direction: Vec2,
}

/// The player is holding the sword input (right mouse button) this tick.
/// Only does something while the sword sits on the right mouse button;
/// weapons decide whether a swing happens (its own cooldown).
#[derive(Message, Debug, Clone, Copy)]
pub struct SwingRequested {
    pub shooter: Entity,
    pub origin: Vec2,
    /// Unit vector toward the cursor.
    pub direction: Vec2,
}

use bevy::prelude::*;

/// Which side an entity fights for. Projectiles never hit their own team.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Team {
    Player,
    Enemy,
}

/// Hit points. Only the combat plugin mutates health; other plugins construct it.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Health {
    current: u32,
    max: u32,
}

impl Health {
    pub const fn full(max: u32) -> Self {
        Self { current: max, max }
    }

    pub const fn current(&self) -> u32 {
        self.current
    }

    pub const fn max(&self) -> u32 {
        self.max
    }

    pub const fn is_dead(&self) -> bool {
        self.current == 0
    }

    /// Returns the damage actually applied (never more than the remaining hp).
    pub(super) fn take(&mut self, amount: u32) -> u32 {
        let applied = amount.min(self.current);
        self.current -= applied;
        applied
    }
}

/// Axis-aligned collision box, centred on the entity's translation.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Hitbox {
    pub half_extents: Vec2,
}

impl Hitbox {
    pub const fn square(half: f32) -> Self {
        Self {
            half_extents: Vec2::splat(half),
        }
    }

    pub fn overlaps(self, at: Vec2, other: Hitbox, other_at: Vec2) -> bool {
        (at - other_at)
            .abs()
            .cmple(self.half_extents + other.half_extents)
            .all()
    }
}

/// A damaging projectile. Despawned by combat after its first hit.
#[derive(Component, Debug, Clone, Copy)]
pub struct Projectile {
    pub damage: u32,
    pub team: Team,
}

/// What dealt a [`Hit`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitSource {
    Projectile(Entity),
    Contact(Entity),
}

/// A confirmed overlap that should deal damage. Written by combat (projectiles)
/// and enemies (contact); applied by combat.
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
}

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

    /// Returns the hp actually restored (never above max).
    pub(super) fn restore(&mut self, amount: u32) -> u32 {
        let applied = amount.min(self.max - self.current);
        self.current += applied;
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

    /// Distance along a ray from `origin` in unit `direction` to this box at
    /// `at`, if the ray enters it within `max_distance`. Slab test.
    pub fn ray_distance(
        self,
        at: Vec2,
        origin: Vec2,
        direction: Vec2,
        max_distance: f32,
    ) -> Option<f32> {
        let min = at - self.half_extents;
        let max = at + self.half_extents;
        let mut t_near = 0.0_f32;
        let mut t_far = max_distance;
        for axis in 0..2 {
            let (o, d, lo, hi) = (origin[axis], direction[axis], min[axis], max[axis]);
            if d.abs() < f32::EPSILON {
                if o < lo || o > hi {
                    return None;
                }
                continue;
            }
            let (t1, t2) = ((lo - o) / d, (hi - o) / d);
            t_near = t_near.max(t1.min(t2));
            t_far = t_far.min(t1.max(t2));
            if t_near > t_far {
                return None;
            }
        }
        Some(t_near)
    }
}

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
    /// The player's shard blast (pickup prototype).
    Blast,
}

/// A confirmed hit that should deal damage. Written by combat (weapon shots)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_hits_box_ahead_and_misses_box_behind_or_beside() {
        let hitbox = Hitbox::square(10.0);
        let ahead = hitbox.ray_distance(Vec2::new(100.0, 0.0), Vec2::ZERO, Vec2::X, 500.0);
        assert_eq!(ahead, Some(90.0));
        assert_eq!(
            hitbox.ray_distance(Vec2::new(-100.0, 0.0), Vec2::ZERO, Vec2::X, 500.0),
            None
        );
        assert_eq!(
            hitbox.ray_distance(Vec2::new(100.0, 50.0), Vec2::ZERO, Vec2::X, 500.0),
            None
        );
        assert_eq!(
            hitbox.ray_distance(Vec2::new(600.0, 0.0), Vec2::ZERO, Vec2::X, 500.0),
            None
        );
    }
}

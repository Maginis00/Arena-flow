//! What a simulated player sees: the world as it was a reaction time ago.

use crate::pickups::api::PickupKind;
use bevy::math::Vec2;
use std::collections::VecDeque;

/// One tick of the world, reduced to what a player reads off the screen.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Snapshot {
    pub enemies: Vec<Vec2>,
    /// Enemies worth shooting first (summoners), also listed in `enemies`.
    pub priority: Vec<Vec2>,
    /// Enemy bolts in flight.
    pub bolts: Vec<Vec2>,
    /// Chargers winding up: where they stand and where they will dash.
    pub tells: Vec<(Vec2, Vec2)>,
    pub pickups: Vec<(Vec2, PickupKind)>,
    /// Shards on the floor (shard pickup rules only).
    pub shards: Vec<Vec2>,
}

/// A fixed-length delay line of snapshots. Until it fills, the bot sees the
/// oldest snapshot it has, like a player who only just looked at the screen.
#[derive(Debug, Clone, Default)]
pub struct DelayedView {
    delay_ticks: usize,
    buffer: VecDeque<Snapshot>,
}

impl DelayedView {
    pub fn new(delay_ticks: usize) -> Self {
        Self {
            delay_ticks,
            buffer: VecDeque::with_capacity(delay_ticks + 1),
        }
    }

    pub fn push(&mut self, snapshot: Snapshot) {
        self.buffer.push_back(snapshot);
        while self.buffer.len() > self.delay_ticks + 1 {
            self.buffer.pop_front();
        }
    }

    /// Change the reaction time. A longer delay fills up from the frames kept.
    pub fn set_delay(&mut self, delay_ticks: usize) {
        self.delay_ticks = delay_ticks;
        while self.buffer.len() > self.delay_ticks + 1 {
            self.buffer.pop_front();
        }
    }

    /// The snapshot from `delay_ticks` ago (or the oldest one kept).
    pub fn seen(&self) -> Option<&Snapshot> {
        self.buffer.front()
    }
}

/// Small deterministic generator (xorshift32) so every run is repeatable.
#[derive(Debug, Clone, Copy)]
pub struct Rng(u32);

impl Rng {
    pub fn new(seed: u32) -> Self {
        // xorshift must never hold zero.
        Self(seed.wrapping_mul(0x9E37_79B9) | 1)
    }

    /// Uniform in `[-1, 1]`.
    pub fn signed_unit(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    /// Roughly bell-shaped in `[-1, 1]`: the mean of two uniforms.
    pub fn wobble(&mut self) -> f32 {
        (self.signed_unit() + self.signed_unit()) * 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_enemy(x: f32) -> Snapshot {
        Snapshot {
            enemies: vec![Vec2::new(x, 0.0)],
            ..Snapshot::default()
        }
    }

    #[test]
    fn view_lags_by_the_delay() {
        let mut view = DelayedView::new(2);
        for x in 0..5 {
            view.push(with_enemy(x as f32));
        }
        assert_eq!(view.seen(), Some(&with_enemy(2.0)));
    }

    #[test]
    fn a_shorter_delay_sees_a_newer_frame_at_once() {
        let mut view = DelayedView::new(3);
        for x in 0..5 {
            view.push(with_enemy(x as f32));
        }
        view.set_delay(1);
        assert_eq!(view.seen(), Some(&with_enemy(3.0)));
    }

    #[test]
    fn rng_is_repeatable_and_bounded() {
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        for _ in 0..100 {
            let x = a.wobble();
            assert_eq!(x, b.wobble());
            assert!((-1.0..=1.0).contains(&x));
        }
    }
}

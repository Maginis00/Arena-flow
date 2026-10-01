//! Short headless runs of the playtest bots through the real plugins.
//! The full report is `cargo run --release --example playtest`.

use flow_arena::enemies::api::EnemyMix;
use flow_arena::playtest::{SessionConfig, SkillTier, Summary, play};

fn settle_against(tier: SkillTier, enemies: EnemyMix) -> Summary {
    let log = play(SessionConfig {
        tier,
        seed: 1,
        minutes: 4.0,
        weapon_lock: None,
        enemies,
    });
    Summary::of(&log)
}

fn settle(tier: SkillTier) -> Summary {
    settle_against(tier, EnemyMix::Grunts)
}

#[test]
fn director_ranks_expert_above_novice() {
    let novice = settle(SkillTier::Novice);
    let expert = settle(SkillTier::Expert);
    eprintln!(
        "novice: {}\nexpert: {}",
        novice.trajectory, expert.trajectory
    );
    assert!(novice.waves >= 5 && expert.waves >= 5);
    assert!(novice.hit_rate > 0.0 && expert.hit_rate > 0.0);
    assert!(
        expert.settled > novice.settled,
        "expert settled at {}, novice at {}",
        expert.settled,
        novice.settled
    );
}

#[test]
fn waves_with_every_enemy_kind_still_end() {
    // Summoners add enemies beyond the wave's count; the wave must still
    // finish once everything is dead, and the director must keep working.
    let expert = settle_against(SkillTier::Expert, EnemyMix::All);
    eprintln!("expert vs all: {}", expert.trajectory);
    assert!(expert.waves >= 5, "only {} waves", expert.waves);
    assert!(expert.clears > 0);
}

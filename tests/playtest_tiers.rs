//! Short headless runs of the playtest bots through the real plugins.
//! The full report is `cargo run --release --example playtest`.

use flow_arena::enemies::api::EnemyMix;
use flow_arena::pickups::api::PickupRules;
use flow_arena::playtest::{SessionConfig, SkillTier, Summary, play};
use flow_arena::weapons::api::SwordBinding;

fn settle_with(tier: SkillTier, enemies: EnemyMix, pickup_rules: PickupRules) -> Summary {
    let log = play(SessionConfig {
        tier,
        seed: 1,
        minutes: 4.0,
        weapon_lock: None,
        enemies,
        pinned: None,
        pickup_rules,
        pickup_policy: None,
        spend: None,
        sword: SwordBinding::Key3,
    });
    Summary::of(&log)
}

fn settle(tier: SkillTier) -> Summary {
    settle_with(tier, EnemyMix::Grunts, PickupRules::Classic)
}

#[test]
fn every_pickup_prototype_plays_through() {
    for rules in PickupRules::ALL {
        let summary = settle_with(SkillTier::Skilled, EnemyMix::Grunts, rules);
        assert!(summary.waves >= 5, "{rules}: only {} waves", summary.waves);
    }
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
    // Summoners and swarms add enemies beyond the wave's count; the wave must still
    // finish once everything is dead, and the director must keep working.
    let expert = settle_with(SkillTier::Expert, EnemyMix::All, PickupRules::Classic);
    eprintln!("expert vs all: {}", expert.trajectory);
    assert!(expert.waves >= 5, "only {} waves", expert.waves);
    assert!(expert.clears > 0);
}

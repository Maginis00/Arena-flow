//! Short headless runs of the playtest bots through the real plugins.
//! The full report is `cargo run --release --example playtest`.

use flow_arena::playtest::{SessionConfig, SkillTier, Summary, play};

fn settle(tier: SkillTier) -> Summary {
    let log = play(SessionConfig {
        tier,
        seed: 1,
        minutes: 4.0,
        weapon_lock: None,
        pinned: None,
    });
    Summary::of(&log)
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

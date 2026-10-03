# flow_arena: key principles

Read this before changing anything. [ARCHITECTURE.md](ARCHITECTURE.md) has the
plugin graph, the message table, the schedule and the file layout; this file
says what must stay true.

## How to work with Jannus

- Don't just execute. Challenge the idea, check the premise, and say where the
  current state is already the right call.
- Keep open questions open: list them instead of inventing an answer. Product
  and design decisions belong to Jannus.
- Offer options with a recommendation for anything churny or hard to undo, and
  wait for the answer.
- Measure before restructuring. "This file is too long" is a claim to verify,
  not an instruction to follow.

## Design north star

- Simple on the surface, deep in mechanics and optimization. The reference is
  Devil Daggers: one platform, two attacks, with depth coming from gem
  collection and from enemies that differ in design and behaviour.
- The dynamic difficulty (flow director) should keep the game challenging for
  every skill level.
- Judge a feature by whether it adds mechanical or optimization depth without
  adding surface complexity. Pickups carry trade-offs that make the player
  think; nothing is free power.

## Process

- Playtest with the headless bots at several skill levels before adding more.
  Prefer tuning or changing what exists early over growing a game that can no
  longer be balanced. Check new features against the bot playtest report.
- Balance numbers are named `const`s marked `PLACEHOLDER`, next to the code
  that uses them. List new ones at the end of the PR or reply.
- A structure-only change must not change behaviour: prove it with the unit
  tests, the headless runs and `cargo clippy --all-targets -- -D warnings`.

## Architecture rules

- Bevy is pinned to `=0.19.1` (Rust 1.95+), edition 2024. Don't bump it casually.
- One plugin per domain. Each domain's public surface is its `api` module;
  other domains use only `crate::<domain>::api::*` plus the plugin type.
  Everything else in a domain is private to it.
- Domains talk through `#[derive(Message)]` facts (`MessageWriter` /
  `MessageReader`). Name facts plainly (`EnemyKilled`), with no `Message`
  suffix. Observers only where a message cannot work.
- All simulation runs in `FixedUpdate`, ordered by `SimSet`. `Update` is only
  for input sampling, camera, debug rendering and telemetry.
- No `unwrap`/`expect` in gameplay systems; handle the missing case
  (`let ... else`, `try_despawn`, `Option<Single<..>>`). Tests may use `expect`.
- `main.rs` only builds the window and adds `FlowArenaPlugins`, plus `FxPlugin`
  with the `fx` feature.
- Sound and particles live in `fx` (feature `fx`, off by default, `cargo play`).
  It only reads facts and never writes back, and bots never load it, so the
  simulation and the bot report stay the same with or without it.
- Keep the harness deterministic: no RNG in the simulation.

## Code structure

- Module files use the edition-2018+ style: `src/combat.rs` is the root of the
  `combat` module and `src/combat/` holds its submodules. No `mod.rs` files.
- Name a file for what it holds (`machine.rs`, `measure.rs`, `transitions.rs`,
  `decide.rs`, `table.rs`). `api.rs` is the one deliberate exception: it is the
  domain's public surface, so it is named for its role. No `utils.rs`,
  `misc.rs` or `helpers.rs`.
- Register every system in the domain's `Plugin::build`, so a domain's whole
  schedule reads in one place.
- Unit tests go in `#[cfg(test)] mod tests` at the bottom of the file they
  test. That is idiomatic Rust and tests do not count toward a file's length.
- Judge file length on non-test code: past roughly 200 lines of logic, split by
  what the code does, not to hit a number. Don't split a small domain into
  ten-line files; let it grow first.
- Keep pure logic that needs no ECS in its own file with its tests, as
  `flow_director/decide.rs` and `pickups/table.rs` do.
- Before pushing: `cargo fmt`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test`.

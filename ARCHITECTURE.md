# flow_arena architecture

Single crate, one module per domain. Each domain module has a `Plugin` and an
`api` submodule. Other domains may use only `<domain>::api` items (and the
plugin type). Everything else is private to the domain.

## Code layout

Modules use the edition-2018+ style: `src/combat.rs` is the root of the
`combat` module and `src/combat/` holds its submodules, so no file is called
`mod.rs`. The root file holds the module docs, the `Plugin` with all of the
domain's system registration, and the systems themselves while the domain is
small. `api.rs` is the public surface; further files are named for what they
hold.

```
src/
  lib.rs            FlowArenaPlugins
  main.rs           window + plugins only
  app_setup.rs      app_setup/api.rs        SimSet, fixed timestep, 2D camera
  arena.rs          arena/api.rs            ArenaBounds, out-of-bounds despawn
  camera.rs         camera/api.rs           light follow (exposes nothing)
  player.rs         player/api.rs           spawn, input, movement, fire intent
  weapons.rs        weapons/api.rs          selection, cooldowns, shots
  combat.rs         combat/api.rs           hit detection and resolution
  enemies.rs        enemies/api.rs          spawning, chasing, contact damage
                    enemies/placement.rs    spawn point outside the safe radius (+ tests)
  pickups.rs        pickups/api.rs          drops, collection, effect timers
                    pickups/table.rs        what each pickup does (pure + tests)
  waves.rs          waves/api.rs            the plugin and its system order
                    waves/machine.rs        phase, wave in play, running stats
                    waves/measure.rs        counting facts during a wave
                    waves/transitions.rs    phase changes, reports, next wave
  flow_director.rs  flow_director/api.rs    the ECS side: state and two systems
                    flow_director/decide.rs the pure decision (+ tests)
  telemetry.rs      telemetry/api.rs        snapshot + debug overlay
  debug_render.rs   debug_render/api.rs     boxes, border, shot flashes
```

Items shared between a domain's root and its submodules are `pub(super)`;
only `api` is `pub`. Unit tests live in `#[cfg(test)] mod tests` at the bottom
of the file they test, and don't count toward a file's length. A file splits
when its non-test code passes roughly 200 lines, by what the code does.

## Plugin graph

Arrows mean "uses the public api of". No domain touches another's private items.

```
main.rs ── FlowArenaPlugins (lib.rs)
             │
app_setup ◄──┴── every gameplay plugin (SimSet ordering)

arena ──────► (nothing)
camera ─────► player (Player), arena (ArenaBounds)
player ─────► arena, combat (Health, Hitbox, Team, PlayerDied), pickups (EffectsChanged), waves (WaveStarted)
weapons ────► player (FireRequested), combat (Projectile, Hitbox, Team, ShotId), pickups (EffectsChanged), arena
combat ─────► weapons (ShotFired, Delivery), pickups (EffectsChanged)
enemies ────► player (Player), combat (Health, Hitbox, Team, Hit), arena, waves (WaveSpec, WaveStarted, WaveCleared, WaveFailed)
pickups ────► combat (Hitbox, EnemyKilled, HealGranted, PlayerDied), player (Player), waves (WaveFailed)
waves ──────► flow_director (Difficulty, DifficultyAdjusted, WaveLevers), combat, enemies, player, weapons, pickups (messages only)
flow_director ► waves (WaveReport)
telemetry ──► waves, flow_director, combat, player, weapons, pickups (messages only)
debug_render ► player, enemies, combat, pickups (markers + Hitbox), weapons (ShotFired), arena
```

## Messages

All cross-plugin facts are `#[derive(Message)]`, written with `MessageWriter`
and read with `MessageReader`. There are no observers in this slice.

| Message | Owner (api) | Written by | Read by |
|---|---|---|---|
| `FireRequested` | player | player `request_fire` | weapons `fire` |
| `PlayerSpawned` | player | player `spawn_player`, `respawn_on_wave_start` | waves `record_director`, `track_hp`, telemetry |
| `WeaponSwitched` | weapons | weapons `announce_initial`, `apply_selection` | telemetry |
| `ShotFired` | weapons | weapons `fire` | combat `resolve_instant_shots`, waves `measure`, debug_render `draw_flashes` |
| `Hit` | combat | combat `detect_projectile_hits`, `resolve_instant_shots`; enemies `contact_damage` | combat `apply_hits`, waves `measure` (shots_hit) |
| `HealGranted` | combat | pickups `collect` | combat `apply_heals` |
| `PlayerHealed` | combat | combat `apply_heals` | waves `track_hp`, telemetry |
| `PlayerDamaged` | combat | combat `apply_hits` | waves `track_hp`, `measure`, telemetry |
| `PlayerDied` | combat | combat `apply_hits` | player `despawn_on_death`, waves `measure`, pickups `clear_on_death` |
| `EnemyKilled` | combat | combat `apply_hits` | waves `measure`, pickups `drop_on_kills`, telemetry |
| `EnemySpawned` | enemies | enemies `spawn_from_queue` | waves `measure` |
| `PickupCollected` | pickups | pickups `collect` | waves `measure`, telemetry |
| `EffectsChanged` | pickups | pickups `tick_effects` | player, weapons, combat (`track_effects`), telemetry |
| `WaveStarted` | waves | waves `advance` | enemies `queue_wave`, player `respawn_on_wave_start`, telemetry |
| `WaveCleared` | waves | waves `advance` | enemies `clear_on_wave_end` |
| `WaveFailed` | waves | waves `advance` | enemies `clear_on_wave_end`, pickups `clear_on_death` |
| `WaveReport` | waves | waves `advance` | flow_director `adjust_after_wave`, telemetry |
| `DifficultyAdjusted` | flow_director | flow_director `announce_initial` (Startup), `adjust_after_wave` | waves `record_director`, telemetry |

Notes:
- Weapons decide *that* a shot happens (cooldown, lock, damage after pickup
  effects) and write `ShotFired`. Combat decides *what it hits*: projectile
  overlaps each tick, hitscan rays and melee arcs straight from `ShotFired`.
- `Hit` is the single path for damage and `HealGranted` the single path for
  healing; only combat changes `Health`.
- Each shot carries a `ShotId`, so a melee swing that hits three enemies counts
  as one landed shot in accuracy.
- Pickup effects are one combined `Effects` value. Player, weapons and combat
  each keep a private copy of the latest `EffectsChanged` and use only their
  own field (move speed; fire rate, outgoing damage, lock; incoming damage).

## Schedules

- `Startup`: camera, arena border, player spawn, overlay text (debug builds),
  initial `DifficultyAdjusted` and `WeaponSwitched`.
- `Update` (variable): input sampling (player movement/aim/fire, weapons 1/2/3),
  camera follow, debug_render boxes and shot flashes, telemetry.
- `FixedUpdate` at 60 Hz: all simulation, in chained `SimSet`s:

| Order | SimSet | Systems |
|---|---|---|
| 1 | `Intent` | player `track_effects`, `request_fire`; weapons `track_effects`, `apply_selection`; combat `track_effects` |
| 2 | `Spawn` | weapons `fire`; enemies `queue_wave` then `spawn_from_queue`; pickups `drop_on_kills`, `age_pickups` |
| 3 | `Movement` | player `move_player`, enemies `chase_player`, weapons `move_projectiles` |
| 4 | `Detect` | combat `detect_projectile_hits`, `resolve_instant_shots`; enemies `contact_damage`; pickups `collect` |
| 5 | `Resolve` | combat `apply_heals` then `apply_hits` |
| 6 | `Progress` | waves `record_director` then `track_hp` then `measure` then `advance` |
| 7 | `Direct` | flow_director `adjust_after_wave` |
| 8 | `Cleanup` | arena `despawn_outside`, enemies `clear_on_wave_end`, player `despawn_on_death` then `respawn_on_wave_start`, pickups `clear_on_death` then `tick_effects` |

A message written by a later set is read by an earlier set on the next tick.
Bevy only drops messages after `FixedUpdate` has run, so none are missed.

## Wave state machine (waves)

`Idle -> Spawning -> Active -> Cleared -> Intermission -> Spawning -> ...`

- `Idle`: waiting for the first `DifficultyAdjusted`.
- `Spawning`: enemies still being spawned. Player death ends the wave.
- `Active`: all enemies spawned; ends when all are killed or the player dies.
- `Cleared(outcome)`: one tick. Writes `WaveCleared` or `WaveFailed`, then `WaveReport`.
- `Intermission`: fixed pause, then the next wave starts with the latest levers.
  After a failed wave the same index is retried (`attempt + 1`).

It is a resource driven in `FixedUpdate`, not Bevy `States`, so transitions
happen on simulation ticks instead of frame boundaries.

## Flow director

`flow_director::decide(current, &WaveReport, memory, &config) -> Decision` is
pure and unit-tested (`cargo test`). The system around it only reads
`WaveReport`, stores `DirectorMemory`, and writes `DifficultyAdjusted`.
`flow_director::levers_for(difficulty)` maps difficulty to the three v1
levers: enemy count, enemy speed, enemy contact damage. `Difficulty` counts
quarter steps from 1.0 to 10.0. The decision reads the wave's risk
(`wave_risk`: share of the wave's starting hp lost, 1 on death), smooths it in
`DirectorMemory`, and steps down faster than up. The decision applies
to the next wave only, because waves reads the levers when that wave starts.

## Playtest bots (playtest)

Not a gameplay plugin: `FlowArenaPlugins` does not include it. `playtest::play`
builds a headless app (`MinimalPlugins`, `InputPlugin`, every gameplay plugin
except debug_render, a fixed 1/60 s clock) and adds `PlaytestBotPlugin`, which
runs `perceive` then `act` in `SimSet::Intent`. The bot only uses what a human
has: it presses keys in `ButtonInput<KeyCode>` for movement and weapon choice,
and writes `FireRequested` as the mouse would. It sees the world through a delay
line of snapshots (reaction time). Pure parts are split out and unit-tested:
`steering` (dodge, wall push, eight-way snapping), `choices` (pickup judgement,
weapon choice), `perception` (delay line, deterministic rng) and `summary`.

`playtest::watch` is the windowed counterpart of `play`: `DefaultPlugins`, all
of `FlowArenaPlugins` (debug_render included), the same `PlaytestBotPlugin` at
real-time speed, and a small bottom-left label (tier, weapon, next difficulty,
last director reason) registered by its own `WatchLabelPlugin`. It sets `SessionFileEnabled(false)`, so watching a
bot never writes a session file that would later read as your own.
With `WatchConfig::quadrant` set, the window starts hidden and frameless and
`place_in_quadrant` moves it into that quarter of the primary monitor's work
area once winit reports the monitor (`tiling` splits the area, pure + tests;
`work_area` asks Windows for it and falls back to the whole monitor
elsewhere). A tile sets `OverlayEnabled(false)` and its label shrinks to
tier, wave and difficulty. The example's `--watch-all` starts one such process per tier.

## Session record (telemetry)

Telemetry also builds a `telemetry::api::SessionRecord` in `Update`: one
`WaveRecord` per finished wave with the `WaveReport`, the director's decision,
a `WeaponTally` per weapon (time held, shots, hits, kills by killing blow,
damage taken and deaths while held) and every `PickupTaken` (time into wave,
hp, weapon). When a `PrimaryWindow` exists and `SessionFileEnabled` is true (the
default), `record_file` appends each wave as
a JSON line to `playtests/session-<unix time>.jsonl`; headless apps write
nothing. The playtest report reads the same record from bot runs and, with
`--human`, from those files.

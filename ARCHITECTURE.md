# flow_arena architecture

Single crate, one module per domain. Each domain module has a `Plugin` and an
`api` submodule. Other domains may use only `<domain>::api` items (and the
plugin type). Everything else is private to the domain.

## Plugin graph

Arrows mean "uses the public api of". No domain touches another's private items.

```
main.rs ── FlowArenaPlugins (lib.rs)
             │
app_setup ◄──┴── every gameplay plugin (SimSet ordering)

player ─────► arena (ArenaBounds), combat (Health, Hitbox, Team, PlayerDied), waves (WaveStarted)
weapons ────► player (FireRequested), combat (Projectile, Hitbox, Team), arena (DespawnOutsideArena)
enemies ────► player (Player), combat (Health, Hitbox, Team, Hit), arena (ArenaBounds), waves (WaveSpec, WaveStarted, WaveCleared, WaveFailed)
combat ─────► (nothing)
waves ──────► flow_director (Difficulty, DifficultyAdjusted, WaveLevers), combat, enemies, player, weapons (messages only)
flow_director ► waves (WaveReport)
telemetry ──► waves, flow_director, combat, player (messages only)
debug_render ► player (Player), enemies (Enemy), combat (Projectile, Hitbox), arena (ArenaBounds)
arena ──────► (nothing)
```

## Messages

All cross-plugin facts are `#[derive(Message)]`, written with `MessageWriter`
and read with `MessageReader`. There are no observers in this slice.

| Message | Owner (api) | Written by | Read by |
|---|---|---|---|
| `FireRequested` | player | player `request_fire` | weapons `fire` |
| `PlayerSpawned` | player | player `spawn_player`, `respawn_on_wave_start` | waves `record_director`, telemetry `collect` |
| `ShotFired` | weapons | weapons `fire` | waves `measure` |
| `Hit` | combat | combat `detect_projectile_hits`, enemies `contact_damage` | combat `apply_hits`, waves `measure` (shots_hit) |
| `PlayerDamaged` | combat | combat `apply_hits` | waves `measure`, telemetry `collect` |
| `PlayerDied` | combat | combat `apply_hits` | player `despawn_on_death`, waves `measure` |
| `EnemyKilled` | combat | combat `apply_hits` | waves `measure`, telemetry `collect` |
| `EnemySpawned` | enemies | enemies `spawn_from_queue` | waves `measure` |
| `WaveStarted` | waves | waves `advance` | enemies `queue_wave`, player `respawn_on_wave_start`, telemetry `collect` |
| `WaveCleared` | waves | waves `advance` | enemies `clear_on_wave_end` |
| `WaveFailed` | waves | waves `advance` | enemies `clear_on_wave_end` |
| `WaveReport` | waves | waves `advance` | flow_director `adjust_after_wave`, telemetry `collect` |
| `DifficultyAdjusted` | flow_director | flow_director `announce_initial` (Startup), `adjust_after_wave` | waves `record_director`, telemetry `collect` |

Notes:
- The spec listed combat as a `ShotFired` reader. In this slice the projectile
  entity carries its own damage, so combat does not need it; waves reads it to
  count `shots_fired`.
- `Hit` is the single path for damage: projectile overlaps and enemy contact
  both produce `Hit`, and only `combat::apply_hits` changes `Health`.

## Schedules

- `Startup`: camera, arena border, player spawn, overlay text, initial
  `DifficultyAdjusted`.
- `Update` (variable): `player::sample_input` (keyboard, mouse, cursor to world),
  `debug_render::attach_boxes`, telemetry `collect` then `render_overlay`.
- `FixedUpdate` at 60 Hz: all simulation, in chained `SimSet`s:

| Order | SimSet | Systems |
|---|---|---|
| 1 | `Intent` | player `request_fire` |
| 2 | `Spawn` | weapons `fire`; enemies `queue_wave` then `spawn_from_queue` |
| 3 | `Movement` | player `move_player`, enemies `chase_player`, weapons `move_projectiles` |
| 4 | `Detect` | combat `detect_projectile_hits`, enemies `contact_damage` |
| 5 | `Resolve` | combat `apply_hits` |
| 6 | `Progress` | waves `record_director` then `measure` then `advance` |
| 7 | `Direct` | flow_director `adjust_after_wave` |
| 8 | `Cleanup` | arena `despawn_outside`, enemies `clear_on_wave_end`, player `despawn_on_death` then `respawn_on_wave_start` |

A message written by a later set is read by an earlier set on the next tick.
Bevy only drops messages after `FixedUpdate` has run, so none are missed.

## Wave state machine (waves)

`Idle -> Spawning -> Active -> Cleared -> Intermission -> Spawning -> ...`

- `Idle`: waiting for the first `DifficultyAdjusted`.
- `Spawning`: enemies still being spawned. Player death ends the wave.
- `Active`: all enemies spawned; ends when all are killed or the player dies.
- `Cleared(outcome)`: one tick. Writes `WaveCleared` or `WaveFailed`, then `WaveReport`.
- `Intermission`: fixed pause, then the next wave starts with the latest levers.

It is a resource driven in `FixedUpdate`, not Bevy `States`, so transitions
happen on simulation ticks instead of frame boundaries.

## Flow director

`flow_director::decide(current, &WaveReport, memory, &config) -> Decision` is
pure and unit-tested (`cargo test`). The system around it only reads
`WaveReport`, stores `DirectorMemory`, and writes `DifficultyAdjusted`.
`flow_director::levers_for(difficulty)` maps difficulty to the three v1
levers: enemy count, enemy speed, enemy contact damage. The decision applies
to the next wave only, because waves reads the levers when that wave starts.

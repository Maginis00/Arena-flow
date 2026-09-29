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
| `PlayerSpawned` | player | player `spawn_player`, `respawn_on_wave_start` | waves `record_director`, telemetry |
| `WeaponSwitched` | weapons | weapons `announce_initial`, `apply_selection` | telemetry |
| `ShotFired` | weapons | weapons `fire` | combat `resolve_instant_shots`, waves `measure`, debug_render `draw_flashes` |
| `Hit` | combat | combat `detect_projectile_hits`, `resolve_instant_shots`; enemies `contact_damage` | combat `apply_hits`, waves `measure` (shots_hit) |
| `HealGranted` | combat | pickups `collect` | combat `apply_heals` |
| `PlayerHealed` | combat | combat `apply_heals` | telemetry |
| `PlayerDamaged` | combat | combat `apply_hits` | waves `measure`, telemetry |
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
| 6 | `Progress` | waves `record_director` then `measure` then `advance` |
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
levers: enemy count, enemy speed, enemy contact damage. The decision applies
to the next wave only, because waves reads the levers when that wave starts.

# flow_arena

A 2D top-down wave-arena prototype. It exists to test a flow-channel difficulty
director: after each wave, the director nudges difficulty so the next wave sits
slightly above recent player performance.

This is a test harness, not a game.

## Run

Requires stable Rust 1.95 or newer (bevy 0.19.1's minimum).

```sh
cargo run            # play
cargo test           # decide() unit tests + a headless multi-wave run
```

On Linux, Bevy needs the usual system packages (for example on Debian or
Ubuntu: `libudev-dev libwayland-dev libxkbcommon-dev`, plus X11 dev headers).

## Controls

- `W` `A` `S` `D`: move
- Mouse: aim
- Hold left mouse button: fire

## What you see

- Cyan box: player. Red boxes: enemies. Yellow boxes: projectiles.
  Dark gray border: arena bounds.
- Top-left debug overlay: wave index, difficulty (current and next), hp, kills,
  wave time, the director's last decision reason, and the last 5 wave reports.

Waves are endless. Dying fails the wave; the player respawns at centre when
the next wave starts after the intermission.

## How difficulty moves

After each wave the director classifies the report:

- cleared fast with high hp: +1
- cleared but close (low hp or slow), or unremarkable: hold
- player died: -1

Difficulty is clamped to 1..=10. Hysteresis: after a change, the next wave can
change difficulty again only if the same signal repeats. Difficulty drives
enemy count, enemy speed and enemy contact damage only. All thresholds and
curves are placeholder constants marked `PLACEHOLDER` in the source.

See [ARCHITECTURE.md](ARCHITECTURE.md) for plugins, messages and schedule order.

## Intentionally not built

- Any weapon beyond the placeholder projectile (weapon fantasy is undecided)
- Pickups (the green box color is reserved, but there is no pickup design yet)
- Camera follow, game over screen, run restart, menus beyond the debug overlay
- Assets, sprites, animation, sound, fonts beyond Bevy's default
- Upgrades, shops, bosses, procedural rooms, networking, save games
- Balance: every number is a placeholder

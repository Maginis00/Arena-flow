# flow_arena

A 2D top-down wave-arena prototype. It exists to test a flow-channel difficulty
director: after each wave, the director nudges difficulty so the next wave sits
slightly above recent player performance.

This is a test harness, not a game.

## Run

Requires stable Rust 1.95 or newer (bevy 0.19.1's minimum).

```sh
cargo run            # play (debug build: shows the telemetry overlay)
cargo run --release  # play without the overlay
cargo test           # unit tests + headless multi-wave runs (idle, aim-bot, playtest bots)
cargo run --example playtest                    # playtest report for every skill tier
cargo run --example playtest -- --watch --tier expert   # watch one bot play
```

The debug build is the everyday profile: our code at `opt-level = 1`,
dependencies at `opt-level = 3`, so the game runs smoothly. `--release` is a
second full Bevy build; skip it unless you need it.

Building Bevy is memory hungry. `.cargo/config.toml` caps cargo at 3 parallel
jobs and the dev profile keeps only line tables as debug info, so a full build
fits on a 16 GB machine. Raise `jobs` there if you have more memory.

On Linux, Bevy needs the usual system packages (for example on Debian or
Ubuntu: `libudev-dev libwayland-dev libxkbcommon-dev`, plus X11 dev headers).

## Controls

- `W` `A` `S` `D`: move
- Mouse: aim
- Hold left mouse button: fire
- `1` `2` `3`: projectile, hitscan, melee arc

## Weapons (all numbers placeholder)

| Key | Weapon | Cooldown | Damage | Reach |
|---|---|---|---|---|
| 1 | Projectile | 0.18 s | 1 | travels at 720/s |
| 2 | Hitscan | 0.35 s | 2 | first enemy on a 520 ray |
| 3 | Melee arc | 0.45 s | 3 | every enemy in a 120° arc, radius 75 |

Enemies have 3 hp.

## Pickups (all numbers placeholder)

Every 5th kill drops a pickup where the enemy died. It lasts 8 s on the floor.
Every pickup has a cost:

| Shade of green | Pickup | Upside | Downside | Duration |
|---|---|---|---|---|
| Bright | Overdrive | fire rate x1.6 | take x1.5 damage | 8 s |
| Dark | Heavy | damage x2 | move speed x0.7 | 8 s |
| Pale | Mend | heal 30 hp | weapon locked | 2.5 s |

Different pickups stack. Taking the same pickup again resets its timer instead
of stacking. Dying clears all effects.

## What you see

- Cyan box: player. Red boxes: enemies. Yellow: projectiles, hitscan tracers,
  melee swings. Green: pickups. Dark gray border: arena bounds.
- The camera follows the player lightly and stops at the arena edge.
- Debug builds only: the top-left overlay shows wave and attempt, difficulty
  (current and next), hp, kills, wave time, weapon, active effects, the
  director's last decision reason, and the last 5 wave reports.

Waves are endless. Dying fails the wave. After the intermission the player
respawns at the centre and the same wave is retried, at whatever difficulty the
director chose.

## How difficulty moves

After each wave the director reads its risk: the share of the hp the player
brought into the wave that the wave took away (1 if the player died). Hp
carries over between waves, so earlier damage does not count again. The risk is
smoothed over recent waves, then:

- player died: -1
- smoothed risk above the band: -0.5
- inside the band: hold
- below the band: +0.25

Difficulty runs from 1.0 to 10.0 in quarter steps. Down steps are bigger than up
steps on purpose: overshooting into danger costs more than staying a little
safe. Difficulty drives enemy count, enemy speed and enemy contact damage only,
interpolated between whole levels. All thresholds and curves are placeholder
constants marked `PLACEHOLDER` in the source.

## Playtest bots

`cargo run --example playtest` runs four simulated players
(novice, casual, skilled, expert) through the real game plugins, headless and
deterministic, and prints a table of where the flow director settled each one:
waves, deaths, settled difficulty, share of decisions held in band, share of
waves whose own risk was in band, median risk, hit rate, clear speed, hp lost and pickups taken. A second table splits the
run by weapon: share of wave time held, kills (credited to the weapon that
landed the killing blow), kills per landed hit, hit rate, and damage taken and
deaths while holding it. A third shows each pickup kind: how often it was
taken, the hp it was taken at and how far into the wave. Options: `--minutes N` (simulated, default 10), `--seeds N` (varies aim
wobble), `--tier NAME` (repeatable), `--weapon NAME` (locks the bot to one
weapon, repeatable; with `--watch` the first one is used), `--matrix` (every tier with its own choice and locked to
each weapon), `--human` (adds your own sessions, see below), `--human-only`.

### Your own sessions in the report

When you play in a window (`cargo run` or `cargo run --release`), the game
writes one file per launch to `playtests/session-<unix time>.jsonl` in the
folder you started it from (gitignored). Each finished wave adds one line:
the wave report, the director's decision, per-weapon numbers, and every
pickup with the time into the wave, your hp and your weapon when you took it.
Then `cargo run --example playtest -- --human` puts your sessions
("you: session-...") in the same tables as the bots, or `--human-only` shows
just yours.

Bots move with `W` `A` `S` `D`, switch weapons with `1` `2` `3` and fire the same
intent the mouse does. Tiers differ only in measurable limits: reaction delay,
decision rate, aim error, dodge radius, strafing, wall awareness, trigger
discipline, pickup judgement and weapon choice. All tier numbers are
PLACEHOLDER, in `src/playtest/tier.rs`.

### Watching a bot

`--watch` opens the normal game window and lets one bot play in real time, so
you can see how a tier actually moves, aims and picks up. The first `--tier`
plays (default expert), `--seeds N` picks the seed, and `--weapon projectile`,
`hitscan` or `melee` pins one weapon instead of the tier's own choice. The
bottom-left label shows the bot, its weapon, the next difficulty and the
director's last reason; in the debug build the usual overlay is there too.

```sh
cargo run --example playtest -- --watch --tier novice
cargo run --example playtest -- --watch --tier casual
cargo run --example playtest -- --watch --tier skilled --weapon melee
cargo run --example playtest -- --watch --tier expert --seeds 2
```

Your own keys and mouse still work in that window and mix with the bot's
input, so keep hands off to see the bot alone. The simulation runs on the same
fixed 60 Hz tick as the headless report, but input is sampled per frame, so a
watched run is not tick-for-tick identical to the headless one.

See [ARCHITECTURE.md](ARCHITECTURE.md) for plugins, messages and schedule order.

## Intentionally not built

- Game over screen, run restart, menus beyond the debug overlay
- A release-build HUD (hp, active effects): nothing is shown without the overlay
- Assets, sprites, animation, sound, fonts beyond Bevy's default
- Upgrades, shops, bosses, procedural rooms, networking, save games
- Balance: every number is a placeholder

See [CLAUDE.md](CLAUDE.md) for the design north star, the process and the code conventions.

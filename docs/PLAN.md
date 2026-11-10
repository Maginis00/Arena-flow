# PLAN Arena-flow

> **Entwurf Ist-Stand, V1-Plan folgt mit Planer.** Angelegt 2026-11-10 vom Projekt-Bot aus dem Repo-Stand main = #22.
> Enthält nur, was im Repo belegt ist. Ziele, Nicht-Ziele und Slices für V1 sind **nicht entschieden** (OP-001).
> Miro-Link: noch keiner.

## Ziel (laut README, Stand heute)
- 2D-Top-down-Wave-Arena als **Testaufbau für einen Flow-Channel-Difficulty-Director**: nach jeder Welle schiebt der Director die Schwierigkeit so, dass die nächste Welle knapp über der jüngsten Spielerleistung liegt (README.md:3–5).
- „This is a test harness, not a game.“ (README.md:7)
- Leitbild aus CLAUDE.md:18–27: „Simple on the surface, deep in mechanics and optimization“, Referenz Devil Daggers; der Director soll das Spiel für jede Skill-Stufe fordernd halten; Pickups haben immer einen Preis.
- Spannung zwischen „Testaufbau“ (README) und Spiel-Leitbild (CLAUDE.md) ist die Kernfrage für V1 → OP-001.

## Erkennbare Nicht-Ziele (README.md:171–177, „Intentionally not built“)
- Game-Over-Screen, Run-Restart, Menüs außer dem Debug-Overlay
- Release-HUD (hp, aktive Effekte)
- Assets, Sprites, Animation, Fonts über Bevys Standard (Sound gibt es inzwischen hinter `fx`, siehe OP-013)
- Upgrades, Shops, Bosse, prozedurale Räume, Netzwerk, Spielstände
- Balance: jede Zahl ist Platzhalter
Ob diese Nicht-Ziele für V1 gelten, ist offen (OP-001, OP-017).

## Architektur (kurz, Details in ARCHITECTURE.md)
- Ein Crate `flow_arena`, Bevy `=0.19.1` (2D + UI), Rust 1.95, Edition 2024 (Cargo.toml, CLAUDE.md „Architecture rules“).
- Ein Plugin je Domäne; die Root-Datei `src/<domäne>.rs` ist die öffentliche Oberfläche, Kommunikation nur über `#[derive(Message)]`-Fakten. Simulation in `FixedUpdate` nach `SimSet`, deterministisch, kein RNG.
- Domänen: `app_setup`, `arena`, `camera`, `player`, `weapons`, `combat`, `enemies` (6 Arten: Grunt, Shooter, Brute, Charger, Summoner, Swarm), `pickups` (Regelwerke Classic/Far/Shards/ShardsRange/ShardsBolts), `waves` (inkl. `danger`), `flow_director` (`decide`, `curve`, `difficulty`), `telemetry` (Overlay, Session-Dateien), `debug_render`, `fx` (Feature, standardmäßig aus).
- `playtest` (kein Gameplay-Plugin): Headless-Bots in 4 Stufen (novice/casual/skilled/expert), Zuschau-Fenster, Strategie-Suche.
- Beispiele: `examples/playtest.rs` (Report, `--watch`, `--watch-all`, `--matrix`, `--human`), `examples/strategy_search.rs` (PR #19), `examples/agent_play.rs` (schrittweises Spielen durch einen externen Agenten, PR #10).
- Tests: Unit-Tests in den Dateien, Integration in `tests/headless_waves.rs`, `tests/playtest_tiers.rs`, `tests/session_file.rs`.
- CI: nur `.github/workflows/build-cache.yml` (Build-Cache für Claude-Cloud-Sessions), keine Test-CI (OP-014).

## Entscheidungen (nur im Repo belegt)
| Datum | Entscheidung | Entscheider | Quelle |
|---|---|---|---|
| 2026-10-02 | Treffer-Schonzeit nach Schaden (`GRACE_SECS` 0,75 s) | Jannus | PR #12 („Jannus's choice“) |
| 2026-10-02 | Geschaffte Welle füllt hp wieder auf | Jannus | PR #12 („Jannus's choice“) |
| 2026-10-03 | Sound/Partikel behalten, aber getrennt: Feature `fx`, standardmäßig aus; Bots/Headless laden es nie | Jannus | playtest-notes/2026-10-03-sound-und-partikel.md:12–14, :31–34; CLAUDE.md „Architecture rules“ |
| 2026-10-03 | Schwert auf Rechtsklick wird Standard (`ARENA_SWORD=key3` = alte Belegung) | Jannus (Playtest-Urteil „gefällt“), Default umgesetzt und gemergt in PR #22 | playtest-notes/2026-10-03-sound-und-partikel.md:15, PR #22 |
| 2026-11-10 | Erst Ist-Stand in STAND.md/OP.md festhalten, dann mit dem Planer-Bot einen Plan bis V1 | Jannus | Chat 2026-11-10 (OP-001) |
Alle Balance-Zahlen sind Platzhalter, keine davon ist entschieden (OP-005, OP-006).

## Slice-Kandidaten (Vorschlag, nicht entschieden)
Reihenfolge und Umfang klärt der Planer-Bot mit Jannus. Modell/Effort wählt der Leiter per Auswahlbox beim Slice-Start. Aufwände sind Schätzungen.

**S-01 · README an den Code angleichen** · Ziel: README beschreibt den Ist-Stand, damit Planer und Playtests nicht von veralteten Angaben ausgehen · Abhängigkeiten: keine
Was: 1. Gegner-hp-Satz (README.md:52) durch Verweis auf die Arten ersetzen. 2. `ARENA_ENEMIES` (grunt, Artname, all) und `ARENA_PICKUPS` (classic, far, shards, shards-range, shards-bolts) dokumentieren. 3. `cargo play` / Feature `fx` und Taste M (stumm) aufnehmen; Sound aus „Intentionally not built“ (README.md:175) nehmen. 4. `strategy_search` und `agent_play` kurz erwähnen.
Pfade: README.md
Fertig wenn: jede genannte Env-Variable und jedes Beispiel existiert im Code; keine Inhaltsentscheidung neu getroffen; `cargo test` unverändert grün.
Nicht-Ziele: Code, Werte, neue Features. · Test: `cargo test` · Aufwand: ca. 5–10 min, S · OPs: OP-013

**S-02 · Test-CI für PRs** · Ziel: fmt, clippy und Tests laufen automatisch auf jedem PR · Abhängigkeiten: OK von Jannus (OP-014)
Was: 1. Workflow, der `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` ausführt. 2. Den vorhandenen Build-Cache (Release-Assets `build-cache`) nutzen, damit Bevy nicht jedes Mal neu baut.
Pfade: .github/workflows/ (neue Datei), ggf. Hinweis in README.md
Fertig wenn: Workflow läuft auf einem Test-PR grün; ein absichtlicher clippy-Fehler macht ihn rot.
Nicht-Ziele: Release-Builds, Deployment. · Test: CI-Lauf auf dem PR · Aufwand: ca. 10–15 min, S · OPs: OP-014

**S-03 · Design-Unterlagen ins Repo** · Ziel: Flow-Kurve, Gegner-, Pickup- und Flow-Bericht sowie Playtest-Anleitung für alle Bots lesbar machen · Abhängigkeiten: Jannus liefert die Dateien (OP-012)
Was: 1. Dateien nach `docs/` (bzw. `design/` für die Kurve, wie im Code referenziert) übernehmen. 2. Verweise in README.md:88, src/flow_director/curve.rs:1 und den Playtest-Notizen prüfen.
Pfade: docs/, design/, ggf. Verweis-Zeilen
Fertig wenn: jeder Verweis im Repo zeigt auf eine vorhandene Datei.
Nicht-Ziele: Inhalte umschreiben. · Test: `rg` auf die Pfade · Aufwand: ca. 5 min, S · OPs: OP-012

**S-04 · Standard-Gegnermix festlegen** · Ziel: `cargo run` startet mit dem Mix, den Jannus spannend findet, statt nur Grunts · Abhängigkeiten: OP-001, OP-002
Was: 1. Default von `EnemyMix` laut Entscheidung OP-002 setzen. 2. Bot-Report vorher/nachher anhängen. 3. README anpassen.
Pfade: src/enemies/kinds.rs:52–70, README.md
Fertig wenn: `cargo run` ohne Env nutzt den neuen Mix; `ARENA_ENEMIES=grunt` liefert das alte Verhalten; Tests grün.
Nicht-Ziele: Werte der Arten tunen (OP-005). · Test: `cargo test` + `cargo run --example playtest` · Aufwand: ca. 5–10 min, S · OPs: OP-002

**S-05 · Pickup-Regelwerk eingrenzen** · Ziel: ein klares Pickup-System statt fünf Prototypen · Abhängigkeiten: OP-001, OP-003 (und OP-010)
Was: 1. Gewähltes Regelwerk als Default. 2. Verworfene Varianten samt Bot-Pfaden entfernen. 3. README/ARCHITECTURE anpassen.
Pfade: src/pickups/ (rules.rs, shards.rs, placement.rs, table.rs), src/playtest/choices.rs, examples/strategy_search.rs, README.md, ARCHITECTURE.md
Fertig wenn: nur die gewählten Regelwerke sind auswählbar; `strategy_search` und Tests laufen; Report für Default unverändert, falls Default gleich bleibt.
Nicht-Ziele: neue Pickup-Arten. · Test: `cargo test` · Aufwand: ca. 15–25 min, M · OPs: OP-003, OP-010 · Risiko nach Urteil: Lösch-Code (Risikoliste Nr. 2)

**S-06 · Playtest-Vorlage** · Ziel: menschliche Playtests vergleichbar machen (Lust vorher, feste Fragen) · Abhängigkeiten: keine (Anleitung aus OP-012 hilft)
Was: 1. `playtest-notes/VORLAGE.md` mit Feldern: Datum, Dauer, Build/Env, Lust vorher 1–10, Flow 1–10, Waffe, Gegnermix, Session-Log ja/nein, Aussagen-Tabelle wie in den bisherigen Notizen.
Pfade: playtest-notes/VORLAGE.md
Fertig wenn: Vorlage deckt alle Spalten der beiden bisherigen Notizen ab.
Nicht-Ziele: Code. · Test: keiner (Doku) · Aufwand: ca. 5 min, S · OPs: OP-011

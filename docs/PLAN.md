# PLAN Arena-flow

> **Entwurf Ist-Stand, V1-Plan folgt mit Planer.** Angelegt 2026-11-10 vom Projekt-Bot aus dem Repo-Stand main = #22, Nachtrag Entscheidungen 2026-11-10.
> Enthält nur, was im Repo oder im Chat mit Jannus belegt ist. Richtung für V1 entschieden (OP-001), Slices für V1 sind **nicht entschieden**.
> Miro-Link: noch keiner.

## Ziel (laut README, Stand heute)
- 2D-Top-down-Wave-Arena als **Testaufbau für einen Flow-Channel-Difficulty-Director**: nach jeder Welle schiebt der Director die Schwierigkeit so, dass die nächste Welle knapp über der jüngsten Spielerleistung liegt (README.md:3–5).
- „This is a test harness, not a game.“ (README.md:7)
- Leitbild aus CLAUDE.md:18–27: „Simple on the surface, deep in mechanics and optimization“, Referenz Devil Daggers; der Director soll das Spiel für jede Skill-Stufe fordernd halten; Pickups haben immer einen Preis.
- Entscheidung Jannus 2026-11-10 (Box, OP-001): V1 ist erst der fertige Testaufbau für den Flow-Director, danach wird ein Spiel daraus.

## Kernidee: Flow-Modell (Quelle: Chat Jannus 2026-11-10)
Ersetzt inhaltlich die fehlende `design/risk-flow-curve.png` (OP-012).
- Klassischer Flow-Graph: x-Achse Skill, y-Achse Challenge, der Flow-Kanal verläuft diagonal nach oben rechts.
- Man ist im Flow, wenn die Challenge knapp über dem Skill liegt.
- Skill hat 3 Bereiche: **mikro** (Zielen und Bewegen), **meso** (Entscheidungen innerhalb einer Welle), **makro** (Strategie über viele Wellen).
- Der Director arbeitet wie ein komplexes System, passt sich automatisch an den Spielertyp an und hält ihn möglichst lange im Kanal (schwerer, aber nicht zu schwer).

### Ist-Stand im Code (geprüft 2026-11-10, main = #23)
- **Director misst bisher nur mikro**, und zwar indirekt: `wave_risk` = Anteil der hp, den eine Welle kostet (1 bei Tod), geglättet und in ein Band gesteuert (src/flow_director/decide.rs:76–82, :103–115). Der Code sagt selbst: liest nur mechanischen Druck, Entscheidungen und Gegner-Lesen nicht (src/flow_director/curve.rs:5–8). Die Code-Kurve ist Risiko → Engagement, kein Skill-/Challenge-Graph.
- **meso:** wird nur gemessen, nicht gesteuert. `waves::danger` zählt Beinahe-Treffer, Bedrohungs- und Ruhezeit, Gedränge (src/waves/danger.rs, ARCHITECTURE.md:222–226); Bot-Stufen haben Entscheidungs-Parameter wie Tells lesen, Prioritätsziel, Pickup- und Waffenwahl (src/playtest/tier.rs:148–176).
- **makro:** nur offline in den Bot-Werkzeugen: `strategy_search` prüft, ob eine Pickup-/Spend-Strategie überall gewinnt („macro difficulty as consequence“, src/playtest/strategy.rs:1–10). Der Director nutzt das nicht.
- **Spielertyp-Anpassung:** gibt es nicht; der Director führt einen Schwierigkeitswert 1–10 und eine geglättete Risikozahl (OP-019).
- Offene Fragen dazu: OP-008, OP-009, OP-019.

## Erkennbare Nicht-Ziele (README.md:171–177, „Intentionally not built“)
- Game-Over-Screen, Run-Restart, Menüs außer dem Debug-Overlay
- Release-HUD (hp, aktive Effekte)
- Assets, Sprites, Animation, Fonts über Bevys Standard (Sound gibt es inzwischen hinter `fx`, siehe OP-013)
- Upgrades, Shops, Bosse, prozedurale Räume, Netzwerk, Spielstände
- Balance: jede Zahl ist Platzhalter
Nach OP-001 (V1 = fertiger Testaufbau, Spiel danach) gehören Game-Over, HUD und Assets eher nach V1; endgültig legt das der V1-Plan fest (OP-016, OP-017).

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
| 2026-11-10 | V1 ist erst der fertige Testaufbau für den Flow-Director, danach wird ein Spiel daraus | Jannus | Box 2026-11-10 (OP-001) |
| 2026-11-10 | Standard-Gegnermix zu Beginn Grunt + Charger + Swarm, weitere Arten in späteren Wellen; ab welcher Welle, legt der V1-Plan fest | Jannus | Chat 2026-11-10 (OP-002, Rest OP-018) |
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

**S-04 · Standard-Gegnermix Grunt + Charger + Swarm** · Ziel: `cargo run` startet mit dem Mix, den Jannus entschieden hat (OP-002), statt nur Grunts · Abhängigkeiten: OP-002 (entschieden); wellenabhängiges Dazukommen weiterer Arten erst nach OP-018
Was: 1. `EnemyMix` kann heute nur eine Zusatzart (`With`) oder alle (`All`); einen Mix Grunt + Charger + Swarm ergänzen und als Default setzen. 2. Bot-Report vorher/nachher anhängen. 3. README anpassen.
Pfade: src/enemies/kinds.rs:52–70, src/enemies/kinds.rs:180–230, README.md
Fertig wenn: `cargo run` ohne Env nutzt den neuen Mix; `ARENA_ENEMIES=grunt` liefert das alte Verhalten; Tests grün.
Nicht-Ziele: Werte der Arten tunen (OP-005), Wellen-Reihenfolge (OP-018). · Test: `cargo test` + `cargo run --example playtest` · Aufwand: ca. 10–15 min, S · OPs: OP-002, OP-018

**S-05 · Pickup-Regelwerk eingrenzen** · Ziel: ein klares Pickup-System statt fünf Prototypen · Abhängigkeiten: OP-003 (wartet auf OP-012, Tendenz Jannus: shards), OP-010
Was: 1. Gewähltes Regelwerk als Default. 2. Verworfene Varianten samt Bot-Pfaden entfernen. 3. README/ARCHITECTURE anpassen.
Pfade: src/pickups/ (rules.rs, shards.rs, placement.rs, table.rs), src/playtest/choices.rs, examples/strategy_search.rs, README.md, ARCHITECTURE.md
Fertig wenn: nur die gewählten Regelwerke sind auswählbar; `strategy_search` und Tests laufen; Report für Default unverändert, falls Default gleich bleibt.
Nicht-Ziele: neue Pickup-Arten. · Test: `cargo test` · Aufwand: ca. 15–25 min, M · OPs: OP-003, OP-010 · Risiko nach Urteil: Lösch-Code (Risikoliste Nr. 2)

**S-06 · Playtest-Vorlage** · Ziel: menschliche Playtests vergleichbar machen (Lust vorher, feste Fragen) · Abhängigkeiten: keine (Anleitung aus OP-012 hilft)
Was: 1. `playtest-notes/VORLAGE.md` mit Feldern: Datum, Dauer, Build/Env, Lust vorher 1–10, Flow 1–10, Waffe, Gegnermix, Session-Log ja/nein, Aussagen-Tabelle wie in den bisherigen Notizen.
Pfade: playtest-notes/VORLAGE.md
Fertig wenn: Vorlage deckt alle Spalten der beiden bisherigen Notizen ab.
Nicht-Ziele: Code. · Test: keiner (Doku) · Aufwand: ca. 5 min, S · OPs: OP-011

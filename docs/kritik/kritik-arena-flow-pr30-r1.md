# Kritik PR #30 (S-04 Standard-Gegnermix), Runde 1

Gegenstand: Branch feat/s-04, Commit f2fff2c (Basis b5d7bdc; main steht auf 3c5fc2f, nur Doku-Commit #26, mergeable clean). CI-Check „test“: success.
Gelesen: Diff, src/enemies/kinds.rs, src/enemies/spawning.rs, src/enemies.rs, examples/playtest.rs, examples/strategy_search.rs, src/playtest/agent.rs, src/playtest/session.rs, tests/playtest_tiers.rs, src/flow_director/decide.rs:143–152, PR-Body, docs/PLAN.md (Plan-Worktree). Keine Builds, kein Bot-Lauf nachgefahren.

## Urteil

ok mit Befunden. Kein Befund blockiert den Merge technisch; B1 sollte Jannus vor dem Merge kennen, weil er den „Standard-Mix“ inhaltlich anders macht, als der Name nahelegt (2/3 der Körper sind Swarm-Mitglieder).

## Nachrechnung Slot-Muster (Fokus 1)

`Slot::takes(slot) = slot >= first && (slot - first) % every == 0` (src/enemies/kinds.rs:182–184); `find` nimmt den ersten passenden Eintrag (kinds.rs:248), Swarm steht vorn.

- Swarm-Slots: 2, 6, 10, 14, 18, 22, … (2 + 4k)
- Charger-Slots: 4, 9, 14, 19, 24, … (4 + 5k)
- Kollision: 2 + 4a = 4 + 5b → Slot 14, dann alle kgV(4,5) = 20: 14, 34, 54 … Swarm gewinnt, Charger verliert jeden vierten Slot.
- Grenzanteile je Slot: Swarm 1/4 = 25 %, Charger 1/5 − 1/20 = 3/20 = 15 %, Grunt 60 %.

Wellengröße (Slots) = `enemy_count` = round(4 + 2·(Level−1)), also 4 (Level 1) bis 22 (Level 10) (src/flow_director/decide.rs:143–152, Tests :340–341). Ausgezählt per Skript:

| Slots n | Muster | G | C | W (Packs) | Körper (W×6) | Körper gesamt | Anteil Körper Swarm / Charger |
|---|---|---|---|---|---|---|---|
| 4 | GWGC | 2 | 1 | 1 | 6 | 9 | 67 % / 11 % |
| 6 | GWGCGW | 3 | 1 | 2 | 12 | 16 | 75 % / 6 % |
| 8 | GWGCGWGG | 5 | 1 | 2 | 12 | 18 | 67 % / 6 % |
| 10 | …GGCW | 5 | 2 | 3 | 18 | 25 | 72 % / 8 % |
| 14 | …GGGW (14 = Kollision) | 8 | 2 | 4 | 24 | 34 | 71 % / 6 % |
| 18 | | 11 | 2 | 5 | 30 | 43 | 70 % / 5 % |
| 22 | GWGCGWGGCWGGGWGGGWCGGW | 13 | 3 | 6 | 36 | 52 | 69 % / 6 % |

`SWARM_SIZE = 6` (src/enemies/spawning.rs:25, Schleife :140–151): ein Swarm-Slot erzeugt 6 Körper mit je 1 hp. Die Umsetzung entspricht der Beschreibung im PR und im Doc-Kommentar; der Test (kinds.rs:279–289) prüft die ersten 10 Slots exakt, die Kollision bei 14 nur indirekt.

## Befunde

### B1 – wichtig – Standard-Mix ist körpermäßig ein Swarm-Mix; Charger sind selten
Fundstelle: src/enemies/kinds.rs:229–242, src/enemies/spawning.rs:25, :140–151.
Pro Slot im Mittel 0,6 G + 0,15 C + 0,25·6 W = 2,25 Körper statt 1 bei `grunt`. Swarm stellt ~70 % aller Körper, Charger ~5–8 %. Zum Vergleich die Einzelmischungen: `With(Charger)` = Slot 2 jeder 3. (33 % der Slots), `With(Swarm)` = Slot 3 jeder 5. (20 %) (kinds.rs:188–196). Der Standard-Mix hat also weniger Charger als „grunts + charger“ und mehr Swarm als „grunts + swarm“. Laut OP-002 fand Jannus gerade die Charger spannend. Zugleich verliert der Director-Hebel `enemy_count` seine Bedeutung als Körperzahl (Faktor ~2,25), was den Sprung der settled difficulty (Expert 10,0 → 7,1) zu einem großen Teil mechanisch erklären dürfte (Annahme, nicht gemessen).
Vorschlag: Jannus die Tabelle oben zeigen und fragen, ob das Verhältnis gewollt ist; Optionen z. B. (a) so lassen, S-07 misst es, (b) Charger dichter / Swarm seltener (z. B. Charger 2+3k, Swarm 4+6k – nur Beispiel, PLACEHOLDER), (c) Kollisionsregel umdrehen (Charger vorn). Entscheidung nicht im PR treffen.
Wer: Jannus (Entscheidung), Executor (Umsetzung falls gewünscht).

### B2 – wichtig – Bot-Vergleich ist nur ein Richtungssignal, nicht nach Messvorschrift
Fundstelle: PR-Body (Tabelle), docs/PLAN.md:56 (Plan-Branch).
1 Seed, 10 min, ohne `--matrix`, nicht vom Code-Prüfer nachgefahren. Die Messvorschrift für S-07 verlangt `--matrix --seeds 3 --minutes 10`, gepoolt. Optionen vorher/nachher dürften gleich sein (nur der Default änderte sich, Pickups classic in beiden), das ist plausibel, aber nicht belegt (Kommandozeile für „vorher“ fehlt). Die Hit-Rate-Senkung (99 → 75 %) ist teilweise ein Zielgrößen-Effekt (Swarm half_size 7 statt 12, kinds.rs:154–159), kein reiner Schwierigkeitseffekt. Die Rangfolge hält (4,29 < 4,71 < 6,21 < 7,10), aber Novice–Casual liegen nur 0,42 auseinander → K1 mit mehreren Seeds gefährdet. Expert: deaths/wave 12 %, held in band 38 %.
Vorschlag: Vor dem Merge reicht es, das genaue Kommando für beide Läufe in den PR-Body zu schreiben (Reproduzierbarkeit). Die belastbare Messung ist S-07; ein zusätzlicher 3-Seed-Lauf vor dem Merge wäre doppelte Arbeit. Wichtig: den „vorher“-Report (`--enemies grunt`, gleiche Optionen) in S-07 mitlaufen lassen, sonst fehlt der Bezug.
Wer: Executor (PR-Body), Planer/Jannus (S-07-Umfang).

### B3 – wichtig – agent_play wechselt still auf den neuen Mix; alte Läufe replayen anders
Fundstelle: src/playtest/agent.rs:55–56 (nur `FlowArenaPlugins`, kein `insert_resource(EnemyMix)`), src/enemies.rs:64–69.
agent_play nimmt den Plugin-Default (bzw. `ARENA_ENEMIES` aus der Umgebung). Da ein Lauf bei jedem Aufruf aus der Order-Datei komplett neu abgespielt wird (examples/agent_play.rs:1–4) und die Datei den Mix nicht speichert (grep in agent.rs: kein „mix“/„EnemyMix“), spielen bestehende `runs/…`-Verzeichnisse nach dem Merge mit Standard-Mix weiter: Journal und Gefühls-Bewertungen passen nicht mehr zu den Wellen. Außerdem hängt der „deterministische“ Replay von einer Umgebungsvariable ab (vorbestehend, jetzt folgenreicher). Nicht erwähnt im PR.
Vorschlag: Im PR-Body/STAND erwähnen; optional (eigene Slice) den Mix beim `start` in die Run-Datei schreiben. Mindestens Jannus informieren, falls er agent_play-Läufe offen hat.
Wer: Executor (Hinweis), Jannus (ob Folge-Slice).

### B4 – wichtig – Kein headless Test läuft mit dem neuen Default
Fundstelle: tests/playtest_tiers.rs:26, :32, :59.
Alle Bot-Tests pinnen `Grunts` bzw. `All`; der Default des Spiels ist nirgends headless geprüft (Wellen enden, Director rankt Expert > Novice). Die Tests brechen nicht, sie decken aber das neue Spiel nicht ab. S-13 (Kriterien als Tests) kommt erst viel später.
Vorschlag: Einen kurzen Test `standard_mix_plays_through` (Skilled, `EnemyMix::Standard`, 4 min, waves ≥ 5, clears > 0) ergänzen; ob `settle()` auf Standard umgestellt wird, ist eine Messfrage (bricht die bisherige Referenz) → eher nicht in diesem PR.
Wer: Executor (Test), Code-Prüfer (prüfen).

### B5 – wichtig – strategy_search sucht nicht im Default-Spiel
Fundstelle: examples/strategy_search.rs:74–76 (Default `[Grunts, All]`), Doc :9.
Nach dem Merge deckt der Default von strategy_search weder den Spiel-Default ab, noch ist er im PR erwähnt. Laut Plan soll K4 über `strategy_search --rules classic` laufen (OP-027) – dann ohne Standard-Mix, falls niemand `--enemies standard` angibt.
Vorschlag: Default auf `[Standard, Grunts]` oder `[Standard, All]` ändern oder bewusst lassen und in PLAN (S-10/K4) `--enemies standard` festschreiben. Optionen an Jannus/Planer.
Wer: Jannus/Planer (Entscheidung), Executor.

### B6 – klein – Report-Beschriftung macht alte und neue Default-Läufe ununterscheidbar
Fundstelle: examples/playtest.rs:283–286.
Der Default-Mix bekommt keine Beschriftung. Vorher hieß „ohne Label“ = grunts, jetzt = standard. Alte gespeicherte Report-Texte (Playtest-Notizen, PR-Bodies #16/#19 u. a.) und neue sehen gleich aus. JSONL-Dateinamen tragen den Mix immer (playtest.rs:390–395), dort kein Problem.
Vorschlag: Mix im Report immer ausgeben (oder eine Kopfzeile „enemies: standard“), damit S-07 und folgende Reports selbsterklärend sind.
Wer: Executor.

### B7 – klein – Unbekannter ARENA_ENEMIES-Wert fällt still auf den Default
Fundstelle: src/enemies.rs:64–67.
Vorbestehend; neu ist nur, dass ein Tippfehler (z. B. `grunt-only`, `charge`) jetzt still den Standard-Mix statt des alten Spiels liefert. Wer bewusst „das alte Spiel“ vergleichen will, merkt den Fehler nicht. Leerzeichen und Groß-/Kleinschreibung werden toleriert (kinds.rs:89). Risiko gering, aber für Vergleichsläufe im Fenster (`cargo run`) real. Bots/Playtest sind nicht betroffen (dort Parse-Fehler mit Meldung, examples/playtest.rs:… über `value()?.parse()?`; session.rs setzt den Mix explizit, :85).
Vorschlag: Bei Parse-Fehler einmal `warn!` mit der Fehlermeldung ausgeben (kein Abbruch). Kein unwrap nötig.
Wer: Executor (optional, kann in Folge-Slice).

### B8 – klein – Doku-Details
- src/enemies.rs:4–5 Modul-Doc: „the grunt mix is the original game“ ist richtig, nennt aber den Default nicht. Vorschlag: „the default is the standard mix (grunt, charger, swarm); the grunt mix is the original game.“
- src/enemies/kinds.rs:230 Kommentar „A wave of 8 already holds both specials“: stimmt, aber schon die kleinste Welle (4 Slots) hält beide (GWGC). Irreführend klein; besser „even the smallest wave (4) holds both“. Der Test kinds.rs:288 prüft entsprechend nur 8.
- ARCHITECTURE.md:166 und README.md:52–55: Zeilen nicht mehr umbrochen (stilistisch). README erwähnt nicht, dass ein Swarm ein Rudel aus 6 ist – sonst wirkt „mix … swarms“ harmlos.
- Nach dem Merge veraltet: docs/PLAN.md:37 (Plan-Branch: „`Grunts` (Default)“, Zeilenangaben kinds.rs:56–65/223–231 verschoben), docs/OP.md:27 OP-018-Hinweis („kennt nur Grunts, With, All“), PLAN.md:60 (S-01: `ARENA_ENEMIES` ohne `standard`). Gehört in die STAND/OP-Pflege, nicht in diesen PR.
Wer: Executor (die ersten beiden), Leiter/Planer (Plan/OP).

### B9 – klein – Variante `Standard` passt nicht zu OP-018 (wellenabhängig)
Fundstelle: src/enemies/kinds.rs:245 (`kind_for_slot(mix, slot)` ohne Wellennummer).
S-08 braucht „ab Welle 6/10/14 kommen Arten dazu“; dafür muss die Signatur die Wellennummer bekommen oder `Standard` durch einen Wellenplan ersetzt werden. Kein Fehler jetzt, aber `Standard` als Enum-Variante wird in S-08 wahrscheinlich umgebaut. Wichtig für S-07: die Baseline misst einen Mix, den es nach S-08 so nicht mehr gibt (ist im Plan schon als bekannt notiert, PLAN.md:54).
Vorschlag: Nichts ändern; nur in S-08 einplanen.
Wer: Planer.

## Geprüft ohne Befund (Fokus 5)
- Kein `unwrap`/`expect` neu in Gameplay-Code; Tests nutzen keine.
- Kein RNG: Muster rein arithmetisch, deterministisch (Test kinds.rs:282).
- Domain-Grenzen: `EnemyMix` bleibt über `crate::enemies::EnemyMix` erreichbar, `STANDARD_MIX`/`Slot` privat, kein `pub mod`.
- PLACEHOLDER: `STANDARD_MIX` markiert und im PR-Body gelistet. `SWARM_SIZE` (bestehend) bestimmt den Effekt mit, ist nicht neu.
- Alle `match` auf `EnemyMix` in examples/playtest.rs sind um `Standard` ergänzt (:349–353, :390–395); Display/FromStr/Fehlertext konsistent.
- `SessionConfig` setzt den Mix nach den Plugins explizit (session.rs:85), Bots hängen also nicht von `ARENA_ENEMIES` ab.
- tests/playtest_tiers.rs bricht nicht (pinnt Grunts/All).

## Offene Fragen an Jannus
1. Ist ~70 % Swarm-Körper / ~6 % Charger der gewollte Standard (B1)?
2. Soll strategy_search per Default den Standard-Mix suchen (B5)?
3. Soll S-07 den Grunt-Bezugslauf mit 3 Seeds mitmessen (B2)?

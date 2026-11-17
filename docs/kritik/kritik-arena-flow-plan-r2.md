# Kritik docs/PLAN.md (Commit aae29de, r3), Runde 2

Rolle: Kritiker, nur gelesen (Worktree /home/user/Arena-flow-wt-plan). Nichts geändert, nichts entschieden.
Geprüft: Diff abbdd5c..aae29de, Tabelle "Kritik R1 -> wo umgesetzt", OP.md auf origin/claude/quirky-johnson-84qh32 (02dc985), Rest-Wahrheiten, Pfad:Zeile.

Urteil: **ok mit Befunden**. 0 blockierend, 6 wichtig, 6 klein.

## 1. R1-Befunde: umgesetzt oder plausibel abgelehnt?
Alle 19 Befunde sind im Plan angefasst. Ehrlich umgesetzt: B2 (Suche läuft, s. 4), B3, B5, B8, B10, B11, B12 (H-D, D-D drin), B13, B14, B15, B16, B17. B19: Ablehnung (Aufwand in Minuten statt Tool-Calls, S-07/S-08 nicht teilen) ist plausibel (OP.md-Kopfzeile: Laufzeit in Minuten), die Parallel-Lücken sind nur teilweise zu (B7 unten).
Nur teilweise umgesetzt (Rest steht unten als neue Befunde): R1-B1 (Erfolgsmaß S-11 ohne Baseline -> B1), R1-B4 (K4 Text, K5 -> B2, B5), R1-B7 (Maß S-07 vs. Steuerung S-11 -> B3), R1-B9 (Definition "gesehen" -> B4), R1-B18 (S-11 konditional -> B6).

## 2. Entscheidungen Jannus
OP-009, OP-018, OP-021, OP-025, OP-026, OP-027, OP-028 stimmen mit OP.md überein, keine Verfälschung. OP-025: der Satz zu S-11 (weniger Tode/overwhelmed direkt nach "anheben", K1 unverändert) ist wörtlich übernommen. OP-029–031 stehen als Annahme Planer, OP-032/033 als später, OP-007/008/019 offen: korrekt. Zusätze des Planers sind meist als Vorschlag markiert; Ausnahmen in B5 und B9.

## 3. Befunde

**B1 | wichtig | PLAN.md:375 (S-11 Punkt 5), :371-379, S-07 Punkt 2/6 (:~330-335); OP-025**
Das von Jannus beschlossene Erfolgsmaß für S-11 ("Anteil Tode/overwhelmed in der Welle direkt nach 'anheben' sinkt gegenüber S-07") hat keine Baseline und keinen Code: S-07 misst Deckel, Boden, bored+knapp, in flow, furthest_wave, aber nicht das Maß nach "anheben". `Summary` kennt es nicht, kein Schritt in S-07 oder S-11 baut es (S-11-Pfade nennen summary.rs nur für `in_band`, :105). Der AK "Erfolgsmaß erreicht und im PR belegt" ist so nicht erfüllbar. Zusätzlich offen: Mindestzahl "anheben"-Ereignisse (mit D-C wird weniger angehoben, die Stichprobe schrumpft).
Vorschlag: Maß in S-07 Punkt 2 aufnehmen (aus Entscheidungsfolge je Session, Nenner = Anzahl "anheben"), Mindeststichprobe nennen; S-11 vergleicht dagegen. Das Maß selbst bleibt Jannus' Wortlaut.
Wer: Planer (Text), Jannus nur falls Maß präzisiert werden soll.

**B2 | wichtig | PLAN.md:66 (K5), S-11 Punkt 6 (:376), :208 (T-A0)**
Der K5-Test ist nicht trennscharf. Bot-Paar "gleiche Hände, Tell-Lesen an/aus" überlebt verschieden, `wave_risk` und damit die Schwierigkeit laufen auseinander, also unterscheiden sich "in mindestens einem Hebel" die Werte ohnehin (`levers_for` hängt nur an der Schwierigkeit, decide.rs:140-156), auch ohne jede meso-Steuerung. Der Test wäre schon ohne T-A0 grün.
Vorschlag: Hebel-Verhältnis bei gleicher Schwierigkeit vergleichen (Anzahl vs. Tempo/Schaden), oder Schwierigkeit pinnen. Messdefinition für K5 festlegen lassen.
Wer: Planer, Jannus (K5-Maß).

**B3 | wichtig | PLAN.md:~330 (S-07 Punkt 2), :174-180 (§4.2), S-11 Punkt 1 (:~371); OP-029**
S-07 misst "bored, aber knapp" relativ (Bedrohungszeit-Anteil >= Median der in-flow-Wellen derselben Stufe). S-11 steuert aber mit einer absoluten PLACEHOLDER-Schwelle in `DirectorConfig`; der Director kennt weder Stufe noch Median. Gemessen wird weiter nicht, worauf entschieden wird (R1-B7 nur verschoben). Außerdem: `Summary::of` arbeitet je Session; "Median derselben Stufe" über Seeds ist dort nicht berechenbar, und für Stufen ohne in-flow-Wellen (Novice am Boden, Skilled am Deckel) ist der Median undefiniert.
Vorschlag: S-07 weist zusätzlich den Anteil bored-Wellen oberhalb einer festen Schwelle aus (dieselbe Größe wie S-11) und legt den Median-Nenner (je Session oder gepoolt, leere Menge) fest.
Wer: Planer, Jannus (OP-029).

**B4 | wichtig | PLAN.md:~345 (S-09a Punkt 2), §4.1 "Grenze", S-09b Punkt 3a; OP-030; src/enemies/movement.rs:87-92, src/playtest/steering.rs:33-47, src/playtest/bot.rs:41, :228**
OP-030 (Annahme Planer): Charger-Tell zählt als "gesehen", wenn der Spieler **am Ende des Wind-ups** höchstens X von der Dash-Linie entfernt ist. Der Bot mit `reads_tells` weicht aber schon während des Wind-ups aus der Spur (sidestep, TELL_LANE = 50, steering.rs:43). Am Ende des Wind-ups steht er außerhalb und fällt aus "gesehen" heraus, solange X nicht deutlich über 50 liegt (X ist offen). Dann zählen bei Tell-Lesern vor allem die Fälle, in denen das Ausweichen misslang. Der S-09b-Test 3a "meso-Quote mit Tells höher" kann so invertiert ausfallen. Kein Beleg, dass er so ausfällt, aber die Definition misst den Endzustand, nicht die Bedrohung beim Tell-Start.
Vorschlag: Position beim Tell-Start (erste Tick mit `ChargeTell.aim` = Some) für "gesehen" nehmen; "ausgewichen" = in der Spur beim Start und kein Treffer.
Wer: Planer, Jannus (OP-030).

**B5 | wichtig | PLAN.md:63 (K4 Text und Prüfung), S-09b Punkt 3b; OP-027, OP-009**
(a) Der K4-Kriterientext sagt weiter "der Report zeigt je Session mikro, meso und makro als eigene Zahlen ... je Bereich ein Bot-Paar". Nach OP-027 ist makro offline (strategy_search) und A-A nur informativ, also zeigt der Report makro nicht als K4-Zahl. Text und Entscheidung widersprechen sich im selben Feld. (b) Das mikro-Bot-Paar "nur Hände" (S-09b 3b) ist eine Planer-Erweiterung gegenüber der Jannus-Entscheidung OP-009 (M-C = Tells an/aus), in K4 nicht als Vorschlag markiert. (c) Fällt der strategy_search-Lauf mit `classic` negativ aus (R1 wies darauf hin, dass der Unterschied mit classic nicht gemessen ist), meldet S-10 nur "Vorbefund", S-13 hängt aber an K4; ein Weg danach fehlt (Frage an Jannus).
Vorschlag: K4-Satz an OP-027 angleichen, 3b als Vorschlag Planer kennzeichnen, Folge bei negativem makro-Befund als offene Frage listen.
Wer: Planer, Jannus (c).

**B6 | wichtig | PLAN.md:457 (Gate "Keine bedingten AKs"), S-13 "Fertig wenn" (:~396), S-11 Kopf (:365), OP-032, §4.3 H-C**
Die Gate-Zeile behauptet, S-13 sei frei von bedingten AKs. S-13 "jedes Kriterium K1-K9 hat einen Test" gilt nicht, wenn K3 bei H-C wegfällt (§4.3), K5 bei T-C entfällt (K5-Zeile), K2 am Rand offen ist (OP-032; ausdrücklich als Blocker genannt). S-11 ist für eine nicht entschiedene Empfehlung (D-C + T-A0) geschnitten, trägt K5-Test und T-A0-Punkt 2 als feste AKs. Das ist ein bedingter AK, nur im Kopf offengelegt.
Vorschlag: Gate-Zeile korrigieren (S-13 und S-11 bedingt auf OP-007/OP-008/OP-019/OP-032) oder S-11 erst nach den Entscheidungen schneiden.
Wer: Planer.

**B7 | klein | PLAN.md:~300 (Nicht-parallel-Liste), :~232 (S-01-Empfehlung)**
Lücken: S-09a ∥ S-10 (beide summary.rs, S-09a :154 Test-Literal), S-10 ∥ S-11 (summary.rs), README.md wird von S-01, S-08, S-10 geändert (STAND.md:16 nennt die README-Regel für S-01/S-04), nicht in der Liste.
Wer: Planer.

**B8 | klein | PLAN.md:§8 Zeile "three v1 levers", S-13 Pfade/OPs; OP-023**
§8 sagt "spätestens S-12, sonst S-13", OP-023 nur "spätestens S-13". S-13 listet OP-023 unter OPs, hat aber keinen Schritt und keine Pfade für src (decide.rs:140, difficulty.rs:107, weapons.rs:121); K9 deckt nur README/ARCHITECTURE. Ohne Schritt bleibt der Code-Begriff liegen.
Wer: Planer.

**B9 | klein | PLAN.md:3 (Kopf), S-13 Kopf, OP.md OP-014**
Kopfzeile nennt als eingearbeitet nur OP-009/018/021, obwohl OP-025-028 ebenfalls entschieden eingearbeitet sind (steht nur im Titel). "Gegenprüfung nach S-08" in S-13-Abhängigkeiten steht dort ohne Vorschlag-Label (in §2 ist sie als Vorschlag Planer gekennzeichnet; OP-021 sagt nur "Schwellen nach S-07"). OP.md OP-014 sagt CI "ca. 15 min", Plan/STAND.md:3 "ca. 16 min": eine Zahl, Sync Leiter.
Wer: Planer/Leiter.

**B10 | klein | PLAN.md S-09b "Fertig wenn", S-07 Punkt 5**
S-09b vergleicht mit "S-07-Lauf mit explizit gesetztem --enemies <S-04-Mix>". S-07 Punkt 5 verlangt keinen Lauf mit explizit benanntem Mix und kein Festhalten seines Namens; der Name ist bis zum S-04-PR unbekannt.
Vorschlag: In S-07 Punkt 5/6 Mixname in die Baseline-Datei schreiben.
Wer: Planer.

**B11 | klein | PLAN.md:62 (K3), S-13 Punkt 1**
K3-Schwelle gilt "gepoolt über Seeds" (10 min, 3 Seeds), der S-13-Test sagt weder Seeds noch Minuten. Ein 4-Minuten-Lauf trifft den Deckel-Anteil anders als 10 Minuten.
Wer: Planer.

**B12 | klein | PLAN.md (Pfad:Zeile)**
Stichproben unten alle stimmig bis auf `src/enemies/spawning.rs:13`: dort steht die Doc-Zeile, die Konstante liegt auf :14 (so auch in §4.3 und §5, "Grobschätzung"). Ebenso in R1 schon so genannt; unschädlich.
Wer: Planer.

## 4. Rest-Wahrheiten (§8, ausgeführt im Worktree)
- `rg -n 'Enemies have 3 hp|grunts only|\bv1\b|sound, fonts' README.md ARCHITECTURE.md`: 4 Treffer (README.md:52, README.md:175, ARCHITECTURE.md:166, ARCHITECTURE.md:209). Entspricht der Erwartung "4".
- `rg -n 'risk-flow-curve\.png' README.md ARCHITECTURE.md src`: 3 Treffer (README.md:88, ARCHITECTURE.md:214, src/flow_director/curve.rs:1). Entspricht.
- `rg -n '\bv1\b' src`: 3 Treffer (difficulty.rs:107, decide.rs:140, weapons.rs:121). Entspricht.
- Kein weiterer Treffer in README/ARCHITECTURE bei `-i \bv1\b|3 hp|grunts? only`.
- Begriffe ohne rg (Spielertyp, Shards entschieden usw.): im Plan nur als Verworfenes zitiert; "Spielertyp-Reaktion" in K5 ist Jannus' Begriff, kein Verstoß (OP-019 offen, K5 bedingt).

## 5. Pfad:Zeile-Stichproben (neu in r2/r3), alle gelesen
examples/playtest.rs:154 (pins 1..=10) ok; src/enemies/kinds.rs:223-231 ok; src/enemies/spawning.rs:128 ok; src/playtest/strategy.rs:16 (TIE_STANDARD_ERRORS 2.0) ok; src/playtest/session.rs:39-42 ok; src/playtest/tier.rs:167/:169 ok, :258 (HandSkill) ok; src/playtest/bot.rs:228 ok; src/enemies.rs:26-32 ok; src/enemies/movement.rs:16/:99 ok; src/playtest/summary.rs:22/:26/:105 ok; src/playtest/report.rs:30-31 ok; src/telemetry/session_line.rs:228-248/:259 ok; src/waves/transitions.rs:8/:70/:101-103 ok; examples/strategy_search.rs:48-49/:72 ok; playtest-notes/VORLAGE.md:27 (Flow Start/Ende) ok; src/playtest/agent.rs:401 ok; docs/STAND.md:3 (16 min) ok. Abweichung nur B12.

## 6. Abhängigkeiten
Graph und Text der Slices: kein Zyklus. S-07 -> S-08/S-09a/S-10/S-11/S-12; S-09a -> S-09b -> S-11; S-12 -.-> S-11 (nur T-A mit H-B); H-1 nach S-06+S-08, H-2 nach S-11; S-13 hängt an allen wie im Text. Keine neuen Widersprüche zwischen Graph und Slice-Kopf gefunden außer den Parallel-Lücken (B7).

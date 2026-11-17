# Kritik docs/PLAN.md (Commit f7f732c, r4), Runde 3 = Ready-Gate

Rolle: Kritiker (Opus), nur gelesen. Nichts geändert, nichts entschieden.
Geprüft: `git diff aae29de f7f732c -- docs/PLAN.md`, §9 Ready-Gate-Checkliste, Tabelle „Schneidbarkeit“, Tabelle „Kritik R2 → wo umgesetzt“, OP.md auf origin/claude/quirky-johnson-84qh32 (d999537), STAND.md ebenda, origin/main (jetzt beccedc = #30, S-04 gemergt), Code-Stichproben im Worktree (Code-Stand #27) und auf origin/main.

**Urteil: Teilstart ready für S-01, S-02 Rest (nach OK Jannus), S-07, S-08, S-10. Nicht ready: S-09a und damit S-09b (B1).** 1 blockierend, 5 wichtig, 6 klein.

## 1. R2-Befunde umgesetzt?
- B1 (Erfolgsmaß „nach anheben“): umgesetzt in S-07 Punkt 2/6, Nenner und Anzahl, S-11 vergleicht dagegen. Geprüft: `WaveRecord.decision` ist `Option<DecisionReason>` (src/telemetry/session_record.rs:112–113), `Raised` existiert (difficulty.rs:80); Tod zählt über `wave_risk` = 1,0 (decide.rs:77–78) als overwhelmed. Stimmt (Vergleich muss `Some(Raised)` lauten, unschädlich).
- B2 (K5): ehrlich als offen markiert, R2-2 = OP-036, S-11 nicht ready. Ok.
- B3 (relativ vs. absolut): in S-07, §4.2, S-11, §4.7 umgesetzt, **aber §4.6 nicht nachgezogen** (B3 unten).
- B4 („gesehen“): umgesetzt, aber die neue Fassung ist am Code gemessen leer bzw. nicht berechenbar (B1 unten).
- B5, B6, B8, B10, B11, B12: umgesetzt, nachgeprüft. B12: keine `spawning.rs:13` mehr außer in der R2-Tabelle.
- B7: umgesetzt; Rest siehe B10 unten (klein).
- B9: Kopf und S-13 ok; OP.md OP-014 sagt jetzt 16 min. Ok.

## 2. Entscheidungen Jannus (OP-002, 009, 018, 021, 025–028)
Wortlaut in §0, §2, §4.1, §4.6, §5, §7 deckt sich mit OP.md; keine Verfälschung gefunden. OP-025-Satz zu S-11 wörtlich. Widersprüche nur bei Annahmen und Verwaltung (B3, B6, B9).

## 3. Befunde

**B1 | blockierend (für S-09a, damit S-09b) | PLAN.md S-09a Punkt 2 und „Fertig wenn“, §4.7 R2-4 a, OP.md OP-030; src/enemies/movement.rs:99–104, src/enemies.rs:98–110, :139–162**
Die Definition R2-4 a ist am Code nicht umsetzbar wie geschrieben:
(1) Der Teil „beim ersten Tick mit `ChargeTell.aim = Some` höchstens PLACEHOLDER Einheiten von der Dash-Linie“ ist leer: beim Übergang Approach → WindUp wird `direction = to_player.normalize_or(..)` gesetzt (movement.rs:99–104) und im selben Tick als `ChargeTell` geschrieben (enemies.rs:157–160). Der Spieler steht im ersten Tick also immer auf der Linie (Abstand ≈ 0, höchstens eine Tick-Bewegung). Der PLACEHOLDER-Abstand filtert nichts.
(2) Damit hängt „gefährdend / nicht gesehen“ allein an „in Dash-Reichweite“. Die Reichweite ist im Plan nicht definiert (kein Default) und aus `waves` nicht berechenbar: sie hängt an Gegner-Tempo × `CHARGE_DASH_SCALE` × `CHARGE_DASH_SECS`, begrenzt durch `dash_cap(level)` (movement.rs:20–29, :50–51), und `Chaser`/`Gait` sind private Typen in `enemies` (enemies.rs:98, :105). S-09a Punkt 1 verlangt für genau diesen Fall „stoppen und als Blocker melden“. Der Executor läuft also absehbar in den Stopp oder erfindet eine Definition; der AK „nicht gefährdender Charger-Tell zählt nicht als gesehen“ ist ohne neue Oberfläche aus `enemies` nicht prüfbar.
(3) Nebenbei: „in diesem Anlauf“ ist für den Swarm-Dive nicht abgegrenzt (`DiveTell` kennt nur `winding_up`, src/enemies.rs:37–39; ob je Schwarm-Mitglied oder je Schwarm gezählt wird, steht nicht da).
Vorschlag (Optionen für Jannus, OP-030): **a** „gesehen“ = jeder Charger-Tell, dessen Charger beim ersten Tick höchstens PLACEHOLDER Einheiten entfernt ist (fester Abstand statt Dash-Reichweite, aus `Transform` lesbar; Default z. B. = Auslöseabstand, dann zählt jeder Tell); **b** OP-030 Option c (jeder Tell zählt), „ausgewichen“ = kein `HitSource::Contact(charger)` bis Ende des Dashs; **c** `enemies` veröffentlicht die Dash-Reichweite (neue Oberfläche, Architekturfrage). Empfehlung Kritiker: b, weil der Bot im ersten Tick ohnehin auf der Linie steht und sich das Paar Tells an/aus dann nur im Ausweichen unterscheidet (genau M-C). Swarm: je Mitglied, Anlauf = vom Wind-up bis zum nächsten Wind-up oder Wellenende.
Wer: Planer (Text), Jannus (OP-030 ist Annahme; bei c Kritiker wegen Architektur).

**B2 | wichtig | PLAN.md Kopf („main = #27“), §6 „Laufend“ (S-04 läuft), S-07 Punkt 1/5, S-08 Pfade, G6, §8; origin/main beccedc (#30)**
S-04 ist als #30 gemergt (STAND.md:3), der Plan steht noch auf #27. Folgen: (a) Mix-Name ist bekannt (`standard`, `EnemyMix::Standard`), alle `<S-04-Mix>` lassen sich auflösen. (b) S-07 Punkt 1 ist erledigt (examples/playtest.rs:185 nutzt `EnemyMix::default()`). (c) **`tests/playtest_tiers.rs` hat S-04 nicht umgestellt** (`settle` nutzt weiter `EnemyMix::Grunts`, :26; :32 ebenso). Der Plan sagt in §6 „Falls nicht, übernimmt S-07 Punkt 1“, S-07 Punkt 1 nennt aber nur examples/playtest.rs. Damit misst der K1-Test Grunts, S-07 Punkt 5 („Bedingungen des K1-Tests“) aber `standard`. (d) Zeilen verschoben: src/enemies/kinds.rs (`EnemyMix` jetzt ab :58, Fehlermeldung :105, `ALL_MIX` :201, `kind_for_slot` :245, Tests ab :256; neu `STANDARD_MIX` :229–241, auf dem S-08 aufbauen sollte, nicht nur auf `ALL_MIX`), examples/playtest.rs :349–352, :390–394. S-07-Pfade (summary.rs, report.rs, playtest.rs:184–186, tests) stimmen weiter.
Vorschlag: Plan auf #30 heben: S-07 Punkt 1 ersetzen durch „Default in tests/playtest_tiers.rs (`settle`, :26) auf `EnemyMix::Standard` bzw. `default()`; K1-Test mit `standard`“, `<S-04-Mix>` → `standard`, S-08-Pfade und G6 neu, S-08 Punkt 2 auf `STANDARD_MIX` beziehen.
Wer: Planer.

**B3 | wichtig | PLAN.md §4.6 Zeilen OP-029 und OP-030 (Zeilen 222–223) vs. S-07 Punkt 2, S-09a Punkt 2, §4.7 R2-3/R2-4, OP.md OP-029/OP-030**
§4.6 nennt weiter die alten Defaults („≥ Median der in-flow-Wellen derselben Stufe“, „beim Ende des Wind-ups“). OP.md und die Slices nutzen die neuen (R2-3 b, R2-4 a). Ein Executor, der über die OP-ID nach §4.6 springt, liest den falschen Default. Der R2-Tabellenvermerk „umgesetzt“ für B3/B4 stimmt damit nur für die Slices.
Vorschlag: §4.6-Zeilen auf den OP.md-Wortlaut bringen oder auf §4.7 verweisen.
Wer: Planer.

**B4 | wichtig | PLAN.md §8 Zeilen „Enemies have 3 hp“ / „grunts only“ / Sound, Anhang A S-01 „Fertig wenn“; README.md auf origin/main**
Nach #30 liefert die K9-Suche `rg -n 'Enemies have 3 hp|grunts only|\bv1\b|sound, fonts' README.md ARCHITECTURE.md` (ausgeführt auf origin/main): README.md:52, **README.md:53** („`ARENA_ENEMIES=grunt` for grunts only (the old game)“, inhaltlich richtig, also Falschtreffer), README.md:178 (statt :175), ARCHITECTURE.md:209. ARCHITECTURE.md:166 ist durch S-04 erledigt. S-01 „Fertig wenn“ nennt README.md:52, :175; die Suche hätte nach S-01 weiter den Falschtreffer :53, K9 („0 Treffer“) wäre so nicht erreichbar. S-01 Punkt 2 (ARENA_ENEMIES dokumentieren) ist durch #30 teilweise erledigt.
Vorschlag: Suchmuster für „grunts only“ schärfen (z. B. `default mix is grunts only`) oder :53 als ausgeschlossenen Falschtreffer führen; S-01 „Fertig wenn“ auf die Suche statt auf Zeilennummern beziehen; S-01 Punkt 2 auf `ARENA_PICKUPS` kürzen.
Wer: Planer.

**B5 | wichtig | PLAN.md §9 Checkliste und „Schneidbarkeit“**
Die Checkliste ist nicht falsch abgehakt (nur drei [x], alle belegt), aber für einen Teilstart unvollständig: (a) kein Punkt für die R2-Fragen (R2-1…R2-7 / OP-034–OP-038, ob als Annahme mit Default oder später eingeordnet); (b) „Keine bedingten AKs … S-07, S-08, S-09a, S-09b, S-10 frei davon“ nennt S-01/S-02 nicht, die in „Schneidbarkeit“ als ready stehen; (c) die Zeile „Jede Slice hat prüfbare Fertig wenn, exakte Pfade …“ bleibt offen, während die Tabelle S-08 als ready führt, obwohl seine Pfade seit #30 verschoben sind (B2) und S-09a einen nicht prüfbaren AK hat (B1). Die Tabelle „Schneidbarkeit“ müsste diese Zeile je ready-Slice abdecken.
Vorschlag: Gate-Zeile „Teilstart“ ergänzen: je ready-Slice AK prüfbar / Pfade auf aktuellem main / gezielter Test / Annahmen mit Default, abgehakt vom Leiter; R2-Punkt ergänzen.
Wer: Planer, Leiter (abhaken).

**B6 | wichtig | PLAN.md überall „R2-1 … R2-7“ vs. OP.md OP-034–OP-038**
Der Leiter hat IDs vergeben (R2-1 → OP-034, R2-2 → OP-036, R2-5 → OP-037, R2-6 → OP-038, R2-7 → OP-035; R2-3/R2-4 in OP-029/OP-030 aufgegangen). Der Plan verweist in K4, K5, S-07, S-09a, S-09b, S-10, S-11, S-13, „Schneidbarkeit“ nur auf R2-x. OP.md verweist umgekehrt auf „PLAN.md §4.7 R2-x“. Das hält, solange beide Dateien leben, ist aber genau die doppelte Buchführung, die bei R1 (F1–F9 → OP-025–033) in §4.6 aufgelöst wurde.
Vorschlag: in §4.7 eine Spalte „OP“ ergänzen und in den Slices OP-IDs nennen (wie §4.6).
Wer: Planer.

**B7 | klein | OP.md Kopfzeile („OP-034–OP-039“) vs. Tabelle (endet bei OP-038)**
Wer: Leiter.

**B8 | klein | OP.md OP-018 Spalte „blockiert Slice“: „S-04 (nur der wellenabhängige Teil)“; PLAN §5/S-08**
Umsetzung ist S-08, S-04 ist gemergt. Ebenso OP-014 Typ „später“, obwohl S-02 Rest daran hängt (in „Schneidbarkeit“ korrekt als „nach OK Jannus“).
Wer: Leiter.

**B9 | klein | PLAN.md Anhang A S-01 Kopf „Abhängigkeiten: keine (Empfehlung: nach S-04)“ vs. §6 Parallel-Zeile („S-01 erst nach S-04“) und „Schneidbarkeit“ („Merge S-04“)**
Seit #30 gegenstandslos, aber der Kopf widerspricht der Tabelle. Kopf angleichen.
Wer: Planer.

**B10 | klein | PLAN.md §6 Parallel-Zeile**
(a) „README.md: S-01, S-08, S-10 und S-13 nacheinander“ ist mehrdeutig: Reihenfolge erzwungen (S-10 erst nach S-08, obwohl S-10 nur S-07 braucht) oder nur „nicht parallel“? (b) S-09a ∥ S-12 fehlt (S-09a ändert Test-Literal decide.rs:173–192, S-12 decide.rs:140–156); erst relevant, wenn S-12 geschnitten ist. (c) Sonst geprüft und vollständig für die ready-Slices: summary.rs (S-07/S-09a/S-09b/S-10), report.rs, examples/playtest.rs (S-07/S-08/S-09b), tests/playtest_tiers.rs (S-07/S-08/S-09b), README.md, session_line.rs, ARCHITECTURE.md (nur S-08 unter den ready).
Wer: Planer.

**B11 | klein | PLAN.md S-10 Punkt 4, S-07 Punkt 5**
Laufzeit des `strategy_search`-Laufs (4 Stufen × 4 Pins × 3 Seeds × alle Policies, 4 min) ist weder geschätzt noch im Aufwand „ca. 20–30 min“ erkennbar eingerechnet; ebenso die Report-Läufe in S-07 (dort als Annahme benannt). Vorschlag: wie S-07 „Wanduhr-Zeit festhalten“ und Aufwand als Annahme markieren.
Wer: Planer.

**B12 | klein | PLAN.md S-02 Rest (Anhang A)**
„Fertig wenn“ prüft nur einen absichtlichen clippy-Fehler, nicht fmt. Ob `cargo clippy --all-targets -- -D warnings` auf main heute grün ist, ist nicht belegt (von mir nicht ausgeführt). Ist es rot, ist S-02 Rest größer als 5–10 min. Vorschlag: vor Start einmal lokal laufen lassen; fmt-Fehler ebenfalls als Gegenprobe.
Wer: Planer/Leiter.

## 4. Ready-Prüfung je „sofort schneidbarer“ Slice
| Slice | AK prüfbar, ohne Bedingung | Pfade exakt | gezielter Test | offene blockierende OPs | Annahmen mit Default | Urteil |
|---|---|---|---|---|---|---|
| S-01 | ja (Zeilen veraltet, B4) | ja | `cargo test` (Doku-Slice, ok) | keine | – | ready nach B4-Korrektur der Suche; startbar |
| S-02 Rest | ja (B12) | ja | CI-Lauf | OK Jannus OP-014 fehlt | Annahme Leiter | ready, startbar nach OK |
| S-07 | ja | ja (Stand #27 gilt hier noch) | ja | keine | OP-029 (R2-3), OP-031, OP-034 | ready; Punkt 1 nach B2 auf tests/playtest_tiers.rs umstellen |
| S-08 | ja | **verschoben seit #30** (B2) | ja | keine | Startwellen PLACEHOLDER | ready nach Pfad-Update; nicht vor S-07 |
| S-09a | **nein** (B1) | ja | ja | keine | OP-030: Default nicht umsetzbar | **nicht ready** |
| S-09b | ja | ja | ja | keine | OP-035 | hängt an S-09a/B1 |
| S-10 | ja | Datei-Ebene, ok | ja | keine | OP-026/027 entschieden; Erholungs-Detail als Vorschlag | ready |
S-11, S-12, S-13: korrekt als Entwurf / nicht geschnitten markiert; kein Einwand.

## 5. Nachgeprüft ohne Befund
src/playtest/summary.rs:15–20 (`Summary` hat `PartialEq`, K7-Test möglich), :57–80; WaveReport-Literale vollständig (summary.rs:154, session_line.rs:179/:282, decide.rs:174, transitions.rs:72; decide.rs:214/:221 mit `..report()`); `HitSource::Contact(Entity)` und `Hit` aus `crate::combat` lesbar (danger.rs:6 tut es schon); examples/strategy_search.rs `--rules`/`--enemies`/`--pin`, Default-Mixe :74–75; session.rs:36–56; tier.rs:169; telemetry/session_line.rs:35–37.

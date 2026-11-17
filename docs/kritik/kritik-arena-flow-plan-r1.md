# Kritik docs/PLAN.md (Commit abbdd5c, PR #29), Runde 1

Rolle: Kritiker, nur gelesen (Worktree /home/user/Arena-flow-wt-plan). Nichts geändert, nichts entschieden.
Hinweis: Auf dem Remote liegt ein neuerer Commit f919657 (OP.md: OP-009, OP-018, OP-021 "entschieden", OP-022..024). Geprüft wurde PLAN.md wie in abbdd5c; Befunde, die K2 betreffen, gelten auch für das von Jannus laut OP-021 bereits angenommene K1-K9.

Urteil: **nicht ready**. 2 blockierend, 10 wichtig, 8 klein.

Schreibweise: B-Nr. | Schwere | Fundstelle | Befund | Vorschlag | Wer

## Blockierend

**B1 | blockierend | PLAN.md:55, :60 (K2), :143 (G7); src/playtest/summary.rs:62, :68-82; src/flow_director/curve.rs:5-27; src/flow_director/decide.rs:20-33**
Der Plan sagt, K2 sei nicht zirkulär, weil es "nicht mit risk in band" misst, sondern auf der Flow-Kurve. Das stimmt nur für die Band-Konstanten, nicht für das Signal. `in flow`, `bored` und `overwhelmed` kommen aus `engagement(wave_risk)` (summary.rs:68-82). Das ist dieselbe Größe `wave_risk`, die der Director glättet und in sein Band steuert (decide.rs:97-118).
Rechnung: in_flow = engagement >= 0,6. Steigend: r/0,5 >= 0,6 gibt r >= 0,30. Fallend: exp(-6(r-0,5)) >= 0,6 gibt r <= 0,5 + ln(1/0,6)/6 = 0,585. Das Fenster ist also r in [0,30; 0,585]. Das Director-Band liegt auf geglättetem Risiko bei [0,35; 0,65]. Beide Fenster überlappen fast vollständig. Ein funktionierender Regler erfüllt K2 damit nahezu per Konstruktion; K2 misst den Regelfehler des Directors, nicht Flow.
Dazu: Das Band wurde laut Kommentar "by a 96-variant bot search for the most waves near the peak" gewählt (decide.rs:26-28). Es ist also auf genau diese Zahl optimiert. Unabhängig ist nur der Anker FLOW_PEAK = 0,5 (Agenten-Bewertung laut curve.rs:10-11) und K6 (ein Mensch).
Gegenläufig zu S-11: Die Danger-Bremse (D-C) hält die Schwierigkeit in "knapp, aber ohne Schaden"-Wellen. Das erzeugt nach dieser Metrik mehr Wellen mit Risiko < 0,15 ("bored", engagement < 0,3; LOST_BELOW-Kommentar summary.rs:11-12 stimmt: r < 0,15 bzw. > ca. 0,70). S-11 würde K2 also absichtlich verschlechtern, und curve.rs:5-8 sagt selbst, meso/Entscheidungen seien in dieser Kurve unsichtbar. Das meso-Signal, für das S-11 gebaut wird, kann K2 daher nicht belegen.
OP-022 (Annahme Planer) löst nur das Band-Problem, nicht dieses.
Vorschlag: Im Plan die Zirkularität ehrlich benennen (K2 = Regelgüte, nicht Flow-Nachweis). Ein von `wave_risk` unabhängiges Maß ergänzen oder die Aussagekraft von K2 begrenzen (z. B. K6 mit mehr als einer Person, Agent-Playtests mit `Feel` -2..+2, agent_view.rs:251-258, oder Danger-basiert). In S-11 vorab festlegen, an welchem Maß die Bremse gemessen wird. Das ist eine Inhaltsentscheidung.
Wer: Jannus (Maß/Kriterium), Planer (Text, S-11-AK).

**B2 | blockierend | PLAN.md:67 (K9), :398 (rg), :359 (S-13 AK), :388-390, :431; OP-012 bewusst offen**
K9 ("keine Treffer") ist so nicht erfüllbar und nicht aussagekräftig:
(a) `design/risk-flow-curve.png` bleibt in README.md:88, ARCHITECTURE.md:214 und curve.rs:1, solange S-03 geparkt ist (OP-012 bewusst offen). Die Suche trifft also dauerhaft. S-13 "K9-Suche ohne Treffer" ist damit unerreichbar, ohne dass OP-012 entschieden wird.
(b) Die rg-Suche aus §8 (Zeile 398) deckt nur 5 von 11 verworfenen Begriffen ab (nicht: "Sound unter not built", "Spielertyp", "Test-CI" usw.). Ich habe sie ausgeführt: Sie findet ARCHITECTURE.md:209 ("three v1" / Zeilenumbruch / "levers") nicht, obwohl §8 (Zeile 388) genau diese Stelle nennt. Die README.md:175-Zeile (sound) findet sie auch nicht.
(c) Falschtreffer: src/pickups/shards.rs:44 ("a grunt has 3 hp", sachlich richtig, Grunt max_hp 3, kinds.rs:117-118) und src/weapons.rs:121 ("One shooter in v1", anderer v1-Sinn). Beide stehen nicht in §8; S-13 Punkt 4 bereinigt nur README/ARCHITECTURE, nicht src.
Vorschlag: K9 auf README/ARCHITECTURE einschränken und die Suche zeilenübergreifend bzw. ohne die Falschtreffer formulieren (z. B. `rg -U`). Den OP-012-Rest als bekannte Ausnahme ausweisen oder K9 an S-03 koppeln. Alle verworfenen Begriffe abdecken oder bewusst nur die genannten.
Wer: Planer (Suche, Text), Jannus (Ausnahme OP-012).

## Wichtig

**B3 | wichtig | PLAN.md:60-61 (K2/K3), :187 (H-C), :189; Commit 6ce6a53 (PR #12)**
K2 (Skilled/Expert in flow >= 50 %) und K3 (<= 10 % am Deckel) hängen beide an OP-007. PR #12 nennt für Skilled/Expert flow score 0,12 bzw. 0,10 (Mittel von engagement; Grunt-Mix, älterer Stand). Ein Mittel von 0,10 mit >= 50 % der Wellen bei engagement >= 0,6 ist rechnerisch unvereinbar. Der Abstand ist groß (Stand veraltet, daher nur Hinweis). Wählt Jannus H-C ("Deckel akzeptieren"), entfällt K3, K2 für Skilled/Expert bleibt aber bestehen und wäre dann vermutlich nicht erreichbar. Der Plan nennt nur "K3 fällt weg".
Vorschlag: K2-Schwelle je Stufe erst nach S-07 festlegen (steht bereits da), zusätzlich klar sagen, was bei H-C mit K2 für Skilled/Expert passiert. S-07 sollte flow score und in-flow je Stufe als Zahlen für diese Entscheidung ausweisen (tut die Flow-Tabelle bereits).
Wer: Planer (Text), Jannus.

**B4 | wichtig | PLAN.md:59-67, :351-363 (S-13), :63 (K5), :62 (K4), :367 (H)**
Prüfbarkeit je Kriterium unvollständig:
- K5 nennt "Test (S-11/S-13)". Weder S-11 noch S-13 enthält einen Schritt, der das Bot-Paar baut (S-11 nur decide-Unit-Tests, S-13 nur K1/K3/K7).
- K4 makro: "je Bereich ein Bot-Paar, das sich nur dort unterscheidet" ist mit der empfohlenen Option A-C + A-A nicht erfüllbar. Bots lernen nicht (PLAN.md:167), und A-A (Erholung, Trend) ist aus `wave_risk` abgeleitet, also nicht getrennt von mikro. S-10 hat auch keinen Validierungstest, S-13 ebenfalls nicht. Der Satz in §4.1 ("K4 gilt für makro nur offline") steht nicht in der K4-Zeile.
- K3 sagt "auch mit der stärksten Waffe" und `--matrix`; der S-13-Test (Punkt 1) prüft nur "Default-Waffe". Der Test (weapon_lock None, tests/playtest_tiers.rs:14) ist damit schwächer als das Kriterium (OP-007: Hitscan war der schlimmste Fall). "Stärkste Waffe" ist nicht definiert.
- K1 als Test: 1 Seed, 4 min, vier Stufen streng monoton. Das ist rauschanfällig; die heutige Test-Aussage ist nur expert > novice (playtest_tiers.rs:38-53). Ob casual < skilled heute überhaupt gilt, ist nicht gemessen (S-07 soll es zeigen; gut).
- K6: Die Vorlage hat "Flow (1-10): Start __, Ende __" (playtest-notes/VORLAGE.md); K6 sagt nur "Flow-Selbsteinschätzung >= 6/10", also welcher Wert? S-13 hängt nicht von H-1/H-2 ab (Diagramm hat H2 -> S13, der Text von S-13 nicht; H-1 fehlt im Graphen).
- Ohne eigene Maße fehlen Kriterien, die zum Auftrag "hält ihn möglichst lange im Kanal" gehören (PLAN.md:28), z. B. Stabilität (Flackern der Schwierigkeit) und Anpassung bei Skill-Wechsel innerhalb einer Session. Das ist eine Frage, nichts Erfundenes.
Vorschlag: Je Kriterium die Slice und den Testschritt nennen oder K streichen. K3-Test für die Waffe aus der Matrix. K4-Text an A-C anpassen. K6-Feld präzisieren.
Wer: Planer, Jannus (K4/K6/fehlende Kriterien).

**B5 | wichtig | PLAN.md:259 (S-07 Punkt 5), :307 (S-09b AK), :354 (S-13 Punkt 2), :40**
K7 (Reproduzierbarkeit) wird erst in S-13 getestet. Schon S-09b ("ohne Override sind alle bisherigen Zahlen gleich, Vergleich mit S-07-Baseline") und alle Vorher/Nachher-Reports (S-08, S-11) setzen exakte Wiederholbarkeit voraus. Außerdem existiert schon ein Replay-Test: src/playtest/agent.rs:401 `a_replay_is_identical_every_time`. Die Aussage in Zeile 40 ("per rg determin tests/ nicht gefunden") prüft nur tests/. Bots nutzen einen geseedeten xorshift (perception.rs:59-61), das ist nicht die Simulation, aber ein Teil der Wiederholbarkeit.
Vorschlag: K7-Test (zwei gleiche `play`-Läufe, `Summary` gleich) in S-07 statt S-13; agent.rs:401 erwähnen.
Wer: Planer.

**B6 | wichtig | PLAN.md:194 (T-A), :198, :227-231 (Graph), :238, :329-332 (S-11 Punkt 2), :345**
Reihenfolge und Abhängigkeit S-11/S-12 widersprüchlich. S-11 Punkt 2 "setzt S-12 mit H-B voraus", der Graph und die Parallel-Zeile führen S-12 aber nicht vor S-11; S-12 ist "nicht ready". Zusätzlich: T-A ist mit "braucht zwei Hebel mit verschiedener Wirkung (OP-007 H-B)" begründet. Heute gibt es aber schon drei Hebel (Anzahl, Tempo, Kontaktschaden; decide.rs:142-156) mit unterschiedlicher Wirkung ("hoch mikro, niedrig meso -> mehr Masse" geht mit `enemy_count`, Tempo/Schaden für Druck). Eine minimale T-A-Variante auf den bestehenden Hebeln fehlt als Option. Die Kopplung T-A -> H-B -> S-12 vor S-11 ist damit nicht zwingend, sondern eine Planer-Setzung.
Vorschlag: Option "T-A auf den vorhandenen drei Hebeln" aufnehmen und die Abhängigkeitskante S-12 -> S-11 (nur bei H-B) in den Graph.
Wer: Planer, Jannus (OP-019).

**B7 | wichtig | PLAN.md:256 (S-07 Punkt 2), :180 (OP-008 Empfehlung), :331 (S-11 Punkt 1)**
S-07 misst "bored, aber knapp" als Anteil der bored-Wellen mit `danger.close_calls > 0`. S-11 steuert aber über den Bedrohungszeit-Anteil (`threat_secs`/Wellendauer). Gemessen wird also nicht das, wonach später entschieden wird. Außerdem: Ein "close call" ist laut Danger-Definition (waves.rs:97-112, danger.rs:12-15) jeder Gegner, der auf 24 Einheiten herankommt und dann stirbt oder wegläuft. Bei vielen Gegnern ist `close_calls > 0` vermutlich fast immer wahr, das Maß trennt dann kaum. Nenner ("Anteil der bored-Wellen": an allen Wellen oder an den bored-Wellen?) ist nicht angegeben. Die Entscheidungsregel "wenn nennenswert viele, D-C, sonst D-A" hat keine Schwelle.
Vorschlag: In S-07 beide Größen (close_calls und threat_secs-Anteil) und den Nenner ausweisen; die Schwelle für "nennenswert" vor S-07 von Jannus festlegen lassen.
Wer: Planer, Jannus (Schwelle).

**B8 | wichtig | PLAN.md:53, :55, :253-258, :269-277, :307, :189**
Messbasis driftet: S-07 misst mit dem S-04-Mix, S-08 stellt den Default danach auf den Wellenplan um. Die Schwellen K2/K3 werden "nach S-07" festgelegt (Zeile 53), gelten aber laut Zeile 55 für "Standard-Gegnermix bzw. Wellenplan", also teils für eine andere Mix-Welt. S-09b hängt nur an S-07 (Graph), vergleicht aber "alle bisherigen Zahlen gleich (Vergleich mit S-07-Baseline)"; ist S-08 vorher gemergt, ändert sich der Default und der Vergleich schlägt fehl. H-A/H-B-Empfehlung (Zeile 189) stützt sich auf die S-07-Zahl ohne Staged-Mix.
Vorschlag: Entweder S-07 mit beiden Mixen (S-04-Mix und Staged) als Baseline ausweisen und Schwellen nach S-08 festzurren, oder S-09b mit explizit gepinntem `--enemies`. Reihenfolge klarstellen.
Wer: Planer.

**B9 | wichtig | PLAN.md:158, :161 (M-B), :286-288 (S-09a Punkt 2), :304 (S-09b Punkt 3); src/enemies/movement.rs:16, :99; src/playtest/bot.rs:228**
Meso-Zahl hat zwei Schwächen. (1) "Ausgewichen = Tell endete, ohne dass dieser Gegner einen Kontakttreffer landete": Der Charger-Tell startet bis 400 Einheiten Abstand (CHARGE_TRIGGER_RANGE); ein Dash, der den Spieler gar nicht gefährdet, zählt als "ausgewichen". Die Quote drängt Richtung 100 % und trennt schlecht. (2) Validierung in S-09b: Der Test "Skilled mit/ohne Tell-Lesen, Quote mit Tells höher" prüft die Metrik gegen genau den Bot-Parameter, der das Ausweichen verursacht (`reads_tells`, bot.rs:228). Das ist eine Selbstbestätigung, kein Nachweis, dass die Zahl menschliches meso misst; die 5-Prozentpunkte-Grenze für die Trefferquote soll das teilweise abfangen. Zu "Annahme Planer" markiert ist beides nicht.
Vorschlag: Definition von "ausgewichen" an den Abstand/die Dash-Linie zum Spieler binden (Inhaltsentscheidung, Vorschlag nur). Die Grenzen der Validierung im Plan benennen.
Wer: Planer, Code-Prüfer.

**B10 | wichtig | PLAN.md:291, :306, :290 (S-09a/S-09b Pfade)**
Pfadlisten unvollständig. `WaveReport`-Literale stehen außerdem in src/waves/transitions.rs:70-72 (Produktivcode, wird von S-09a nicht genannt), src/telemetry/session_line.rs:179 und :282, src/playtest/summary.rs:154, src/flow_director/decide.rs:174 (plus :214/:221, nicht auf `..` geprüft). Plan nennt für decide.rs nur :173-192. `SessionConfig`-Literale stehen in tests/playtest_tiers.rs:10, examples/playtest.rs:253, examples/strategy_search.rs:154; S-09b nennt nur tests/playtest_tiers.rs. Ein neues Feld bricht die beiden Beispiele. S-09a "Report-Ausgabe unverändert" gilt dann nur, wenn alle Stellen angefasst sind.
Vorschlag: Pfade ergänzen.
Wer: Planer.

**B11 | wichtig | PLAN.md:258, :264; examples/playtest.rs:123 (`minutes: 10.0`), :148-150, :253-262, :282-300**
S-07 Punkt 4: Befehl ohne `--minutes` läuft 10 simulierte Minuten je Session (K1-Test: 4). `--matrix` ergibt 4 Waffenvarianten (eigene + 3) x 4 Stufen x 3 Seeds = 48 Sessions je Mix, also 96 Sessions plus Wiederholung; Laufzeit ist laut Plan nicht gemessen (Zeile 239). Jeder Seed ist eine eigene Report-Zeile (`"{tier} #{seed}"`, playtest.rs:296-300), zusammengefasst wird nichts. "Deckel-Anteil je Stufe x Waffe" (K3) braucht daher eine Zusammenfassung über Seeds, die weder S-07 noch K3 festlegt.
Vorschlag: Minutenzahl und Aggregation (gepoolt oder je Seed) festlegen; Laufzeit in S-07 einmalig messen und festhalten.
Wer: Planer.

**B12 | wichtig | PLAN.md:183-189 (OP-007), :174-180 (OP-008), :153-171 (OP-009), :194-198 (OP-019), :200-212 (OP-018)**
Optionen. Fair: OP-018 (W-A/W-B/W-C; die Aussage "Wiederholung nach Tod behält den Index" stimmt, transitions.rs:101-103) und OP-009 meso (M-A/B/C). Schwächen:
- OP-007: Es fehlt ein Hebel, der kein Mix ist, z. B. Spawn-Intervall/Gleichzeitigkeit (`SPAWN_INTERVAL_SECS` = 0,35 ist heute Konstante, spawning.rs:13, kein Hebel). Er erweitert die Tiefe, ohne Oberflächenkomplexität. H-A wird mit "gegen das Leitbild" abgewertet, das ist eine Bewertung, kein Befund.
- OP-008: D-C ist nur eine Bremse; ein zweites Band auf der Danger-Achse (Hysterese, nicht additiv) fehlt als Gegenstück zu D-B. Empfehlung hängt an B7 (keine Schwelle für "nennenswert").
- OP-009 makro: A-A ist als "sofort messbar" empfohlen, hat aber laut Plan selbst keine Validierung (Bots lernen nicht) und ist von mikro nicht getrennt (siehe B4). Das schwächt die Empfehlung.
- OP-019: siehe B6.
Vorschlag: H-D und ein D-D als Optionen oder bewusst verwerfen und sagen, warum.
Wer: Planer, Jannus.

## Klein

**B13 | klein | PLAN.md:277 (S-08 AK: "ein unbekannter Name gibt weiter die Fehlermeldung"); src/enemies.rs:64-68**
Für `ARENA_ENEMIES` im Spiel gibt es keine Fehlermeldung. `parse().ok()` und `unwrap_or_default()` fallen still auf den Default zurück; die Meldung existiert nur für `--enemies` im Beispiel (kinds.rs:94-99). Mit geändertem Default wird ein Tippfehler im Spiel unbemerkt zum Wellenplan.
Vorschlag: AK auf `--enemies` begrenzen oder stilles Fallback bewusst festhalten.
Wer: Planer.

**B14 | klein | PLAN.md:3, :12, :220, :239-243, :367; docs/STAND.md:3; git log**
Stand veraltet/uneinheitlich: Plan sagt main = #25 + Doku #26 und S-06 "läuft" (Branch feat/s-06). Im Worktree liegt `playtest-notes/VORLAGE.md` (PR #27 gemergt, OP-020 nennt "S-06, #27"). CI-Zeit: Plan "ca. 15 min", STAND.md "ca. 16 min". Der Plan nennt K1-K9 und OP-009/018 "nicht entschieden", f919657 hat sie bereits als entschieden. Sync ist Sache des Leiters.
Vorschlag: Stand und S-06 aktualisieren, eine Zahl für die CI-Zeit.
Wer: Planer/Leiter.

**B15 | klein | Zeilenangaben (Stichproben)**
Geprüft und stimmig (>= 25): summary.rs:15-54/:56-130/:62/:146; decide.rs:56-61/:76-82/:97-138/:140-156/:173-192; difficulty.rs:10-53/:61-126; waves.rs:39-49/:69-95/:97-120/:154-165; enemies.rs:26-40/:64-69/:139-162; kinds.rs:56-65/:114-154; tier.rs:148-175/:192; report.rs:46-70; session_line.rs:35-37; agent_view.rs:251-258; movement.rs:50; transitions.rs:106-113; measure.rs:19; flow_director.rs:77-96; README.md:3-7/:52/:88/:171-177; ARCHITECTURE.md:166/:204-226; tests/playtest_tiers.rs:25-27/:38; examples/strategy_search.rs:75; examples/playtest.rs:184-186/:284/:349/:390.
Abweichungen: `kinds.rs:222-230` -> `kind_for_slot` steht in :223-231 (Doc-Zeile :222); `examples/playtest.rs:155` (--sweep 1..=10) -> :154. Beides sachlich unschädlich.
Wer: Planer.

**B16 | klein | PLAN.md:210 (6/10/14), :40, :59 (4 sim. Minuten)**
Nicht gerechnet, ob die Staged-Startwellen in den Test-/Report-Läufen erreicht werden. Grobschätzung (Annahme, nicht gemessen): Welle = 2,5 s Pause (transitions.rs:8) + Spawn n x 0,35 s (n = 4 + 2(d-1)) + Kampfzeit. Bei d = 3-5 ca. 10-20 s je Welle, also 12-24 Wellen in 4 min; Novice (Tode, gleicher Index) vermutlich weniger. S-07 weist `furthest_wave` bereits aus (summary.rs:23), erwähnt es aber nicht als Messgröße.
Vorschlag: In S-07 Wellen je Session/Stufe notieren; daraus prüfen, ob Brute (10) und Summoner (14) in K-Läufen überhaupt vorkommen.
Wer: Planer.

**B17 | klein | PLAN.md:317 (S-10 Punkt 1), :171**
S-10 überlässt dem Executor (Sonnet, medium) die Normierung des Risiko-Trends auf die Schwierigkeit ("begründet im PR"). Das ist eine Messdefinition, also Inhalt (CLAUDE.md: Produkt-/Designentscheidungen gehören Jannus).
Vorschlag: Normierung vor S-10 festlegen oder als offene Frage listen.
Wer: Planer/Jannus.

**B18 | klein | PLAN.md:329-340 (S-11), :331, :256; ARCHITECTURE-Regel; CLAUDE.md "Design north star"**
S-11 AK lässt aus: Wellendauer 0 (Division bei threat_secs/duration_secs), Auswirkung auf `Summary::in_band` (zählt nur `HeldInBand`, summary.rs:107-109), die neue `DecisionReason` muss in `DECISIONS`/`decision_name`/`parse_decision` (session_line.rs:228-238) eingetragen werden (Pfad ist genannt, Tabelle nicht). Außerdem wird das CLAUDE.md-Kriterium "Tiefe ohne Oberflächenkomplexität" bei T-A/T-B/D-C nicht explizit bewertet (sieht der Spieler mehr? nur Overlay?). Modellwahl: Opus nur S-11/S-12 ist für Kernlogik nachvollziehbar; S-12 ist aber noch nicht geschnitten (könnte bei H-C entfallen), die Zuordnung ist vorläufig. S-09a (G1/G8, Sonnet high) ist die riskanteste Nicht-Opus-Slice; mit Kritikrunde abgedeckt, kein Mangel, nur Hinweis.
Wer: Planer.

**B19 | klein | PLAN.md:253-365 (Aufwand "ca. 25-45 min")**
Aufwände sind in Minuten, nicht Tool-Calls; die Frage "30-45 Tool-Calls" lässt sich daran nicht prüfen. Am oberen Rand/eng: S-07 (drei Report-Läufe, B11), S-08 (Code in 4 Beispiel-Stellen, FromStr/Display, README + ARCHITECTURE, Report vorher/nachher), S-09a (>= 7 Literal-Stellen, B10, neue Datei, Round-Trip). Kein Zyklus im Abhängigkeitsgraph; Vorgriff nur wie in B6/B8. Parallel-Aussage (S-08 ∥ S-09a) hat ein Datei-Risiko: S-09a ändert Test-Literale in summary.rs, die S-07 ebenfalls ändert (S-07 ∥ S-09a ist nicht ausgeschlossen, Zeile 238 schließt nur S-07 ∥ S-09b ∥ S-10 aus).
Vorschlag: S-07 ∥ S-09a in die Nicht-parallel-Liste, ggf. S-08 oder S-07 teilen.
Wer: Planer.

## Positiv (belegt)
- Reihenfolge "Baseline vor Features" (S-07 vor S-08/S-11/S-12) folgt CLAUDE.md "Measure before restructuring".
- Quellenkennzeichnung §7: GRACE_SECS 0,75 (combat.rs:24) und "Jannus chose both levers" (Commit 6ce6a53) stimmen; Entscheidungen 2026-11-10 sind in OP.md belegt; Vorschläge sind als Vorschlag/Annahme markiert. Chat-Quellen (Flow-Modell, 2026-11-10) kann ich nicht prüfen.
- Determinismus/FixedUpdate/fx: Der Plan berührt keine RNG in der Simulation, registriert S-09a in `waves` (SimSet::Progress), `fx` bleibt unverändert; `waves` liest Gegner-Komponenten schon (danger.rs:7, :30), die Frage nach neuem Fakt ist in S-09a offen benannt.
- Rest-Wahrheiten im Plan selbst: keine Treffer der verworfenen Begriffe in neuem Text außer als Zitat in §8.

# Playtest YYYY-MM-DD: Titel (Name)

**Kurzcharakter der Session.** Eine Zeile: volle Session oder kurzer Eindruck,
wie viele Personen, ob Session-Log vorhanden, ob die Fragen aus der Anleitung
(`/mnt/project-files/playtest/anleitung.md`) gestellt wurden. Als Eindruck
lesen, nicht als Messung.

## Rahmen (vor dem Spielen ausfüllen)

| Feld | Wert |
|---|---|
| Datum | YYYY-MM-DD |
| Dauer | ca. __ min |
| Build | `cargo run` / `cargo play` (Feature `fx`) / Branch oder PR: __ |
| `ARENA_ENEMIES` | unset (nur Grunts) / `grunt` / Typname / `all` |
| `ARENA_PICKUPS` | unset (classic) / Name |
| `ARENA_SWORD` | unset / `right` / `key3` |
| **Lust vorher (1-10)** | __ (vor dem ersten Start abfragen, nicht danach schätzen) |
| Session-Log | ja / nein (`playtests/session-<unix time>.jsonl`, entsteht beim Spielen im Fenster; auswertbar mit `cargo run --example playtest -- --human`) |

## Was berichtet wurde

Gespielt: Varianten, Waffe(n), Gegnermix, Besonderheiten der Session.

- **Gegnermix:** (welche Typen, Eindruck)
- **Waffe:** (welche benutzt, gewechselt?, warum)
- **Flow (1-10):** Start __, Ende __, Difficulty-Wert aus dem Overlay __
- **Sound/Partikel, Pickups, Neues:** (nur falls relevant)
- **Wunsch / Auffälliges:** (Freitext)

## Was es misst und was nicht

| Aussage | Hebel / Kriterium | Skill-Art | Verlässlichkeit |
|---|---|---|---|
| (Aussage in den Worten der Person) | Entscheidungen / Druck / Ungewissheit / Konsequenz / Immersion | Makro / Meso, mechanisch | Personen, Dauer, Log ja/nein, Störfaktoren (andere Änderungen am selben Abend), Deckung mit Bot-Befund oder `pickups/entscheidungen.md` |

Nicht abgedeckt: (Konsequenz, längere Sessions, Lust/Müdigkeit vorher,
Lautstärken, ...).

## Offene Fragen

- (Frage, die der Test offen lässt; nichts erfinden)

## Folge

- (Optional: PR, Entscheidung, Branch; Feature-Stand, z. B. `fx` standardmäßig aus)

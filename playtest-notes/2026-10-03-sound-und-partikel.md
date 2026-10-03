# Playtest 2026-10-03: Sound und Partikel (Jannus)

**Kurze Session, ca. 10 Minuten.** Subjektiv, eine Person, kein Session-Log,
ohne vorher die Lust (1-10) abzufragen. Als Eindruck lesen, nicht als Messung.

Gespielt: Branch von PR #20 mit `cargo play` (Feature `fx`), dazu am selben
Abend der neue Gegnertyp, das Schwert auf Rechtsklick und das Sammeln von
Pickups (eigene Threads).

## Was Jannus berichtet hat

- **Sound und Partikel:** "schon mal sehr gut". Flow von ca. 4 auf 6-7,
  ca. 10 Minuten gespielt und "war schon gut". Definitiv behalten, aber
  getrennt im Build.
- **Schwert auf Rechtsklick:** gefällt, "passt einfach gut wie man es benutzt".
- **Neuer Gegnertyp:** noch nicht so gut, muss mehr ausprobiert werden.
- **Pickups sammeln:** guter Start für Strategie, aber noch keine echte
  Langzeit-Strategie (Makro).

## Was es misst und was nicht

| Aussage | Kriterium | Verlässlichkeit |
|---|---|---|
| Flow 4 -> 6-7 mit Sound und Partikeln | Immersion (eigenes Kriterium, getrennt von Makro/Meso/Mechanik) | eine Person, eine Session; andere Änderungen am selben Abend mitgespielt, der Anteil von fx ist also nicht sauber getrennt |
| Pickups noch keine Strategie | Entscheidungsdichte, Makro | Eindruck, deckt sich mit `flow_arena/pickups/entscheidungen.md` |

Nicht abgedeckt: ob der Charger-Ton beim Ausweichen hilft (Meso), ob ein
Sound nervt, Lautstärken. Die Sounds sind noch die selbst erzeugten
Platzhalter, keine CC0-Samples.

## Folge

- PR #20 auf main (2026-10-03). `fx` bleibt ein Feature, standardmäßig aus;
  Bots, Headless und Zuschau-Fenster laden es nie.

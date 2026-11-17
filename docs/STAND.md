# STAND Arena-flow – zuerst lesen (nach Kontextverlust / Sessionstart)
## Stand
main = letzter Merge #27 (2026-11-17), Playtest-Vorlage `playtest-notes/VORLAGE.md` (S-06) ist da. Test-CI (`cargo test`) läuft auf jedem PR und Push nach main, Lauf auf main grün (ca. 16 min). Code unverändert seit #22: Wave-Arena-Testaufbau mit Flow-Director, 6 Gegnerarten, Pickup-Prototypen, Playtest-Bots, Sound hinter `fx`.
Richtung (OP-001): V1 = fertiger Testaufbau für den Flow-Director, danach Spiel.
## Letzte Merges
- #27 S-06 Playtest-Vorlage (2026-11-17)
- #26 Doku: STAND nach #25, OP-014 teilweise erledigt (2026-11-17)
- #25 CI: cargo test on every PR and push to main (2026-11-10)
- #24 Doku: Entscheidungen Jannus 2026-11-10 (OP-001, OP-002, OP-003), Flow-Modell, STAND nach #23 (2026-11-10)
## Offene PRs
- #29 V1-Plan (r4 nach Kritik R2: 0 blockierend, 6 wichtig → eingearbeitet)
- #30 S-04 Standard-Gegnermix (Code-Prüfer grün, Risiko ja → Kritik vor Merge)
- #28 dieser Doku-PR
## Laufend (seit 2026-11-17 13:07 UTC)
- Kritiker R3 (Ready-Gate, Opus): V1-Plan PR #29 nach Planer r4
- Kritiker (Opus): Risiko-PR #30 S-04 (Code-Prüfer grün, Risiko Nr. 3 Balance)
## Nächste Slice
Nach Merge S-04: S-07 Baseline-Messung (Plan r1), danach S-08 Wellenplan (OP-018 entschieden). S-01 (README) erst nach Merge von S-04 (beide ändern README.md).
## Offene Entscheidungen
blockierend: OP-003, OP-012 (Jannus 2026-11-17: bewusst später, Planer plant Pickups als offenen Punkt) · gesamt offen: 29 (Details in docs/OP.md)
## Letzte Kritikrunde
R2 V1-Plan 2026-11-17: ok mit Befunden (0 blockierend, 6 wichtig, 6 klein); R1: nicht ready · Merges seit dann: 5

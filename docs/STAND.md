# STAND Arena-flow – zuerst lesen (nach Kontextverlust / Sessionstart)
## Stand
main = Merge #30 (2026-11-17) + Doku #28/#29: Standard-Gegnermix Grunt + Charger + Swarm ist Default (S-04), Playtest-Vorlage (S-06, #27), V1-Plan in docs/PLAN.md (Planer r4, 3 Kritikrunden; Befunde in docs/kritik/). Test-CI (`cargo test`) auf jedem PR und main.
Richtung (OP-001): V1 = fertiger Testaufbau für den Flow-Director, danach Spiel.
**Stopp für heute (Jannus 2026-11-17).** Keine laufenden Sub-Agents, keine offenen Branches außer diesem Doku-Branch.
## Letzte Merges
- #29 V1-Plan (Planer r1–r4) (2026-11-17)
- #28 Doku: STAND/OP, Entscheidungen 2026-11-17 (2026-11-17)
- #30 S-04 Standard-Gegnermix als Default (2026-11-17)
- #27 S-06 Playtest-Vorlage (2026-11-17)
- #26 Doku: STAND nach #25 (2026-11-17)
## Offene PRs
- keine
## Nächste 3 Schritte
1. Planer r5 (Doku): Kritik R3 aus docs/kritik/kritik-arena-flow-plan-r3.md einarbeiten – Plan auf Stand #30 (Mix `standard`, verschobene Zeilen kinds.rs/playtest.rs für S-08, K9-Treffer README.md:53, S-01 „Fertig wenn“), §4.6 auf OP-029/OP-030 neu, OP-034–OP-038 statt R2-x, Ready-Gate-Haken je Slice.
2. S-02 Rest (OP-014 entschieden): fmt + clippy in CI; vorher prüfen, ob clippy auf main grün ist.
3. Danach Teilstart laut Kritik R3 ready: S-07 Baseline (mit Grunt-Bezugslauf, mehreren Seeds, OP-039/OP-041), S-01 README, S-10; S-08 nach Pfad-Update; S-04b Folge-Slice (OP-040). S-09a/b erst nach r5 (OP-030 jetzt entschieden).
## Offene Entscheidungen
blockierend: OP-003, OP-012 (bewusst später) · nach S-07 vorlegen: OP-007, OP-008, OP-019 · gesamt offen: 31 (Details in docs/OP.md)
## Letzte Kritikrunde
R3 V1-Plan 2026-11-17 (Ready-Gate): Teilstart ok für S-01, S-07, S-10 (S-08 nach Pfad-Update), 1 blockierend (OP-030, inzwischen entschieden), 5 wichtig, 6 klein · Kritik PR #30: ok mit Befunden · Merges seit dann: 3 (#30, #28, #29)
## Hinweise Umgebung
- Cloud-Container: `claude-limit-check` fehlt; 5h-Limit wurde am 2026-11-17 gegen 13:50 UTC erreicht (Opus-Kritiker), Reset 14:20 UTC.
- Bevy-Build braucht System-Libs (wayland, alsa, udev, xkbcommon) per apt; erster Build ca. 30+ min.

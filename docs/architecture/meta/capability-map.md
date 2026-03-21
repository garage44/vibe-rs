# Capability map

Themes that connect **vision goals** to **ADRs** and **gaps**. Status is a coarse label; adjust as the project moves.

**Legend:** done = accepted ADR + meaningful implementation path; partial = ADR exists or work in progress; gap = not yet covered by an ADR / not started.

| Theme | Goals | Status | ADRs (examples) | Notes |
|-------|-------|--------|-----------------|--------|
| World model & geo | G-04, G-02 | partial | [ADR-006](../adr/006-world-coordinate-and-osm-anchor.md), [ADR-004](../adr/004-osm-tile-integration.md) | Coordinates, tiles, regions |
| Interest & replication | G-02 | partial | [ADR-012](../adr/012-interest-management-and-osm-tiles.md), [ADR-011](../adr/011-static-world-replication-v0.md) | AOI, static world v0 |
| Networking & envelope | G-02, P-02 | partial | [ADR-008](../adr/008-network-transport-layer.md), [ADR-009](../adr/009-application-protocol-envelope-v0.md) | Transport, app framing |
| Sim vs client | G-02, P-05 | partial | [ADR-007](../adr/007-simulation-vs-client-process-model.md) | Process split |
| Avatars & authority | G-02, G-03, P-01 | partial | [ADR-010](../adr/010-authoritative-avatar-state-v0.md) | Server-led avatar v0 |
| Persistence & migrations | G-01, G-02 | partial | [ADR-002](../adr/002-sqlite-storage.md), [ADR-013](../adr/013-sqlite-migrations-and-server-writer.md) | SQLite, single writer |
| Runtime & ops | P-03 | partial | [ADR-014](../adr/014-runtime-configuration-and-operations.md) | Config, operations |
| Workspace boundaries | P-05 | partial | [ADR-015](../adr/015-workspace-module-boundaries.md) | Crate layout |
| Rendering & client UX | G-01, G-03 | partial | [ADR-001](../adr/001-bevy-game-engine.md), [ADR-003](../adr/003-ecs-architecture.md), [ADR-005](../adr/005-sky-lighting-system.md) | Bevy, ECS, atmosphere |
| Asset pipeline & storage | G-01, G-05 | gap | — | Coherent asset server / CDN-style story TBD |
| AuthN / AuthZ | G-05, P-04 | gap | — | Who can edit what |
| Chat & social presence | G-03 | gap | — | Beyond avatar state |
| Voice / WebRTC / video | G-03, P-07 | gap | — | After session + abuse basics |
| LLM-assisted & in-world generation | G-06, P-06 | gap | — | Constrained action surface TBD |
| Headless / GNSS-linked clients | G-07, P-07 | gap | — | Privacy + accuracy TBD |
| Scripting & integrations | G-08, P-04 | gap | — | e.g. automation hooks to external systems |

## Related

- [vision.md](vision.md)
- [principles.md](principles.md)
- [../adr/index.md](../adr/index.md)

# Strategic foundation (meta)

This directory holds **slow-moving** intent and rules. **`docs/architecture/adr/`** holds **point-in-time** decisions. Together they form the architecture spine: **meta constrains and prioritizes; ADRs record what we chose.**

## Read order (humans and agents)

1. [vision.md](vision.md) — mission, goals (`G-*`), non-goals
2. [principles.md](principles.md) — engineering principles (`P-*`)
3. [capability-map.md](capability-map.md) — themes, coverage, gaps
4. [glossary.md](glossary.md) — shared vocabulary (optional)
5. [adr/index.md](../adr/index.md) — decision log

Then code and existing ADRs in context.

## Discipline

### When an ADR is required

| Change | ADR? |
|--------|------|
| Wire protocol, authority model, persistence schema, or security boundary | Yes — new or superseding ADR |
| New major dependency or process model (sim vs client, new crate boundary) | Yes |
| Behavior already covered by an existing ADR | No — code only; extend ADR if semantics shift |
| Bug fix, perf tweak, refactor inside established ADR scope | No |
| Routine dependency bumps within the same stack | No |

If unsure, prefer a **short ADR** or an explicit addendum to the nearest ADR over silent drift.

### Definition of done (ADR-worthy PRs)

1. **`Aligns with`** in ADR metadata lists relevant `G-*` / `P-*` from [vision.md](vision.md) and [principles.md](principles.md), or states **`N/A`** with one line why (e.g. local refactor under ADR-NNN).
2. **`adr/index.md`** updated in the same change set as a new ADR file.
3. If work **materially advances** a capability row, update [capability-map.md](capability-map.md).

### Conflicts with principles

If a decision **contradicts** a `P-*` principle, either:

- Update **principles.md** with rationale and date, or
- Document a **time-boxed exception** in the ADR (principle ID + why).

## Files

| File | Purpose |
|------|---------|
| [vision.md](vision.md) | Mission, lineage, `G-*` goals, non-goals |
| [principles.md](principles.md) | `P-*` durable rules |
| [capability-map.md](capability-map.md) | Themes ↔ ADRs ↔ status |
| [glossary.md](glossary.md) | Domain terms |

## See also

- [Architecture overview](../README.md)
- [ADR template](../adr/template.md) — includes **Aligns with**

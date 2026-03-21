# Instructions for AI assistants

This file explains **how architecture documentation works** in vibe-rs so agents stay aligned with **intent** (meta), **decisions** (ADRs), and **code**.

## Layers (what goes where)

| Layer | Location | Role |
|-------|----------|------|
| **Strategic foundation** | `docs/architecture/meta/` | **Why** and **rules**: mission, goals (`G-*`), principles (`P-*`), capability map, glossary. Changes rarely. |
| **Decision log** | `docs/architecture/adr/` | **What we chose** for a specific problem: context, decision, alternatives, consequences. One ADR per major fork. |
| **Implementation** | `crates/` | Code must **respect** ADRs and **align** with `G-*` / `P-*` when work is architectural. |

**Meta constrains and prioritizes; ADRs record choices; code implements.** Do not duplicate the full mission inside every ADR—cite **`Aligns with: G-…, P-…`** instead.

## Read order (before large or cross-cutting work)

1. [`docs/architecture/meta/vision.md`](docs/architecture/meta/vision.md) — `G-*` goals, non-goals
2. [`docs/architecture/meta/principles.md`](docs/architecture/meta/principles.md) — `P-*` principles
3. [`docs/architecture/meta/capability-map.md`](docs/architecture/meta/capability-map.md) — themes, existing ADRs, **gaps**
4. [`docs/architecture/adr/index.md`](docs/architecture/adr/index.md) — full ADR list
5. Relevant ADR files and then the crates they mention

Full discipline (when an ADR is required, definition of done): [`docs/architecture/meta/README.md`](docs/architecture/meta/README.md).

## Identifiers

- **`G-01` … `G-08`** — Outcome goals in `meta/vision.md`. Use in plans and in ADR **Aligns with** when the decision advances that outcome.
- **`P-01` … `P-07`** — Durable engineering rules in `meta/principles.md`. Use in **Aligns with**; if you must violate one, call it out and follow the conflict process in `meta/README.md`.
- **`ADR-NNN`** — Point-in-time decisions; cite them when touching the same subsystem.

## When to add or update an ADR

**Create or supersede an ADR** when changing wire protocol, authority, persistence schema, security boundaries, major dependencies, or sim/client/process boundaries.

**Skip a new ADR** for bug fixes, refactors, or performance work **inside** an existing ADR’s scope, and for routine dependency bumps that do not change architecture.

When in doubt, prefer a **short ADR** over silent drift. See the trigger table in [`meta/README.md`](docs/architecture/meta/README.md).

## New ADR checklist

1. Copy [`docs/architecture/adr/template.md`](docs/architecture/adr/template.md).
2. Set **Aligns with** to relevant `G-*` and `P-*`, or **`N/A`** plus one line (e.g. refactor under ADR-010).
3. Add a row to [`docs/architecture/adr/index.md`](docs/architecture/adr/index.md) in the **same change set** as the new file.
4. If capability status changes meaningfully, update [`docs/architecture/meta/capability-map.md`](docs/architecture/meta/capability-map.md).

## Cursor rules

- **ADRs + meta**: `.cursor/rules/adr.mdc` — consult when making architectural proposals.
- **Build scope**: `.cursor/rules/build-and-iteration.mdc` — prefer scoped `cargo check -p …` over full workspace builds when possible.

## Human-oriented docs

- Project overview and run instructions: [`README.md`](README.md).
- Architecture entry and links: [`docs/architecture/README.md`](docs/architecture/README.md).

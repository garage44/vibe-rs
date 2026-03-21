# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records for vibe-rs, documenting architectural decisions and technology choices. ADRs help maintain consistency and provide context for future development.

The **strategic foundation** (`docs/architecture/meta/`) defines **goals (`G-*`)** and **principles (`P-*`)**. New ADRs should set **Aligns with** in metadata (see `template.md`).

## Quick Start for AI Assistants

**Before making architectural decisions:**

1. Read [meta/vision.md](../meta/vision.md) and [meta/principles.md](../meta/principles.md)
2. Skim [meta/capability-map.md](../meta/capability-map.md) for gaps and linked ADRs
3. Search ADRs in `index.md` for relevant decisions
4. Cite ADRs and meta goal/principle IDs in plans
5. Create new ADRs for significant decisions using `template.md` (**Aligns with** `G-*` / `P-*` or `N/A` with reason)

**After successful implementation:**

- Document significant decisions using `template.md`
- Add new ADRs to `index.md`

## ADR Format

Each ADR contains:

- **Context**: Background and problem statement
- **Decision**: What was decided
- **Rationale**: Why this decision was made
- **Consequences**: Pros, cons, risks, trade-offs
- **Related ADRs**: Links to related decisions

## Location

- **Meta (vision & principles)**: `docs/architecture/meta/`
- **Index**: `docs/architecture/adr/index.md`
- **Template**: `docs/architecture/adr/template.md`
- **Individual ADRs**: `docs/architecture/adr/XXXX-topic.md`

## Rust ecosystem (implementation hints)

Server and protocol ADRs (**006–015**) include **recommended crates** where they help: e.g. **`tokio`** + **`tokio-util`** framing, **`postcard`**/`serde` for payloads, **`refinery`** + **`rusqlite`** for migrations, **`figment`** + **`clap`** for config, **`tracing`** for observability. Prefer **workspace dependency** versions in `[workspace.dependencies]` (ADR-015) instead of duplicating versions per crate.

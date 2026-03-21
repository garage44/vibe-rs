# Architecture

Architecture documentation for vibe-rs.

**AI assistants:** workflow and read order are summarized in the repo root [AGENTS.md](../../AGENTS.md).

## Read order

1. **Strategic foundation** — [meta/](meta/) (intent, `G-*` / `P-*`, capability map)
2. **Decision log** — [adr/](adr/) (ADRs)

## Strategic foundation (meta)

Slow-moving **vision**, **principles**, and **discipline** live in [docs/architecture/meta/](meta/).

- **Overview & rules**: [meta/README.md](meta/README.md)
- **Vision & goals (`G-*`)**: [meta/vision.md](meta/vision.md)
- **Principles (`P-*`)**: [meta/principles.md](meta/principles.md)
- **Capability map**: [meta/capability-map.md](meta/capability-map.md)
- **Glossary**: [meta/glossary.md](meta/glossary.md)

## Architecture Decision Records (ADRs)

Point-in-time decisions are documented in [docs/architecture/adr/](adr/).

- **Index**: [adr/index.md](adr/index.md) – List of all ADRs
- **Template**: [adr/template.md](adr/template.md) – Template for new ADRs

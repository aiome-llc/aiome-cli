# Aiome: For Human Readers

Welcome. This document is for humans who want a quick, honest overview.

## What is Aiome?

Aiome is a CLI that automatically generates, maintains, and serves multi-scale knowledge about a codebase — so that AI
agents get exactly the right context for any task.

The core idea: AI agents have limited context windows and no memory between sessions. Hand-written context files
(CLAUDE.md, .cursorrules) work for small projects but don’t scale. Aiome analyzes your codebase’s structure, identifies
natural subsystem boundaries, generates knowledge at multiple resolutions, and serves the right resolution for each
task.

## Current State

Early stage. The CLI is a REPL that echoes input. The architecture is designed
([docs/design-doc.md](./docs/design-doc.md)) but not yet implemented. Next step is Phase 1: static and temporal
coupling analysis.

## Quick Start

```bash
cargo run                   # Start the REPL
cargo install --path .      # Or install locally as `aiome`
```

Ctrl+D to exit.

## Where to Look

- **[README.md](./README.md)** — Project constitution (conventions, commands, repo layout)
- **[docs/design-doc.md](./docs/design-doc.md)** — Architecture: Analyze → Generate → Serve
- **[docs/vision.md](./docs/vision.md)** — Foundational ideas (BRAIN pattern, stigmergy, cognitive scaffolding)
- **src/main.rs** — The current CLI entry point

## The Name

"Aiome" = AI + microbiome. Like gut bacteria, AI agents form a symbiotic ecosystem with their human host. You didn’t
create them, you don’t fully control them, but you’re better together.

# Aiome

> **Human readers**: See [HUMANS.md](./HUMANS.md) for a gentler introduction.

Aiome automates multi-scale knowledge infrastructure for codebases — so that any AI agent, at any moment, can load
exactly the context it needs at exactly the right resolution. See [docs/design-doc.md](./docs/design-doc.md) for the
full architecture. See [docs/vision.md](./docs/vision.md) for the foundational ideas.

---

## Current State

The CLI is a REPL that echoes input. Everything below is direction, not implemented.

The next milestone is Phase 1 (Analyze): static + temporal coupling analysis producing an entanglement map and partition
structure. No LLM needed. See the design doc’s "CLI" section for the phased build plan.

---

## Repository Layout

```
├── Cargo.toml              # Rust 2024 edition, rustyline dependency
├── src/
│   └── main.rs             # CLI entry point: REPL loop
├── docs/
│   ├── design-doc.md       # Architecture: Analyze → Generate → Serve
│   └── vision.md           # Foundational ideas: BRAIN pattern, stigmergy, scaffolding
├── HUMANS.md               # Human-oriented overview
└── .github/workflows/
    └── ci.yml              # CI: fmt, clippy, test
```

---

## Conventions

- **Rust 2024 edition.** `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` must all pass. CI enforces this.
- **Small changes.** Prefer focused commits over sweeping refactors.
- **Coordination via artifacts.** Code, tests, issues, and PRs over chatty handoffs.
- **Honesty over aspiration.** If this document drifts from the code, update the document.

---

## Commands

```bash
cargo run                   # Start the REPL
cargo test                  # Run tests (none yet)
cargo fmt                   # Format code
cargo clippy                # Lint
cargo install --path .      # Install locally as `aiome`
```

---

## What’s Next

Phase 1 of the design doc: `aiome init` producing an entanglement map and partition structure from static analysis
(tree-sitter) and git history (git2). Key dependencies to add: `tree-sitter`, `git2`, `petgraph`, `clap`.

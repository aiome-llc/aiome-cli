# Aiome

> **Human readers**: See [HUMANS.md](./HUMANS.md) for a gentler introduction.

Aiome is an early-stage CLI exploring how multiple AI agents can collaborate on software work. This README is
intentionally lightweight: it sets direction without pretending the project is further along than it is.

---

## Current State (Reality Wins)

Right now this repository is small and experimental. The CLI is a simple REPL that echoes input. Anything beyond that is
directional, not guaranteed.

If this document drifts from the code, update the document.

---

## Direction (Not Mandates)

These are the ideas we want to grow toward, without locking ourselves into a rigid process:

- **Roles are verbs.** Treat agents as functions, not personas.
- **Bounded work.** Keep scope small, context limited, and changes reversible.
- **Coordination via artifacts.** Prefer code, tests, issues, and notes over chatty handoffs.
- **Transparency.** If you’re unsure, say so and leave a clear trail.
- **Locality.** Touch what you must, avoid sweeping refactors.

---

## Potential Roles (Names May Change)

These are placeholders for how we might structure work over time:

- **plan** — identify work worth doing
- **implement** — make a focused change
- **review** — check correctness and scope
- **debug** — fix a concrete issue
- **refactor** — improve structure without behavior changes
- **retrospect** — reflect and propose process tweaks

Treat these as a vocabulary, not a contract.

---

## Working Here (Lightweight)

- Prefer small, readable changes over perfect systems.
- If you add tests, run `cargo test`. If you add formatting or linting conventions, document them.
- If you want to introduce new process or guardrails, open an issue first so humans can weigh in.

---

## Repository Snapshot (Today)

This is the current layout and may evolve:

```
aiome-cli/
├── Cargo.toml
├── src/
│   └── main.rs
├── HUMANS.md
└── README.md
```

---

## Final Note

Let the structure emerge from the work. Keep the docs honest. Make it easy for the next person to understand what
changed and why.

# Aiome: A Renormalization Engine for Codebases

## The Problem

AI coding agents are boundedly rational. Finite context windows, no memory across sessions, no understanding of your
project's architecture or history. The current solution is hand-written context files — CLAUDE.md, .cursorrules,
AGENTS.md — that encode project knowledge in a format agents can consume at session start.

This works at small scale. A 1,000-line project fits in a single file. A 100,000-line project does not.
[Vasilopoulos (2026)](https://arxiv.org/abs/2602.20478) documents what happens next: you end up building a three-tier
knowledge hierarchy by hand — a constitution loaded every session, 19 specialized agent specs, 34 subsystem documents —
totaling 26,000 lines maintained at 1-2 hours per week, with stale specs as the primary failure mode.

That hierarchy is correct. The manual maintenance is not sustainable.

Aiome automates the hierarchy. It analyzes your codebase's structure, identifies where the natural boundaries are,
generates knowledge at multiple resolutions, maintains it as the code evolves, and serves the right resolution to the
right agent for the right task.

## Why This Design Works

The knowledge hierarchy isn't an arbitrary organizational choice. It's a consequence of how codebases are structured and
how bounded agents consume information.

**Codebases have entanglement structure.** Some files are tightly coupled — they share types, change together, fail
together. Others are nearly independent. This structure determines what an agent needs to know: if you're modifying a
file in the networking subsystem, you need to understand the determinism invariants that cross into the RNG and
persistence modules. You don't need to know how the UI layout works. The entanglement map tells you which knowledge is
relevant to which task.

**Agents need knowledge at the right resolution.** A simple task in an isolated module needs a thin context: project
conventions and the module's interface. A complex task in a highly entangled domain needs a thick context: the full
theory of the domain, the cross-module interactions, the known failure modes. Loading too little context causes
mistakes. Loading too much wastes the context window on irrelevant information and can confuse the agent. The right
amount depends on the task's entanglement with the codebase.

**Knowledge at different scales preserves different things.** A subsystem spec preserves interfaces, invariants, and
failure modes while discarding implementation details. A domain spec preserves cross-subsystem theory while discarding
single-subsystem internals. A constitution preserves universal conventions while discarding everything domain-specific.
Each level compresses aggressively, keeping only what matters for decisions at that scale. This is the same principle
that makes scientific models work: you don't need quantum mechanics to design a bridge, but you do need it to design a
transistor. The resolution must match the problem.

**The hierarchy maintains itself through boundary detection.** Not every code change requires a knowledge update. An
internal refactor that preserves all interfaces doesn't change what external agents need to know — the boundary is
intact. A one-line type change that breaks a cross-module invariant does — the boundary is breached. By tracking
*boundary-relevant* changes rather than *any* changes, the maintenance cost drops from "update specs whenever code
changes" to "update specs when interfaces or invariants change." Most commits don't cross boundaries.

## Architecture

Three phases, each independently valuable:

```
ANALYZE ──→ GENERATE ──→ SERVE
  │              │             │
  │  no LLM      │  LLM        │  no LLM (usually)
  │  seconds     │  minutes    │  milliseconds
  │  every run   │  on init    │  every session
  │              │  + drift    │
  ▼              ▼             ▼
Entanglement   Knowledge     Context
map            hierarchy     packages
```

### Analyze

**Input:** Source code + git history.  
**Output:** Entanglement map + partition structure.

The analyze phase measures how the codebase's files are coupled, using signals that are always available:

**Static entanglement** — from the code itself. Parse every source file (via tree-sitter) to extract import edges, type
sharing across module boundaries, and interface surface area. Files that reference each other's types are entangled.

**Temporal entanglement** — from git history. Files that change together in the same commit are entangled. Files where a
change to one predicts a change to the other within N subsequent commits are entangled. Modules with correlated churn
rates share hidden dependencies.

These two signals are always present, deterministic, and cheap to compute. Two optional signals improve the map when
available:

**Causal entanglement** — from tests. Which tests exercise which source files? Which tests fail together? Coverage gaps
represent unknown entanglement.

**Experiential entanglement** — from agent session logs. Which files did agents read together to complete tasks? Which
domains caused repeated failures? This signal is only available after Aiome has been running for a while, creating a
feedback loop that improves partition quality over time.

The signals combine into a weighted graph — the **entanglement map** — where nodes are files and edge weights represent
coupling strength. Community detection on this graph (Louvain, spectral clustering) identifies the natural partition
boundaries: groups of files that are tightly coupled internally and loosely coupled externally.

Each partition gets an **entanglement score**: how much does this partition interact with the rest of the codebase? A
utility module with a clean interface has low entanglement. The networking layer that touches persistence, RNG, combat,
and UI has high entanglement. This score determines how much knowledge the partition needs at its boundary.

```
┌─────────────────────────────────────────────────────────┐
│              Entanglement Map (example)                 │
│                                                         │
│   ┌─────────┐         ┌─────────┐                       │
│   │   UI    │─ ─ ─ ─ ─│  Auth   │     weak coupling     │
│   │  (0.31) │         │  (0.44) │                       │
│   └────┬────┘         └─────────┘                       │
│        │                                                │
│   ┌────┴────┐   ══════╗                                 │
│   │  Game   │═════════║═══╗                             │
│   │  State  │   ║     ║   ║    strong coupling          │
│   │  (0.78) │   ║  ┌──╨───╨───┐                         │
│   └─────────┘   ║  │Networking│                         │
│                 ║  │  (0.91)  │                         │
│            ┌────╨──┴──┐       │                         │
│            │   RNG    │───────┘                         │
│            │  (0.85)  │                                 │
│            └──────────┘                                 │
│                                                         │
│  Scores = entanglement with rest of codebase            │
│  ═══ = high mutual information (need domain specs)      │
│  ─ ─ = low mutual information (subsystem specs suffice) │
└─────────────────────────────────────────────────────────┘
```

**Implementation:** tree-sitter for AST parsing, git2 for history analysis, petgraph for graph algorithms. All Rust, all
deterministic, runs in seconds on codebases up to ~500K lines. Phase 1 alone is useful as a codebase structure
visualizer even without the rest of the system.

### Generate

**Input:** Partition structure + entanglement scores.  
**Output:** Complete knowledge hierarchy.

This phase requires an LLM. It's the expensive step, run once on init and incrementally on drift.

**Tier 3 — Subsystem specs.** One per partition. For each partition, the LLM reads the constituent files and generates a
spec that captures:

- **Interfaces:** What does this subsystem expose? Function signatures, types, API contracts.
- **Invariants:** What must always be true? Extracted from assertions, test expectations, comments, and inferred from
  patterns.
- **Failure modes:** What has gone wrong? From reverted commits, bug-fix patterns in git history, known edge cases.
- **Design intent:** Why is it built this way? From commit messages, PR descriptions, architectural decision records.
- **Couplings:** Which other subsystems does it depend on, and at what interface?

The generation prompt embodies a specific principle: *extract the minimal representation that maximizes an agent's
ability to make correct decisions when working on adjacent code.* This is not "summarize the code." It's "what would a
competent new team member need to know to safely modify this subsystem without breaking anything downstream?"

**Tier 2 — Domain specs.** One per high-entanglement partition. When a partition's entanglement score exceeds a
threshold, a subsystem spec isn't rich enough. The networking subsystem that interacts with persistence, RNG, combat,
and UI needs a *domain model* — a coherent theory that synthesizes across multiple subsystem specs:

- The full domain theory (e.g., "all combat state must be deterministic; here's why and how")
- Cross-subsystem interaction patterns
- Accumulated symptom → cause → fix tables
- Pre-synthesized views that would otherwise require loading 4-5 subsystem specs

The threshold for promotion from Tier 3 to Tier 2 is the entanglement score. High entanglement means the partition's
boundary carries more information — agents working near it need richer context. The threshold is a tunable parameter (
default ~0.7), but the principle is fixed: richer boundary for more entangled partitions.

**Tier 1 — Constitution.** One per project. The most aggressive compression. From all specs and domain definitions,
extract only what's universal:

- Conventions that apply everywhere (naming, style, error handling patterns)
- The routing table: file patterns → which specs to load
- Cross-cutting invariants that span all subsystems
- Build, test, deploy commands

**Hard constraint:** The constitution must fit in ~1000 lines. It's loaded into every agent session, so every line must
earn its place. This budget forces real prioritization — the same way a limited context window forces agents to
satisfice, a limited constitution forces the system to compress.

Generation proceeds bottom-up: Tier 3 first (each partition independently, parallelizable), then Tier 2 (reads relevant
Tier 3 specs), then Tier 1 (reads everything). On first run, human review of generated specs is strongly recommended —
the first pass has no feedback signal on what agents actually need.

### Serve

**Input:** Task description (natural language, file paths, or both).  
**Output:** Context package at the right resolution.

The serve phase is a routing algorithm. Given a task, it determines which partitions are involved, computes the task's
entanglement with the codebase, and loads context proportional to that entanglement:

```
Task: "fix the desync bug in combat damage"
  ↓
Touched partitions:  combat, networking, rng
Task entanglement:   high (0.88)
  ↓
Context package:
  ✓ Tier 1: constitution.md              (847 lines)
  ✓ Tier 2: networking-domain.md         (915 lines)
  ✓ Tier 3: combat-system.md             (412 lines)
  ✓ Tier 3: deterministic-rng.md         (283 lines)
  Total: 2,457 lines (~6,100 tokens)
```

vs.

```
Task: "add a tooltip to the settings page"
  ↓
Touched partitions:  ui
Task entanglement:   low (0.31)
  ↓
Context package:
  ✓ Tier 1: constitution.md              (847 lines)
  ✓ Tier 3: ui-components.md             (218 lines)
  Total: 1,065 lines (~2,700 tokens)
```

The same system, different resolutions for different tasks. A simple task in an isolated module gets a thin context. A
complex task in a tangled domain gets everything relevant. The agent's context window is spent on what matters.

**Routing mechanics:** When file paths are provided, partition mapping is direct (glob patterns in the routing table).
When only a natural language description is given, keyword matching against partition names and spec content provides
the mapping. An LLM routing call is available as a fallback for ambiguous descriptions but is not the default path —
it's too expensive for a fast operation.

**Output formats:** The same hierarchy, multiple delivery formats:

| Format       | Delivery            | Use case              |
|--------------|---------------------|-----------------------|
| CLAUDE.md    | File in repo        | Claude Code sessions  |
| AGENTS.md    | File in repo        | Codex, Copilot        |
| .cursorrules | File in repo        | Cursor                |
| MCP server   | On-demand retrieval | Any MCP-capable agent |
| Raw markdown | Direct injection    | Custom tooling        |

The knowledge is agent-agnostic. The format is tool-specific.

## Maintenance

The ongoing maintenance loop is where Aiome becomes worth more than its setup cost. On every commit:

```
Changed files
    │
    ▼
Map to partitions
    │
    ├── Did the change cross a partition boundary?
    │     │
    │     ├── Modified a public interface (pub fn, exported type)?
    │     ├── Changed a function signature used by other partitions?
    │     ├── Added or removed a cross-partition import?
    │     │
    │     └── YES → flag spec for regeneration
    │
    └── Does the change contradict the existing spec?
          │
          ├── Spec says "uses reliable delivery" but code now uses unreliable?
          ├── Spec documents an invariant that the change violates?
          │
          ├── YES → flag spec for update
          └── NO  → no action needed (internal details changed, spec still valid)
```

**The key distinction:** A naive drift detector asks "did the code change without a spec update?" and fires on every
commit that touches a documented subsystem. Aiome asks "did boundary-relevant information change?" and only fires when
an interface, invariant, or coupling is affected. Most commits are internal to a partition. Internal changes don't break
the knowledge hierarchy. This is why the maintenance cost stays low as the codebase grows — the number of
boundary-crossing changes doesn't scale linearly with commit volume.

**Periodic re-analysis.** The entanglement map itself can drift. New modules emerge, old ones merge, coupling patterns
shift during major refactors. A weekly re-analysis (cheap — it's the deterministic Analyze phase) detects structural
drift and flags when the partition structure no longer reflects the codebase's actual boundaries. Full re-partitioning
is expensive (triggers regeneration) and should be explicit: `aiome init --reanalyze`.

## CLI

```bash
aiome init                                # analyze + generate full hierarchy
aiome status                              # partitions, entanglement scores, stale specs
aiome update                              # incremental regeneration for flagged specs
aiome context "refactor the auth module"  # compute optimal context package
aiome context --files src/net/*.rs        # context from file paths
aiome inspect --partition networking      # examine a specific partition's knowledge
aiome inspect --entanglement              # coupling heatmap
aiome export --format claude              # generate CLAUDE.md
aiome export --format mcp                 # generate MCP server config
aiome watch                               # continuous: monitor commits, flag drift
aiome run "implement the auth module"     # orchestrate: auto-route, auto-context, run agent
aiome run @claude "review the PR"         # orchestrate: specific agent
aiome run @all "design the API"           # orchestrate: all agents in parallel
```

**Phase 1** ships `init` (analyze only — no LLM, just the entanglement map and partition structure) + `status` +
`inspect`. This is already useful: a visual map of your codebase's coupling structure, the natural subsystem boundaries,
which modules are dangerously entangled. No LLM cost. Pure graph analysis.

**Phase 2** adds generation: `init` now produces the full hierarchy, `update` handles incremental regeneration, `export`
generates tool-specific context files.

**Phase 3** adds serving: `context` computes optimal packages, `watch` maintains integrity.

**Phase 4** adds orchestration: `run` routes tasks to agents with auto-generated context. This is the original Aiome
vision — a microbiome of AI agents coordinating through shared knowledge — but now with principled infrastructure
instead of hand-written routing tables.

## What Aiome Is Not

**Not RAG.** RAG retrieves chunks of code based on embedding similarity. Aiome generates *knowledge about code* — design
intent, invariants, failure modes — organized at multiple scales. The retrieval is entanglement-driven, not
embedding-driven. RAG answers "find me similar code." Aiome answers "what do I need to know to safely change this code?"

**Not a documentation generator.** Documentation is written for humans. Aiome generates specs written for AI agents —
file paths, function signatures, explicit do/don't rules, symptom-cause-fix tables. The audience is a bounded rational
agent that needs to make correct decisions, not a human that needs to understand the system.

**Not an agent framework.** AutoGen, CrewAI, LangGraph define how agents coordinate. Aiome structures the *knowledge*
that agents depend on. Any framework can consume Aiome's output.

**Not static.** The hierarchy is alive. Regenerated when structure shifts, updated when boundaries are breached, serving
different resolutions to different tasks. It's an engine, not a snapshot.

## Open Questions

**Partition stability.** Community detection algorithms can produce different partitions on small input changes. Should
we pin partitions and only re-detect explicitly? How do we handle files that sit on partition boundaries with high
coupling to multiple partitions? The current design treats re-partitioning as an explicit, infrequent operation, but the
right cadence is unknown.

**Cold start cost.** On `aiome init` for a 100K-line project with 12 partitions, the Generate phase makes 12+ LLM calls
reading thousands of lines each. What's the actual cost in time and money? Can we generate incrementally with useful
intermediate output? Should there be a "constitution-only" quick start mode?

**Spec quality signal.** How do we know a generated spec is good? The real test is downstream: does an agent make fewer
mistakes with the spec than without? But that requires running agent tasks end-to-end. Can we validate sooner — e.g.,
asking the LLM "given this spec, what would you do if asked to modify file X?" and checking the answer against
known-good approaches?

**Routing ambiguity.** When a task is described only in natural language, mapping to partitions requires interpretation.
How reliable is keyword matching? When is an LLM routing call justified? What's the failure mode when routing is wrong —
and can the agent detect that it has insufficient context?

**Cross-language boundaries.** Tree-sitter handles many languages, but cross-language coupling (Rust backend ↔
TypeScript frontend via API) isn't captured by import analysis. API contracts, shared type definitions, and OpenAPI
specs could serve as cross-language entanglement signals. Are cross-language boundaries always partition boundaries?

**The experiential feedback loop.** Agent session logs could improve partition quality over time — if agents
consistently need to load specs A and B together, maybe those partitions should merge or their coupling weight should
increase. But this creates a feedback loop between the knowledge hierarchy and the agents consuming it. How do we
prevent runaway drift? What's the right inertia?

## Implementation

### Dependencies

| Crate / Library | Purpose                               | Phase              |
|-----------------|---------------------------------------|--------------------|
| tree-sitter     | Multi-language AST parsing            | Analyze            |
| git2            | Git history analysis                  | Analyze            |
| petgraph        | Graph algorithms, community detection | Analyze            |
| tokio           | Async runtime                         | Serve, Orchestrate |
| clap            | CLI                                   | All                |
| LLM APIs        | Spec generation                       | Generate           |

The Python question: Louvain and spectral clustering have mature Python implementations (networkx, scipy) and less
mature Rust equivalents. Phase 1 should try petgraph first. If the partitioning quality is insufficient, shell out to
Python. The analysis is a batch job — startup overhead doesn't matter.

### What's Deterministic

The Analyze phase, the Serve routing, and the maintenance drift detection are all deterministic. Graph algorithms, glob
matching, interface diffing. Fast, reliable, no LLM cost. These run on every commit without hesitation.

### What Requires Judgment

Spec generation, domain synthesis, constitution distillation, and boundary-relevance classification for ambiguous
changes. These require an LLM. They need flexibility, interpretation, the ability to infer unstated design intent. The
LLM does the creative work within the scaffold that the entanglement analysis provides.

# Aiome: A Renormalization Engine for Codebases

## The Problem

AI coding agents are boundedly rational. Finite context windows, no memory across sessions, no understanding of your
project's architecture or history. The current solution is hand-written context files — CLAUDE.md, .cursorrules,
AGENTS.md — that encode project knowledge in a format agents can consume at session start.

This works at small scale. A 1,000-line project fits in a single file. A 100,000-line project does not.
[Vasilopoulos (2026)](https://arxiv.org/abs/2602.20478) documents what happens next: you end up building a multi-tier
knowledge hierarchy by hand — a constitution loaded every session, 19 specialized agent specs, 34 subsystem documents —
totaling 26,000 lines maintained at 1-2 hours per week, with stale specs as the primary failure mode.

That hierarchy is correct. The manual maintenance is not sustainable.

Aiome automates the hierarchy. It analyzes your codebase's structure, identifies where the natural boundaries are,
generates knowledge at multiple resolutions, maintains it as the code evolves, and serves the right resolution to the
right agent for the right task.

## Why This Design Works

The knowledge hierarchy isn't an arbitrary organizational choice. It's a consequence of how codebases are structured and
how bounded agents consume information. There is a finite optimal context size for any agent operating in an environment
with local interactions — adding more information beyond this bound actively degrades performance. For codebases, this
means the hierarchy is *necessary*, not merely convenient.

**Codebases have entanglement structure.** Some files are tightly coupled — they share types, change together, fail
together. Others are nearly independent. This structure determines what an agent needs to know: if you're modifying a
file in the networking subsystem, you need to understand the determinism invariants that cross into the RNG and
persistence modules. You don't need to know how the UI layout works. The entanglement map tells you which knowledge is
relevant to which task.

**Agents need knowledge at the right resolution.** A simple task in an isolated module needs a thin context: project
conventions and the module's interface. A complex task in a highly entangled domain needs a thick context: the full
theory of the domain, the cross-module interactions, the known failure modes. Loading too little context causes
mistakes. Loading too much wastes the context window on irrelevant information and can actively degrade the agent's
performance — beyond an optimal context size, additional inputs act like noise that dilutes useful signal. The right
amount depends on the task's entanglement with the codebase.

**Knowledge at different scales preserves different things.** This is the renormalization group applied to code. At each
scale, "irrelevant operators" are integrated out and only "relevant operators" survive:

| Scale transition | What's preserved (relevant)                     | What's integrated out (irrelevant) |
|------------------|-------------------------------------------------|------------------------------------|
| 0 → 1            | Function signatures, types, intent per file     | Implementation details, formatting |
| 1 → 2            | Subsystem interfaces, invariants, failure modes | Individual file internals          |
| 2 → 3            | Cross-subsystem interactions, domain theory     | Single-subsystem details           |
| 3 → 4            | Universal conventions, routing rules            | Domain-specific knowledge          |

Each level compresses aggressively, keeping only what matters for decisions at that scale. This is the same principle
that makes scientific models work: you don't need quantum mechanics to design a bridge, but you do need it to design a
transistor. The resolution must match the problem.

**The bond dimension should be adaptive.** The entanglement between a task and the codebase is not constant — a simple
timeout fix in the networking module has low entanglement, while debugging a distributed state desync has high
entanglement, even though both touch the same partition. The optimal amount of context (the "bond dimension" at the
boundary between agent and codebase) should vary with the task, not just with the partition. Aiome's serve phase adapts
context not just by routing to partitions, but by trimming within specs based on task-level relevance.

**The hierarchy maintains itself through boundary detection.** Not every code change requires a knowledge update. An
internal refactor that preserves all interfaces doesn't change what external agents need to know — the boundary is
intact. A one-line type change that breaks a cross-module invariant does — the boundary is breached. By tracking
*boundary-relevant* changes rather than *any* changes, the maintenance cost drops from "update specs whenever code
changes" to "update specs when interfaces or invariants change." Most commits don't cross boundaries.

**The coarse-graining improves through feedback.** The initial knowledge hierarchy is generated using generic heuristics
about what agents need. Over time, agent session logs reveal what *actually* matters: which spec sections correlate with
task success, which are loaded but never used, which gaps cause repeated failures. This feedback loop is the variational
optimization of the RG flow — tuning the coarse-graining to maximize downstream task performance, not just linguistic
coherence.

## Architecture

Five phases, each independently valuable:

```
ANALYZE ──→ GENERATE ──→ SERVE ──→ LEARN ──→ ORCHESTRATE
  │              │             │          │          │
  │  no LLM      │  LLM        │  LLM opt │  LLM opt │  LLM
  │  seconds     │  minutes    │  ms      │  async   │  per task
  │  every run   │  on init    │  every   │  every   │  on
  │              │  + drift    │  session │  session │  demand
  ▼              ▼             ▼          ▼          ▼
Entanglement   Knowledge     Adaptive   Spec quality  Task
map +          hierarchy     context    scores +      routing +
partitions     (5 scales)    packages   feedback      agents
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
┌────────────────────────────────────────────────────────────┐
│              Entanglement Map (example)                    │
│                                                            │
│   ┌─────────┐         ┌─────────┐                          │
│   │   UI    │─ ─ ─ ─ ─│  Auth   │     weak coupling        │
│   │  (0.31) │         │  (0.44) │                          │
│   └────┬────┘         └─────────┘                          │
│        │                                                   │
│   ┌────┴────┐═════════╗                                    │
│   │  Game   │═════════║═══╗                                │
│   │  State  │   ║     ║   ║    strong coupling             │
│   │  (0.78) │   ║  ┌──╨───╨───┐                            │
│   └─────────┘   ║  │Networking│                            │
│                 ║  │  (0.91)  │                            │
│            ┌────╨──┴──┐       │                            │
│            │   RNG    │───────┘                            │
│            │  (0.85)  │                                    │
│            └──────────┘                                    │
│                                                            │
│  Scores = entanglement with rest of codebase               │
│  ═══ = high mutual information (need Scale 3 domain specs) │
│  ─ ─ = low mutual information (Scale 2 specs suffice)      │
└────────────────────────────────────────────────────────────┘
```

**Implementation:** tree-sitter for AST parsing, git2 for history analysis, petgraph for graph algorithms. All Rust, all
deterministic, runs in seconds on codebases up to ~500K lines. Phase 1 alone is useful as a codebase structure
visualizer even without the rest of the system.

### Generate

**Input:** Partition structure + entanglement scores.  
**Output:** Knowledge hierarchy (Scales 1–4; Scale 0 is the raw source code itself).

This phase requires an LLM. It's the expensive step, run once on init and incrementally on drift. The hierarchy follows
the renormalization group convention: Scale 0 is the finest resolution (raw code), and each higher scale integrates out
more detail, preserving only what's relevant for decisions at that scale.

```
Scale 0:  Raw source code            (every token of every file)
Scale 1:  File-level summaries       (what each file does, its interfaces)
Scale 2:  Subsystem specs            (how components interact, invariants)
Scale 3:  Domain specs               (synthesized knowledge for high-entanglement domains)
Scale 4:  Project constitution       (universal conventions, routing table)
```

Generation proceeds bottom-up: Scale 1 first (each file independently, fully parallelizable), then Scale 2 (reads
constituent files + their Scale 1 summaries), then Scale 3 (reads relevant Scale 2 specs), then Scale 4 (reads
everything). On first run, human review of generated specs is strongly recommended — the first pass has no feedback
signal on what agents actually need.

**Scale 1 — File-level summaries.** One per source file. For each file, the LLM generates a brief summary (one
paragraph, typically 3-8 lines) capturing:

- **Purpose:** What does this file do in one sentence?
- **Key interfaces:** The public functions, types, or traits it exposes.
- **Internal dependencies:** Which other files within the same partition it couples to, and how.
- **Gotchas:** Anything non-obvious — implicit ordering requirements, unsafe blocks, performance-sensitive paths.

File summaries are the cheapest tier to generate (short context per call, highly parallelizable) and the cheapest to
maintain (regenerate only when the file itself changes). They fill a critical gap: when an agent is working *inside* a
partition, the Scale 2 subsystem spec tells it about the boundary, but says nothing about the internal structure. A
5,000-line partition with 20 files has its own entanglement structure — some files are tightly coupled internally (the
state machine and the event handler), others are relatively independent (the serialization layer and the utilities).
File summaries give the serve phase an intermediate resolution to draw from, so agents working on the state machine can
load summaries for coupled internal files without loading the entire partition.

**Scale 2 — Subsystem specs.** One per partition. For each partition, the LLM reads the constituent files (guided by
their Scale 1 summaries) and generates a spec that captures:

- **Interfaces:** What does this subsystem expose? Function signatures, types, API contracts.
- **Invariants:** What must always be true? Extracted from assertions, test expectations, comments, and inferred from
  patterns.
- **Failure modes:** What has gone wrong? From reverted commits, bug-fix patterns in git history, known edge cases.
- **Design intent:** Why is it built this way? From commit messages, PR descriptions, architectural decision records.
- **Couplings:** Which other subsystems does it depend on, and at what interface?

The generation prompt embodies a specific principle: *extract the minimal representation that maximizes an agent's
ability to make correct decisions when working on adjacent code.* This is not "summarize the code." It's "what would a
competent new team member need to know to safely modify this subsystem without breaking anything downstream?"

**Scale 3 — Domain specs.** One per high-entanglement cluster. When a group of partitions are strongly coupled, their
individual Scale 2 specs aren't rich enough — an agent working near their boundary needs the *theory* of how they
interact. A domain spec synthesizes across multiple subsystem specs:

- The full domain theory (e.g., "all combat state must be deterministic; here's why and how")
- Cross-subsystem interaction patterns
- Accumulated symptom → cause → fix tables
- Pre-synthesized views that would otherwise require loading 4-5 subsystem specs

The threshold for promotion from Scale 2 to Scale 3 is the entanglement score. High entanglement means the partition
cluster's boundary carries more information — agents working near it need richer context. The threshold is a tunable
parameter (default ~0.7), but the principle is fixed: richer boundary for more entangled clusters.

**Scale 4 — Constitution.** One per project. The most aggressive compression. From all specs and domain definitions,
extract only what's universal:

- Conventions that apply everywhere (naming, style, error handling patterns)
- The routing table: file patterns → which specs to load
- Cross-cutting invariants that span all subsystems
- Build, test, deploy commands

**Hard constraint:** The constitution must fit in ~1000 lines. It's loaded into every agent session, so every line must
earn its place. This budget forces real prioritization — the same way a limited context window forces agents to
satisfice, a limited constitution forces the system to compress.

### Serve

**Input:** Task description (natural language, file paths, or both).  
**Output:** Adaptive context package at the right resolution, trimmed to task-level relevance.

The serve phase is more than a routing algorithm. Given a task, it determines which partitions are involved, computes
the task's entanglement with the codebase, loads context proportional to that entanglement, and then *trims within
specs* based on section-level relevance to the specific task. This adaptive bond dimension is the key difference from a
naive tier-loading approach.

**Partition routing.** When file paths are provided, partition mapping is direct (glob patterns in the routing table).
When only a natural language description is given, keyword matching against partition names and spec content provides
the mapping. An LLM routing call is available as a fallback for ambiguous descriptions but is not the default path —
it's too expensive for a fast operation.

**Task-level entanglement scoring.** After identifying the touched partitions, the serve phase computes a *task-level*
entanglement score — not just the partition's static score, but the specific task's coupling to each section of each
relevant spec. A simple timeout fix in the networking module has low task-level entanglement (it only needs the socket
interface section). Debugging a distributed state desync has high task-level entanglement (it needs the full domain
theory, the determinism invariants, the cross-subsystem interaction patterns).

The task-level scorer uses lightweight signals: keyword and embedding overlap between the task description and each
section header/content of each spec. No LLM call required — the expensive LLM-generated spec is the knowledge; the
cheap deterministic scorer selects which slices cross the boundary into the agent's context.

**Adaptive context assembly.** The context package draws from all five scales, loading only what the task demands:

```
Task: "fix the desync bug in combat damage"
  ↓
Touched partitions:  combat, networking, rng
Task entanglement:   high (0.88)
  ↓
Context package:
  ✓ Scale 4: constitution.md                            (847 lines)
  ✓ Scale 3: networking-domain.md                       (915 lines)
  ✓ Scale 2: combat-system.md [trimmed: invariants,
              failure modes, couplings sections only]   (283 lines)
  ✓ Scale 2: deterministic-rng.md                       (283 lines)
  ✓ Scale 1: networking/serializer.rs summary           (6 lines)
  ✓ Scale 1: combat/damage_calc.rs summary              (5 lines)
  Total: ~2,340 lines
```

vs.

```
Task: "add a tooltip to the settings page"
  ↓
Touched partitions:  ui
Task entanglement:   low (0.31)
  ↓
Context package:
  ✓ Scale 4: constitution.md                            (847 lines)
  ✓ Scale 2: ui-components.md [trimmed: interfaces
              section only]                             (134 lines)
  Total: ~980 lines
```

vs. a *within-partition* task that uses the new Scale 1:

```
Task: "refactor the event handler in the game state module"
  ↓
Touched partitions:  game-state (internal task)
Task entanglement:   medium (0.52)
  ↓
Context package:
  ✓ Scale 4: constitution.md                            (847 lines)
  ✓ Scale 2: game-state.md [trimmed: interfaces,        (195 lines)
              invariants sections]
  ✓ Scale 1: game-state/event_handler.rs summary        (7 lines)
  ✓ Scale 1: game-state/state_machine.rs summary        (6 lines)
  ✓ Scale 1: game-state/transitions.rs summary          (5 lines)
  (file summaries selected by intra-partition coupling)
  Total: ~1,060 lines
```

The same system, different resolutions and different *sections* for different tasks. The agent's context window is spent
on what matters — not just at the partition level, but at the section level within each spec.

**Task-conditioned partition overlay.** For tasks that span partition boundaries, the serve phase can dynamically merge
or split the static partition structure. If a task description spans two partitions that are adjacent in the
entanglement graph, serve loads a combined view. If a task is entirely internal to a large partition, serve uses the
Scale 1 file summaries and intra-partition entanglement to load only the relevant sub-cluster. The static partitions
from the Analyze phase are the default, but the serve phase adapts them per-task using a lightweight re-weighting of the
entanglement map edges based on the task's semantic overlap. This runs in milliseconds — it doesn't recompute the full
partition structure, just adjusts which boundaries are "active" for this task.

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

### Learn

**Input:** Agent session logs (tool calls, files read, edits made, success/failure outcomes).  
**Output:** Spec quality scores, gap detection, bond dimension tuning.

The learn phase closes the loop from "generate specs based on what *should* matter" to "generate specs based on what
*actually* matters." This is the variational optimization of the RG flow — tuning the coarse-graining to maximize
downstream task performance.

**Spec section scoring.** After every agent session, log which spec sections were loaded (from context package traces),
which edits the agent made, and whether the task succeeded or failed. Over time, this builds a dataset of (spec section,
task type, outcome) tuples. Sections that are frequently loaded but never correlated with success are candidates for
compression or removal — they're wasting bond dimension on irrelevant operators. Sections that are rarely loaded but
whose absence correlates with failures are candidates for promotion — they should be more prominent, or included at
higher scales.

**Gap detection.** When agents repeatedly fail on tasks in a partition *despite loading the full spec*, the spec is
missing a relevant operator. The learn phase flags these partitions for targeted regeneration: "agents consistently fail
at X when working in this partition — what knowledge is missing?" This can trigger an LLM call to regenerate the spec
with a focused prompt, or flag for human review.

**Empirical bond dimension tuning.** Track the relationship between context size and task success rate per partition. If
agents succeed equally well with 200 lines of context vs. 900 lines for a given partition, the spec is over-specified —
the serve phase should trim more aggressively. If success drops sharply below 500 lines, that's the empirical $C^*$ for
that partition. Over time, the learn phase builds a per-partition profile of optimal context size, which the serve phase
uses to calibrate its task-level trimming.

**Partition quality feedback.** If agents consistently need to load specs from two partitions together, the entanglement
weight between those partitions should increase — they may need a shared Scale 3 domain spec, or even a partition merge.
If agents working in a large partition consistently only need a small subset of its files, the partition may be too
coarse and should be split. The learn phase adjusts entanglement weights in the map (soft, continuous) and flags
structural re-partitioning when the adjustments accumulate beyond a threshold (hard, infrequent).

**Inertia.** The learn phase updates spec scores and entanglement weights with exponential moving averages, not raw
session-by-session changes. This prevents runaway drift from a few unusual sessions. Structural changes (partition
merges, splits, new Scale 3 domains) require consistent signal over many sessions before triggering. The right inertia
is a tunable parameter, but the principle is: fast feedback on spec content quality, slow feedback on structural
changes.

```
Session log
    │
    ├── Which spec sections were loaded?
    │     → Update section relevance scores
    │
    ├── Did the task succeed or fail?
    │     → Correlate with loaded context
    │     → Update per-partition C* estimates
    │
    ├── Which files were actually read/edited?
    │     → Compare to predicted partition routing
    │     → Detect routing errors
    │
    └── Were specs from multiple partitions loaded together?
          → Update cross-partition entanglement weights
          → Flag potential domain spec promotion
```

### Orchestrate

**Input:** Task description + available agents.
**Output:** Routed task execution with auto-generated context.

The orchestrate phase is the capstone: given a task, it selects the right agent(s), generates the appropriate context
package via the serve phase, and dispatches execution. This is the original Aiome vision — a microbiome of AI agents
coordinating through shared knowledge infrastructure — but built on the principled foundation of the previous four
phases rather than hand-written routing tables.

**Task routing.** Given a task description, orchestrate determines which agent is best suited (based on agent
capabilities, task type, and historical success rates from the learn phase), generates an adaptive context package, and
dispatches the task. For tasks that span multiple partitions or require multiple perspectives, orchestrate can fan out
to multiple agents in parallel, each receiving context tailored to their portion of the work.

**Agent coordination.** When multiple agents work on related tasks, orchestrate ensures they receive consistent context
— the same partition specs, the same invariants — so their outputs are compatible. The entanglement map drives this:
agents working on entangled partitions receive overlapping context at the boundary, preventing conflicting changes.

## CLI

```bash
aiome init                                # analyze + generate full hierarchy
aiome status                              # partitions, entanglement scores, stale specs
aiome update                              # incremental regeneration for flagged specs
aiome context "refactor the auth module"  # compute adaptive context package
aiome context --files src/net/*.rs        # context from file paths
aiome inspect --partition networking      # examine a specific partition's knowledge
aiome inspect --entanglement              # coupling heatmap
aiome inspect --quality                   # spec quality scores from learn phase
aiome export --format claude              # generate CLAUDE.md
aiome export --format mcp                 # generate MCP server config
aiome watch                               # continuous: monitor commits, flag drift
aiome learn --session <log>               # ingest agent session log, update quality scores
aiome learn --report                      # show spec quality report, gap detections
aiome run "implement the auth module"     # orchestrate: auto-route, auto-context, run agent
aiome run @claude "review the PR"         # orchestrate: specific agent
aiome run @all "design the API"           # orchestrate: all agents in parallel
```

**Phase 1** ships `init` (analyze only — no LLM, just the entanglement map and partition structure) + `status` +
`inspect`. This is already useful: a visual map of your codebase's coupling structure, the natural subsystem boundaries,
which modules are dangerously entangled. No LLM cost. Pure graph analysis.

**Phase 2** adds generation: `init` now produces the full hierarchy (Scales 1-4), `update` handles incremental
regeneration, `export` generates tool-specific context files.

**Phase 3** adds serving: `context` computes adaptive packages with task-level trimming, `watch` maintains integrity.

**Phase 4** adds learning: `learn` ingests agent session logs, computes spec quality scores, detects gaps, tunes bond
dimensions. This closes the feedback loop.

**Phase 5** adds orchestration: `run` routes tasks to agents with auto-generated context. This is the original Aiome
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

**Not static.** The hierarchy is alive. Regenerated when structure shifts, updated when boundaries are breached, tuned
by the learn phase based on what agents actually need, serving different resolutions to different tasks. It's an engine,
not a snapshot.

## Open Questions

**Partition stability.** Community detection algorithms can produce different partitions on small input changes. Should
we pin partitions and only re-detect explicitly? How do we handle files that sit on partition boundaries with high
coupling to multiple partitions? The current design treats re-partitioning as an explicit, infrequent operation, but the
learn phase's partition quality feedback may eventually provide signal for when re-partitioning is warranted.

**Cold start cost.** On `aiome init` for a 100K-line project with 12 partitions, the Generate phase now makes many more
LLM calls (file-level summaries for every source file, plus 12+ partition-level specs). Scale 1 file summaries are
individually cheap but numerous. Can we generate them lazily — only for files in partitions that agents actually touch?
Should there be a "constitution-only" quick start mode that generates Scale 4 immediately and backfills Scales 1-3 on
demand?

**Spec quality signal cold start.** The learn phase needs agent session logs to compute quality scores, but at
initialization there are no logs. The initial specs are generated with generic heuristics. How many sessions are needed
before the learn phase provides reliable signal? Can we bootstrap with synthetic sessions — asking the LLM "given this
spec, what would you do if asked to modify file X?" and scoring the answer against known-good approaches?

**Routing ambiguity.** When a task is described only in natural language, mapping to partitions requires interpretation.
How reliable is keyword matching? When is an LLM routing call justified? What's the failure mode when routing is wrong —
and can the agent detect that it has insufficient context? The learn phase can detect routing errors after the fact (the
agent read files outside the predicted partitions), but real-time correction during a session is harder.

**Cross-language boundaries.** Tree-sitter handles many languages, but cross-language coupling (Rust backend ↔
TypeScript frontend via API) isn't captured by import analysis. API contracts, shared type definitions, and OpenAPI
specs could serve as cross-language entanglement signals. Are cross-language boundaries always partition boundaries?

**Task-level trimming calibration.** The serve phase trims within specs based on section-level relevance scores. How
aggressive should the trimming be? Too aggressive and the agent misses relevant context that didn't keyword-match the
task description. Too conservative and you're back to loading full specs. The learn phase can tune this threshold
empirically, but the initial default matters for the cold-start experience.

**Scale 1 quality vs. cost tradeoff.** File-level summaries are cheap per file but add up across a large codebase.
For a 100K-line project with 500 source files, that's 500 LLM calls at init time. Are all files worth summarizing? Can
we skip files below a complexity threshold (e.g., pure re-exports, simple constants files) and only summarize files that
tree-sitter identifies as having non-trivial structure?

**The learn phase feedback loop.** The learn phase adjusts entanglement weights and spec quality scores based on agent
outcomes. But the agents are consuming context that Aiome provides — creating a feedback loop between the knowledge
hierarchy and the agents consuming it. How do we prevent runaway drift? The exponential moving average provides inertia,
but the right decay rate is unknown. Too fast and the system chases noise. Too slow and it can't adapt to genuine
structural changes.

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
| sled / sqlite   | Session logs, quality scores          | Learn              |

The Python question: Louvain and spectral clustering have mature Python implementations (networkx, scipy) and less
mature Rust equivalents. Phase 1 should try petgraph first. If the partitioning quality is insufficient, shell out to
Python. The analysis is a batch job — startup overhead doesn't matter.

### What's Deterministic

The Analyze phase, the Serve routing and trimming, the maintenance drift detection, and the Learn phase's score
aggregation are all deterministic. Graph algorithms, glob matching, interface diffing, section-level relevance scoring,
exponential moving averages over session outcomes. Fast, reliable, no LLM cost. These run on every commit or session
without hesitation.

### What Requires Judgment

Spec generation, domain synthesis, constitution distillation, boundary-relevance classification for ambiguous changes,
and targeted spec regeneration triggered by the Learn phase's gap detection. These require an LLM. They need
flexibility, interpretation, the ability to infer unstated design intent. The LLM does the creative work within the
scaffold that the entanglement analysis provides. The Learn phase tells the LLM *where* to focus; the LLM decides *what*
to generate.

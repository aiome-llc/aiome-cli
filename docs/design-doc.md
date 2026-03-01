# Aiome: Engineering Design Document

## Problem Statement

AI coding agents are boundedly rational. They have finite context windows, no persistent memory across sessions, and no
innate understanding of your project's architecture, conventions, or history. The current solutions — hand-written
CLAUDE.md files, .cursorrules, AGENTS.md manifests — work for small projects but collapse at scale. A 100,000-line
codebase cannot be described in a single file, and manually maintaining a hierarchy of documentation is a full-time job.

Aiome automates this. It analyzes a codebase's coupling structure, identifies natural subsystem boundaries, generates
knowledge at multiple scales of resolution, maintains that knowledge as the codebase evolves, and serves the right
resolution to the right agent for the right task.

---

## System Overview

Aiome operates in three phases:

```
┌─────────────────────────────────────────────────────────────┐
│                        ANALYZE                              │
│                                                             │
│  Source code ──→ Dependency graph ──→ Co-change graph       │
│  Test coverage ──→ Failure history ──→ Coupling map         │
│                                                             │
│  Output: partition structure + coupling scores              │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────┐
│                       GENERATE                               │
│                                                              │
│  For each partition:                                         │
│    Extract interfaces and invariants at target scale         │
│    Generate spec via LLM                                     │
│    Validate: does the spec capture what agents need?         │
│                                                              │
│  Distill constitution from all specs                         │
│                                                              │
│  Output: complete knowledge hierarchy                        │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────┐
│                        SERVE                                 │
│                                                              │
│  Task description ──→ Coupling analysis ──→ Context load     │
│                                                              │
│  Low coupling:    Constitution + relevant spec               │
│  High coupling:   Constitution + domain spec + coupled specs │
│  Very high:       Pull everything relevant into context      │
│                                                              │
│  Output: optimally-sized context for any agent session       │
└──────────────────────────────────────────────────────────────┘
```

**Analyze** is deterministic: graph algorithms on dependency and git data. **Generate** requires LLM judgment: it
synthesizes knowledge from source code. **Serve** is a routing algorithm: it maps tasks to the right slice of the
knowledge hierarchy.

---

## Data Model

### Coupling Graph

A weighted undirected graph where nodes are files or modules and edge weights represent coupling strength.

```rust
struct CouplingGraph {
    /// Nodes: file paths or module identifiers
    nodes: Vec<NodeId>,
    /// Edges: weighted coupling between nodes
    edges: Vec<(NodeId, NodeId, CouplingWeight)>,
}

struct CouplingWeight {
    /// Import/type-sharing weight (from static analysis)
    static_coupling: f64,
    /// Co-change frequency weight (from git history)
    temporal_coupling: f64,
    /// Test co-failure weight (from CI, when available)
    causal_coupling: Option<f64>,
    /// Combined score
    combined: f64,
}
```

Built from four signal sources, in order of availability:

| Signal            | Source             | Always available? |
|-------------------|--------------------|-------------------|
| Static coupling   | AST / imports      | Yes               |
| Temporal coupling | Git history        | Yes               |
| Causal coupling   | Test/CI data       | No                |
| Experiential      | Agent session logs | No                |

The system works with just static + temporal. Additional signals improve partition quality.

### Partition

A grouping of nodes from the coupling graph into subsystems, produced by community detection.

```rust
struct Partition {
    id: PartitionId,
    name: String,
    /// Files belonging to this partition
    files: Vec<PathBuf>,
    /// Coupling score with the rest of the codebase (0.0 - 1.0)
    coupling_score: f64,
    /// Whether this partition is promoted to a domain (high coupling)
    is_domain: bool,
}
```

### Knowledge Hierarchy

The multi-scale knowledge structure. Four tiers, from most detailed to most abstract:

```
Tier 4: Raw source code            (every token of every file — not stored, just referenced)
Tier 3: Subsystem specs            (one per partition — interfaces, invariants, failure modes)
Tier 2: Domain specs               (one per high-coupling domain — cross-subsystem synthesis)
Tier 1: Constitution               (one per project — universal conventions, routing table)
```

Note: the theoretical design describes a fifth level — file-level summaries sitting between raw source and subsystem
specs. In practice, file-level summaries are an intermediate artifact produced *during* Tier 3 generation (the LLM
reads individual files to produce the subsystem spec) rather than a tier stored and served independently. If future
usage shows that agents frequently need single-file summaries, this could be promoted to a standalone tier.

Each tier captures what matters at that scale and omits what doesn't:

| Tier transition | What's preserved                            | What's omitted                     |
|-----------------|---------------------------------------------|------------------------------------|
| 4 → 3           | Function signatures, types, intent          | Implementation details, formatting |
| 3 → 2           | Cross-subsystem interactions, domain theory | Single-subsystem internals         |
| 2 → 1           | Universal conventions, routing rules        | Domain-specific knowledge          |

```rust
struct KnowledgeHierarchy {
    constitution: Constitution,       // Tier 1
    domains: Vec<DomainSpec>,         // Tier 2
    subsystems: Vec<SubsystemSpec>,   // Tier 3
    partitions: Vec<Partition>,       // The underlying structure
    coupling_graph: CouplingGraph,    // The raw coupling data
}

struct Constitution {
    /// Universal conventions (naming, style, patterns)
    conventions: String,
    /// Routing table: partition → spec mapping
    routing_table: Vec<RoutingEntry>,
    /// Cross-cutting invariants
    invariants: String,
    /// Build/test/deploy commands
    commands: String,
    /// Must fit within ~1000 lines — loaded into every session
    raw_markdown: String,
}

struct SubsystemSpec {
    partition_id: PartitionId,
    /// What this subsystem exposes to others
    interfaces: String,
    /// What must always be true (from assertions, tests, comments)
    invariants: String,
    /// What has gone wrong (from git blame, reverts, issues)
    failure_modes: String,
    /// Why it's built this way (from commits, PRs, ADRs)
    design_intent: String,
    /// Dependencies on other subsystems and at what interface
    known_couplings: Vec<(PartitionId, String)>,
    raw_markdown: String,
}

struct DomainSpec {
    partition_ids: Vec<PartitionId>,
    /// Full "theory" of the domain (e.g., determinism requirements for networked state)
    domain_model: String,
    /// Cross-subsystem interaction patterns
    interaction_patterns: String,
    /// Accumulated symptom → cause → fix tables
    troubleshooting: String,
    /// Pre-synthesized views that replace loading 4-5 subsystem specs
    synthesized_views: String,
    raw_markdown: String,
}

struct RoutingEntry {
    /// Glob patterns for files in this partition
    file_patterns: Vec<String>,
    /// Which specs to load when touching these files
    specs: Vec<SpecRef>,
}
```

---

## Phase 1: Analyze

**Input:** A codebase (source files + git history).
**Output:** A coupling graph and a partition structure.

### Step 1: Build the Coupling Graph

#### Static Coupling (from the code)

Parse every source file with tree-sitter to extract:

- **Import edges:** which files reference which types, functions, modules
- **Interface surface area:** how many symbols cross each module boundary
- **Type sharing:** which types appear in multiple modules (strong coupling signal)

Tree-sitter provides language-agnostic AST parsing. For a Rust project, this means extracting `use` statements, `pub`
items, trait implementations, and type references across module boundaries.

#### Temporal Coupling (from git)

Process git history (via `git2`) to extract:

- **Co-change frequency:** files that change together in the same commit are coupled
- **Change propagation:** when file A changes, how often does file B change within N subsequent commits?
- **Churn correlation:** modules with correlated churn rates may share hidden dependencies

#### Causal Coupling (from tests, when available)

If test infrastructure is present:

- **Test dependency:** which tests exercise which source files
- **Failure correlation:** which tests fail together (shared failure modes)
- **Coverage gaps:** untested boundaries represent unknown coupling

#### Experiential Coupling (from agent logs, when available)

If agent session logs exist:

- **Session traces:** which files did agents read together to complete tasks?
- **Error patterns:** which domains caused repeated re-explanation?
- **Spec access patterns:** which specs are accessed together?

#### Combining Signals

Each signal type produces edge weights. Combine with a weighted sum:

```
combined(a, b) = w_s * static(a, b) + w_t * temporal(a, b) + w_c * causal(a, b) + w_e * experiential(a, b)
```

Default weights: `w_s = 0.4, w_t = 0.4, w_c = 0.15, w_e = 0.05`. Missing signals get weight redistributed to
available signals.

### Step 2: Partition the Graph

Run community detection on the coupling graph to identify natural subsystem boundaries.

**Algorithm options:**

- **Louvain algorithm:** Fast, greedy modularity optimization. Good default for most codebases.
- **Spectral clustering:** More principled, better for irregular coupling structures. Requires eigendecomposition of the
  graph Laplacian.

Output: a set of partitions with modularity score Q (higher = cleaner boundaries). Typical healthy codebases: Q > 0.5.

### Step 3: Score Coupling

For each partition, compute a coupling score: how much mutual information does this partition share with the rest of the
codebase?

```
coupling_score(P) = sum of edge weights crossing partition P's boundary
                    / total edge weight in the graph
```

Partitions with coupling scores above a threshold (e.g., > 0.7) are candidates for domain promotion in the Generate
phase.

### Implementation Notes

Static and temporal analysis are deterministic and cheap. They run in seconds on codebases up to ~500K lines. Causal
and experiential analysis are optional enhancements.

**Dependencies:** `tree-sitter` (AST parsing), `git2` (git history), `petgraph` (graph algorithms).

---

## Phase 2: Generate

**Input:** Partition structure + coupling scores from Analyze.
**Output:** Complete knowledge hierarchy (Tier 1 + Tier 2 + Tier 3 specs).

This phase requires an LLM. It's the expensive one-time cost.

### Tier 3: Subsystem Specs

For each partition, the LLM reads the constituent files and generates a spec capturing:

- **Interfaces:** what does this subsystem expose to others?
- **Invariants:** what must always be true? (extracted from assertions, tests, comments)
- **Failure modes:** what has gone wrong? (from git blame, reverted commits, issue trackers)
- **Design intent:** why is it built this way? (from commit messages, PR descriptions, ADRs)
- **Known couplings:** which other subsystems does this one depend on, and at what interface?

**Prompt strategy:** The generation prompt is guided by the information bottleneck principle — extract the minimal
representation that maximizes an agent's ability to make correct decisions when working on adjacent code. This is not
"summarize the code." It's "what would a new team member need to know to safely modify this subsystem?"

The prompt includes:

1. All source files in the partition
2. The partition's coupling data (which other partitions it touches, at what weight)
3. Relevant git history (recent commits, reverts, notable changes)
4. A structured output template (interfaces, invariants, failure modes, design intent, couplings)

### Tier 2: Domain Specs

Partitions with high coupling scores — high mutual information with many other partitions — get promoted to domains.
These are the networking layers, the persistence systems, the authentication modules.

The domain spec synthesizes across multiple subsystem specs into a coherent domain model:

- The full "theory" of the domain (e.g., determinism requirements for networked state)
- Cross-subsystem interaction patterns
- Accumulated symptom → cause → fix tables
- Pre-synthesized views that would otherwise require loading 4-5 specs

**Promotion threshold:** If a partition's coupling score exceeds a threshold (default 0.7), it gets a domain spec. This
means it needs a richer representation than a simple subsystem spec provides, because agents working near it need more
context.

### Tier 1: Constitution

The final distillation step. From all specs and domain definitions, extract:

- Universal conventions (naming, style, patterns that apply everywhere)
- The routing table (partition structure expressed as "if touching files in X, load spec Y")
- Cross-cutting invariants (things that must be true across all subsystems)
- Build/test/deploy commands

**Context budget constraint:** The constitution must fit within ~1000 lines. It's loaded into every agent session, so it
must be minimal. This is the hard constraint that forces real prioritization.

### The Bootstrapping Problem

On first run (`aiome init`), there are no existing specs to build on. The LLM must generate everything from scratch.

**Mitigations:**

- The Analyze phase provides structure even without specs — the partition boundaries and coupling scores guide the LLM
- Generation proceeds bottom-up: Tier 3 first, then Tier 2 (which reads Tier 3 specs), then Tier 1 (which reads
  everything)
- Each spec is generated independently within its partition, so generation can be parallelized
- Human review of generated specs before they're used is strongly recommended on first run

---

## Phase 3: Serve

**Input:** A task description (natural language and/or file paths).
**Output:** An optimally-sized context package.

### The Routing Algorithm

**Step 1: Identify touched partitions.**

From the task description and target files, determine which partitions are involved. If file paths are provided, map
them directly. If only a natural language description is given, use keyword matching against partition names and spec
content.

**Step 2: Compute task coupling.**

Using the coupling graph, determine how coupled the touched partitions are to the rest of the codebase.

```
task_coupling = max(coupling_score(P) for P in touched_partitions)
              + cross_partition_coupling(touched_partitions)
```

A task touching only files within a single low-coupling partition has low task coupling. A task touching files across
multiple high-coupling partitions has high task coupling.

**Step 3: Select context.**

Load context proportional to coupling:

| Coupling level | Context loaded                                               |
|----------------|--------------------------------------------------------------|
| Minimal        | Constitution only                                            |
| Low            | Constitution + relevant Tier 3 spec                          |
| Medium         | Constitution + Tier 3 specs for touched + coupled partitions |
| High           | Constitution + Tier 2 domain spec + all coupled specs        |
| Very high      | Everything relevant pulled into main context                 |

### Output Formats

The same knowledge hierarchy, multiple delivery formats:

- **CLAUDE.md / AGENTS.md:** Constitution formatted for Claude Code / Codex / Copilot
- **MCP server:** Specs served via Model Context Protocol for on-demand retrieval
- **.cursorrules:** Constitution formatted for Cursor
- **Raw markdown:** For any tool that reads project docs

---

## Maintenance Loop

The most valuable ongoing function. On every commit:

```
1. Identify changed files
2. Map to partitions
3. For each affected partition:
   a. Did the change cross a partition boundary?
      - Modified an interface type?
      - Changed a function signature used by other partitions?
      - Added/removed a dependency?
   b. If YES: flag spec for regeneration (module boundary may be broken)
   c. If NO: check if change contradicts existing spec
      - If contradicts: flag for update
      - If consistent: no action (internal details changed, spec still valid)
4. Periodically: re-run coupling analysis
   - Has coupling structure shifted?
   - Have new partitions emerged?
   - Have old ones merged?
   - If so: re-partition and regenerate
```

**Key distinction from syntactic drift detection:** A naive system checks whether files changed without spec updates.
Aiome checks whether *boundary-relevant information* changed. An internal refactor that preserves all interfaces needs
no spec update. A one-line type change that breaks a cross-module invariant does.

**Triggers for spec regeneration:**

- Modified `pub` item signatures (Rust), exported functions/types (TS/JS), public methods (Python/Java)
- Added or removed cross-partition imports
- Changes to test files that exercise cross-partition boundaries
- Manual `aiome update` invocation

**Triggers for full re-partitioning:**

- Significant new files added (new module)
- Large-scale refactors (many files moved between directories)
- Coupling structure drift (detected by periodic re-analysis, e.g., weekly)
- Manual `aiome init --reanalyze` invocation

---

## CLI Design

```bash
# Initialize: analyze codebase, generate full knowledge hierarchy
aiome init

# Status: show partition structure, coupling scores, stale specs
aiome status

# Update: incremental regeneration after changes
aiome update

# Serve: compute optimal context for a task
aiome context "refactor the persistence layer"
aiome context --files src/auth/*.rs

# Inspect: examine the knowledge at any scale
aiome inspect --tier 3                     # all subsystem specs
aiome inspect --partition auth             # specific partition
aiome inspect --coupling                   # coupling heatmap

# Export: generate tool-specific context files
aiome export --format claude               # CLAUDE.md
aiome export --format cursor               # .cursorrules
aiome export --format mcp                  # MCP server config

# Watch: continuous maintenance loop
aiome watch                                # monitor commits, flag drift

# Agent orchestration
aiome run "implement the auth module"      # auto-routes, auto-contexts
aiome run @claude "review the PR"          # specific agent, auto-context
aiome run @all "design the API"            # all agents, parallel
```

### Example: `aiome init`

```
$ aiome init
Analyzing codebase...
  Static coupling:   405 files, 1,247 import edges
  Temporal coupling:  148 commits, 892 co-change pairs
  Test coupling:      312 test files, 1,891 coverage edges

Identifying partitions...
  Found 12 natural subsystems (modularity Q=0.73)
  3 high-coupling domains (networking, persistence, coordinates)

Generating knowledge hierarchy...
  Tier 3: 12 subsystem specs generated          [████████████] 12/12
  Tier 2: 3 domain specs generated               [███]          3/3
  Tier 1: Constitution distilled (847 lines)     [█]            1/1

Knowledge infrastructure ready.
  Total: 16 specs, ~18,400 lines
  Knowledge-to-code ratio: 17.0%

Run `aiome status` to inspect, `aiome watch` to maintain.
```

### Example: `aiome context`

```
$ aiome context "fix the desync bug in combat damage"
Task coupling analysis:
  Primary partition:  combat (coupling: 0.82)
  Coupled partitions: networking (0.91), rng (0.88), persistence (0.34)

Recommended context (high coupling):
  Tier 1: constitution.md                    (847 lines)
  Tier 2: network-protocol.md               (915 lines)
  Tier 3: combat-system.md                   (412 lines)
  Tier 3: deterministic-rng.md               (283 lines)

  Total context: 2,457 lines (~6,100 tokens)

Serve to agent? [Y/n]
```

---

## Implementation Plan

### What's Deterministic

The Analyze phase is almost entirely deterministic and does not require an LLM:

- Dependency graph extraction: AST parsing via tree-sitter
- Co-change analysis: git log processing via git2
- Community detection: Louvain or spectral clustering via petgraph (or Python networkx/scipy)
- Coupling scoring: weighted sums on the coupling graph
- Drift detection: interface comparison against stored specs

These are fast, reliable, and can run on every commit.

### What Requires an LLM

The Generate phase requires judgment:

- Generating subsystem specs from source code (extracting intent, not just structure)
- Synthesizing domain specs across multiple subsystems
- Distilling the constitution (deciding what's universal vs. local)
- Classifying whether a change is boundary-relevant or internal (for the maintenance loop)

### Language and Dependencies

**Core (Rust):**

| Crate       | Purpose                                          |
|-------------|--------------------------------------------------|
| tree-sitter | AST parsing for static analysis (multi-language) |
| git2        | Git history analysis                             |
| petgraph    | Graph algorithms for coupling and partitioning   |
| tokio       | Async runtime for agent orchestration            |
| clap        | CLI argument parsing                             |

**Analysis (Python, optional):**

| Library  | Purpose                                          |
|----------|--------------------------------------------------|
| networkx | Community detection, modularity optimization     |
| scipy    | Spectral clustering for partition identification |

The Python question: Louvain and spectral clustering have mature Python implementations (networkx, scipy) but less
mature Rust equivalents. Options: (a) implement in Rust with petgraph, accepting less sophistication initially; (b) call
Python as a subprocess for analysis; (c) use Rust bindings to C implementations. Decision deferred until Phase 1
implementation.

**LLM integration:**

- Direct API calls to Claude, GPT, Gemini for spec generation
- MCP server for context serving

### Build Phases

**Phase 1: Analyze.** Static + temporal coupling analysis. Output: coupling graph, partition structure. No LLM needed.
This alone is valuable — a visual map of your codebase's coupling structure.

**Phase 2: Generate.** LLM-powered spec generation guided by partition structure. Output: full knowledge hierarchy.
This is the expensive one-time cost.

**Phase 3: Serve.** Context computation and delivery. MCP server + CLI. This is the ongoing value.

**Phase 4: Watch.** Continuous drift detection and incremental regeneration. This is where it becomes self-maintaining.

**Phase 5: Orchestrate.** Full multi-agent routing and context-serving. This is the original aiome multi-model vision:
bounded agents coordinating through a shared knowledge substrate. The knowledge hierarchy provides three capabilities:

- **Automatic routing:** The coupling graph determines which specs a task needs, replacing hand-written trigger tables
- **Context right-sizing:** Each agent gets exactly the context its task requires, not a one-size-fits-all dump
- **Coordination through knowledge:** Agents don't need to talk to each other if they share the same specs — the specs
  are the stigmergic medium, updated by one agent's work and consumed by the next

For tasks requiring a reasoning/execution split (e.g., a planning model paired with a coding model), the same hierarchy
is sliced differently for each role. The planner gets constitution + domain specs (high-level, goal-shaped). The
executor gets subsystem specs with concrete code patterns, file paths, and function signatures (action-shaped).

---

## What This Is Not

**Not a RAG system.** RAG retrieves chunks of code based on semantic similarity. Aiome generates *knowledge about
code* — design intent, invariants, failure modes, coupling structure — organized at multiple scales. The retrieval is
coupling-driven, not embedding-driven.

**Not a documentation generator.** Documentation is written for humans. Aiome generates specs written for AI agents —
with file paths, function signatures, explicit do/don't rules, and symptom-cause-fix tables. The audience is a
boundedly rational agent that needs to make correct decisions.

**Not an agent framework.** Frameworks like AutoGen, CrewAI, and LangGraph define how agents coordinate. Aiome
structures the *knowledge* that agents depend on. It's complementary — any framework can consume Aiome's output.

**Not static.** The knowledge hierarchy is a living system. It's regenerated when coupling structure shifts, updated
when boundary-relevant changes occur, and serves different resolutions to different tasks.

---

## Open Questions

### Cold Start

On `aiome init` for a large codebase, the Generate phase must process every partition. For a 100K-line project with 12
partitions, this could mean 12+ LLM calls reading thousands of lines each. Questions:

- What's the cost (time and money) for initial generation on real-world codebases?
- Can we generate incrementally (one partition at a time) with useful intermediate output?
- Should we support a "constitution-only" mode that skips Tier 2/3 generation for quick onboarding?

### Serve-Phase Ambiguity

When a task is described only in natural language ("fix the desync bug"), mapping it to partitions requires
interpretation. Questions:

- How reliable is keyword matching against partition names and spec content?
- Should we require file paths alongside natural language descriptions?
- Is an LLM call justified for routing, or is it too expensive for a fast-path operation?
- What happens when the routing is wrong? (Agent gets insufficient context, makes bad changes)

### Quality Validation

How do we know the generated specs are good? Questions:

- What does "good" mean for an AI-targeted spec? (Measurable: does an agent make fewer mistakes with the spec than
  without?)
- Can we validate specs without a full agent task loop? (E.g., ask the LLM "given this spec, what would you do if
  asked to modify file X?" and check the answer against ground truth)
- How do we detect and correct spec drift before it causes agent errors?

### Partition Stability

Community detection algorithms can produce different partitions on small input changes. Questions:

- How stable are the partitions across minor code changes?
- Should we pin partitions and only re-detect on explicit request?
- How do we handle files that sit on partition boundaries (high coupling to multiple partitions)?

### Multi-Language Codebases

Tree-sitter supports many languages, but cross-language coupling (e.g., Rust backend calling TypeScript frontend via
API) isn't captured by import analysis. Questions:

- How do we detect cross-language coupling? (API contracts, shared types, OpenAPI specs?)
- Should cross-language boundaries always be partition boundaries?

---

## Theoretical Inspiration

The architecture described here has roots in the renormalization group from physics — a systematic procedure for
building scale-dependent descriptions of complex systems. The multi-tier knowledge hierarchy corresponds to successive
"coarse-graining" steps that preserve relevant information while discarding scale-inappropriate detail. The coupling
graph is analogous to an interaction Hamiltonian; the context budget is analogous to bond dimension in tensor networks;
module boundaries are analogous to Markov blankets in probabilistic graphical models. This design doc translates these
ideas into engineering terms.

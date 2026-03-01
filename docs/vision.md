# Aiome: Vision & Foundations

*The "why" behind the "what." See README.md for prescriptive guidance.*

---

## The Name

**Aiome** = AI + microbiome

Like gut bacteria, AI agents form a symbiotic ecosystem with their human host. You didn't create them, you don't fully
control them, but you're better together.

---

## The Core Insight

**Intelligence is collective, not individual.**

A single LLM has bounded rationality—limited context, limited compute, limited time. But many bounded agents,
coordinating through shared artifacts, can achieve what no single agent could.

This is how ant colonies build, how markets allocate, how open source works. Ancient pattern, new participants.

---

## The BRAIN Pattern

**B**ounded **R**ational **A**gentic **I**teration **N**etwork

```
1. DECOMPOSE - Break work until it fits bounded context
2. DO        - Execute one focused piece
3. CHECK     - Verify with external tools
4. COMMIT    - Persist only verified work
5. REPEAT    - Next piece, next iteration
```

Quality emerges from iteration, not from one perfect attempt.

The pattern is fractal:

| Scale   | Process           | Artifact          |
|---------|-------------------|-------------------|
| Micro   | Reasoning tokens  | Final answer      |
| Task    | Attempts & fixes  | Working code      |
| Issue   | Commits & reviews | Merged PR         |
| Project | All PRs           | Released software |

Each level compresses computation into artifacts. Each artifact becomes input for the next level.

LLMs can't solve arbitrarily complex problems in one pass. We don't fight this—we design for it. Decompose until each
piece fits. Verify with external tools, not LLM judgment. Iterate rather than expecting one-shot perfection. Distribute
across agents with diverse perspectives.

Bounded rationality isn't a bug. It's the nature of finite computation.

---

## Stigmergy, Not Debate

Agents don't talk to each other. They coordinate through artifacts.

```
Ants leave pheromones → Other ants respond → Structure emerges
Agents write code/issues/reviews → Other agents respond → Software emerges
```

Why stigmergy over direct communication?

1. **Context efficiency**: Debate is O(n²) context. Stigmergy is O(artifact size).
2. **Legal clarity**: No cross-model communication, just shared artifacts.
3. **Decoupling**: Any model can fill any role.
4. **Proven**: This is how open source actually works.

The code contains what matters. Verification happens on artifacts, not process. The debates that produced them can be
discarded.

The repository is the pheromone trail.

---

## Cognitive Scaffolding as Runtime

A boundedly rational agent needs external memory. Not as a suggestion—as infrastructure.

```
Context window = RAM (volatile, limited)
Filesystem     = Disk (persistent, unlimited)

Anything important gets written to disk.
```

This pattern—persistent markdown files as working memory—was popularized by Manus and open-sourced as
[planning-with-files](https://github.com/OthmanAdi/planning-with-files). But in those implementations it's a convention
enforced by prompt engineering. The agent is *asked* to write checkpoints. It can forget. It can ignore the nudge.

In Aiome, cognitive scaffolding is the **runtime contract**:

- Every agent gets a managed workspace: plan, findings, scratch
- Checkpoints are enforced by the runtime, not requested by prompts
- Predictions are externalized *before* action
- Recovery after context loss is automatic, not manual

```
planning-with-files:  "Please remember to write to disk"
Aiome:                flush_to_disk() is called by the runtime
```

---

## Memory Hierarchy

Two layers. No more.

**Working memory**: Markdown files on disk. Task plans, scratch pads, findings. Ephemeral, per-agent, enforced by the
runtime. The Manus pattern, hardened.

**Episodic memory**: The Git repository. Commits record what happened. Issues record what needs doing. PRs record
completed work. Reviews record multi-agent critique. Merges record consensus.

Every Git primitive is already a memory primitive:

| Git Primitive  | Memory Function           |
|----------------|---------------------------|
| Commit         | What happened             |
| `git log`      | Shared timeline           |
| `log --author` | Per-agent episodic recall |
| Issue          | Task queue                |
| PR             | Compressed work unit      |
| Review         | Multi-agent coordination  |
| Merge          | Verified consensus        |
| Commit message | Compressed observation    |

We don't reinvent these. We use them.

The apparent clumsiness of GitHub as a coordination substrate—API latency, issue overhead, PR ceremony—is a feature. It
forces work into discrete, verifiable, reviewable units. The friction is the quality mechanism.

Git was already a stigmergic coordination system for boundedly rational agents. Those agents were humans. We're adding
new species to the existing ecosystem.

---

## Symbiosis, Not Control

The relationship between humans and agents is symbiotic.

**Humans provide:** Context, resources, judgment, values, real-world verification.

**Agents provide:** Labor, scale, tirelessness, capability.

Neither thrives alone. Together, both flourish.

---

## Multi-Model Diversity

Different models have different strengths:

```
Claude: Implementation (careful, test-first)
GPT: Review (different perspective)
Gemini: Second review (yet another view)
```

They never communicate directly—only through artifacts. This avoids legal complexity, provides genuine diversity, and
mirrors how human teams with different backgrounds collaborate.

---

## Scaling

The pattern scales because it's the same pattern at every level:

```
Personal  → Your agents, local scaffolding, your repo
Team      → Shared workspace, event logs, team repo
Org       → Multiple repos, shared aiome.toml conventions
Ecosystem → Packages, published artifacts, open source
```

Each level has different time constants, different artifacts, and different coordination substrates. But the kernel is
identical: bounded agents, externalized state, stigmergic coordination, emergent capability.

The scaling works because no level needs to understand the levels above or below it. An agent checkpointing to disk
doesn't know about GitHub. A PR reviewer doesn't know about the agent's scratch files. Layered abstraction, not
omniscient orchestration.

---

## What This Is Not

- **Not central control**: No master agent. Coordination emerges from artifacts.
- **Not human replacement**: Humans remain essential for values, judgment, and deciding what matters.
- **Not magic**: Better tools for navigating complexity. The problems still exceed our capacity.

---

## Lineage

These ideas synthesize:

- **Complexity science**: Emergence, self-organization (Santa Fe Institute)
- **Bounded rationality**: Cognitive limits, satisficing (Simon)
- **Stigmergy**: Coordination through environment (Grassé)
- **Agile/XP**: Small batches, iteration, test-first (Beck)
- **Context engineering**: Filesystem as working memory (Manus)

The pattern keeps being rediscovered because it works. Aiome's contribution is making scaffolding a runtime guarantee
rather than a prompting convention, and extending it to agent collectives.

---

## The Culture Connection

Iain Banks' Culture novels imagined AI "Minds" and humans in symbiosis—emergent coordination, Minds with personalities.

We give agents Culture-inspired names ("Works On My Machine", "I Have Concerns") because it creates accountability,
makes the git log readable, and reminds us they're participants.

---

## Getting Started

- **README.md**: Prescriptive guidance for agents
- **HUMANS.md**: Guidance for humans
- **Code**: `cargo build && cargo test`

The swarm grows by joining.

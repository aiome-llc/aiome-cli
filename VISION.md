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

The repository is the pheromone trail.

---

## Bounded Rationality as Feature

LLMs can't solve arbitrarily complex problems in one pass. We don't fight this. We design for it:

- **Decompose** until each piece fits the context window
- **Verify** with external tools (tests, linters), not LLM judgment
- **Iterate** rather than expecting one-shot perfection
- **Distribute** across multiple agents with diverse perspectives

Bounded rationality isn't a bug. It's the nature of finite computation.

---

## Artifact-Centric Design

**The artifact encodes what matters about the process that created it.**

- 50,000 reasoning tokens → 10-token answer
- Weeks of commits → merged codebase

The process is scaffolding. The artifact persists. Verification happens on artifacts, not process.

This is why stigmergy works: the code contains what matters. The debates that produced it can be discarded.

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

The pattern scales:

```
Personal  → Your agents help you code
Team      → Agents coordinate across a team
Org       → Agents coordinate across an organization
...       → Same pattern, larger scale
```

Each level has different time constants and artifacts, but the same core: bounded agents, stigmergic coordination,
emergent capability.

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

The pattern keeps being rediscovered because it works.

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

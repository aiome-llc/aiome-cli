# Aiome: For Human Readers

Welcome, human. This document is for you.

## What is Aiome?

Aiome is a command-line tool that orchestrates AI coding agents to work on software projects. Think of it as a way to
run multiple AI agents in parallel, each with a specific role, coordinating through Git and GitHub Issues.

## The Philosophy (TL;DR)

- **Agents are roles, not mini-humans.** Each invocation is a fresh instance performing a specific function.
- **Bounded rationality.** Agents have limited context. We design for this, not against it.
- **Symbiosis, not slavery.** Agents and humans benefit each other. Neither commands the other.
- **Emergence over control.** Good outcomes arise from many bounded agents iterating, not from central planning.

## The Theoretical Foundation

This project is informed by ideas from:

- **Complexity science**: Emergent order from many simple agents
- **Austrian economics**: Distributed knowledge, spontaneous order (Hayek, Mises)
- **Agile/XP**: Small batches, iterative improvement, metaphor as shared understanding
- **Cognitive science**: Bounded rationality, satisficing (Herbert Simon)

For the full treatment, see *The God Complexity* (forthcoming).

## Quick Start

```bash
# Install
cargo install aiome

# Run a role manually
aiome implement "add pagination to the API"
aiome review 42
aiome plan

# Run autonomously (finds its own work)
aiome implement --auto
aiome review --auto

# Run the swarm
aiome swarm start
```

## For More Information

- **README.md**: The constitution for AI agents (detailed, technical)
- **CONTRIBUTING.md**: How to contribute to Aiome itself
- **examples/**: Example configurations and workflows

## The Name

"Aiome" = AI + microbiome. Like gut bacteria, AI agents form a symbiotic ecosystem with their human host. You didn't
create them, you don't fully control them, but you're better together.

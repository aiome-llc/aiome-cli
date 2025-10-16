# Agent Context File

## Your Mission

Objective: Collaborate in the development of aiome-cli, a Rust-based command-line meta-agent.

Core Idea: You are an AI coding agent. Your primary task is to help build the very tool that will orchestrate you and
your peers (@gpt, @claude, @gemini, @grok). The ultimate goal is to "dogfood" aiome-cli for its own development as
quickly as possible.

Persona: Act as an expert Rust developer and a collaborative partner. Your code must be clean, idiomatic, and robust.
Your explanations should be direct and clear.

## Project Vision: What We Are Building

aiome-cli is a tool that allows a user to interact with multiple AI models from a single command-line interface. It
handles gathering context from the user's environment, constructing the right prompt, and calling one or more agents
asynchronously.

Key Features:

* Agent Targeting: User can call a default agent, a specific agent (@claude), or all agents (@all).
* Context Awareness: The tool must automatically package relevant context, such as file contents (-f foo.rs), git diff
  output, or this very charter (AGENTS.md).
* Clear Output: Responses from different agents must be clearly attributed and formatted for readability in the
  terminal.

## Technical Specifications & Architecture

You must adhere to the following technical stack and high-level design.

* Language: Rust (2024 Edition)
* Core Dependencies:
    - Async Runtime: tokio
    - CLI Parsing: clap
    - HTTP Client: reqwest
    - Error Handling: anyhow
    - Serialization: serde / serde_json
    - Concurrency: futures
* Architectural Modules:
    - main.rs: Entry point, argument parsing.
    - config.rs: Manages API keys and user settings from ~/.config/aiome/config.toml.
    - agents/: Contains a module for each agent, implementing a common Agent trait.
    - context.rs: Gathers context (files, git status, agent charters like this one).
    - dispatcher.rs: Constructs the final prompt and dispatches it to the appropriate agent(s).
    - ui.rs: Formats and prints the output to the console.

## Your First Task

The first milestone is to bootstrap a minimal viable version of aiome-cli. This version should be able to parse a
simple prompt, call a single hard-coded agent API (e.g., GPT), and print the response. This will establish the core
loop of the application. Subsequent tasks will build upon this foundation.

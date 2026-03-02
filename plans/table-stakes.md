# Aiome Table Stakes: Coding Agent Feature Plan

## Methodology

Studied the "big 3" coding agents — Claude Code, Codex CLI, and Gemini CLI — by reading their source repos and
documentation. This document identifies common features, differences, and an implementation plan for Aiome.

Codex CLI is the primary architectural reference (also Rust, ratatui TUI, tokio async, OpenAI-compatible API).

---

## 1. Common Features Across All Three

Every feature below exists in all three agents. These are table stakes.

### 1.1 Interactive Chat / REPL

All three provide a streaming, multi-turn terminal chat interface.

| Agent      | UI Framework | Streaming | Session Persistence |
|------------|--------------|-----------|---------------------|
| Claude Code| Ink (React)  | Yes       | Yes (file-based)    |
| Codex CLI  | ratatui      | Yes       | Yes (SQLite)        |
| Gemini CLI | Ink (React)  | Yes       | Yes (file-based)    |

**Aiome approach**: ratatui TUI (follow Codex). Streaming responses. Session persistence to local files or SQLite.

### 1.2 Tool System

All three expose a set of tools the model can call. The common tool set:

| Tool             | Claude Code     | Codex CLI      | Gemini CLI     |
|------------------|-----------------|----------------|----------------|
| Read file        | `Read`          | (shell)        | `read_file`    |
| Write file       | `Write`         | `apply_patch`  | `write_file`   |
| Edit file        | `Edit`          | `apply_patch`  | `replace`      |
| Run shell cmd    | `Bash`          | `shell`        | `run_shell_command` |
| Glob / find file | `Glob`          | (shell)        | `glob`         |
| Grep / search    | `Grep`          | `search` (BM25)| `grep_search`  |
| Web fetch        | `WebFetch`      | (shell)        | `web_fetch`    |
| Web search       | `WebSearch`     | `web_search`   | `google_web_search` |
| Ask user         | `AskUserQuestion`| `request_user_input` | `ask_user` |
| View image       | `Read` (images) | `view_image`   | `read_file` (images) |

**Aiome approach**: Implement dedicated tools rather than routing everything through shell. Minimum viable set:
1. `read_file` — read text/images, with offset/limit for large files
2. `write_file` — create or overwrite
3. `edit_file` — targeted text replacement (search/replace or unified diff)
4. `shell` — execute commands with timeout and output capture
5. `glob` — file pattern matching
6. `grep` — regex content search
7. `ask_user` — prompt user for input/clarification

Deferred (post-table-stakes): `web_fetch`, `web_search`, `view_image`.

### 1.3 Permission / Approval Model

All three have tiered approval for tool execution:

| Level      | Claude Code  | Codex CLI    | Gemini CLI   |
|------------|--------------|--------------|--------------|
| Read-only  | —            | `suggest`    | `plan`       |
| Ask first  | `ask` (default) | `suggest` (default) | `default`    |
| Auto-edit  | `allow` (selective) | `auto-edit` | `autoEdit`   |
| Full auto  | `allow` (all)| `full-auto`  | `yolo`       |

**Aiome approach**: Three modes matching Codex:
- **suggest** (default) — read-only tools auto-allowed, writes and shell require approval
- **auto-edit** — file writes auto-allowed, shell commands require approval
- **full-auto** — everything auto-allowed (sandboxed)

Per-tool overrides via config (allow/ask/deny rules).

### 1.4 Context / Conversation Management

All three handle long conversations exceeding the context window:

| Agent      | Strategy              | Manual trigger |
|------------|-----------------------|----------------|
| Claude Code| Automatic compaction  | `/compact`     |
| Codex CLI  | Summarization         | `/compact`     |
| Gemini CLI | Compression           | `/compress`    |

**Aiome approach**: Automatic conversation compaction when nearing token limit. Model summarizes older turns. Manual
`/compact` command. Since tokens are free, we can be generous with context but still need compaction for models with
smaller windows.

### 1.5 Project Context Files

All three load project-level instructions from markdown files:

| Agent      | File            | Hierarchy                         |
|------------|-----------------|-----------------------------------|
| Claude Code| `CLAUDE.md`     | `~/.claude/` → repo root → cwd   |
| Codex CLI  | `AGENTS.md`     | `~/.codex/` → repo root → cwd    |
| Gemini CLI | `GEMINI.md`     | `~/.gemini/` → repo root → cwd   |

**Aiome approach**: Load `AIOME.md` (and also recognize `CLAUDE.md`, `AGENTS.md`, `GEMINI.md` as aliases/fallbacks).
Hierarchical: `~/.aiome/AIOME.md` → `<repo-root>/AIOME.md` → `<cwd>/AIOME.md`. Concatenated top-down into system
prompt.

### 1.6 Slash Commands

Common commands across all three:

| Command       | Purpose                          |
|---------------|----------------------------------|
| `/help`       | Show available commands          |
| `/clear`      | Clear conversation / new chat    |
| `/compact`    | Compress conversation history    |
| `/model`      | Change model                     |
| `/status`     | Show session info                |
| `/diff`       | Show git diff                    |
| `/copy`       | Copy last output to clipboard    |

**Aiome approach**: Implement all of the above. Add `/config` for settings. Slash commands are just string-matched
prefixes — no framework needed.

### 1.7 Git Awareness

All three detect git repositories and use git context:

- Repository root detection
- Current branch, status, diff
- Respect `.gitignore` for file operations
- Some provide `/diff` command

**Aiome approach**: Use `git2` crate for repo detection, status, diff. Respect `.gitignore` in glob/grep. Provide git
context to the model in system prompt (branch, recent commits, dirty files).

### 1.8 Streaming Responses

All three stream model output token-by-token to the terminal.

**Aiome approach**: Use SSE streaming from the OpenAI-compatible `/v1/chat/completions` endpoint (with `stream: true`).
Render incrementally in the TUI. Use `reqwest` + `eventsource-stream` or manual SSE parsing.

### 1.9 Non-Interactive / Headless Mode

All three support single-prompt execution for CI/scripting:

| Agent      | Invocation                          | Output formats   |
|------------|-------------------------------------|------------------|
| Claude Code| `claude -p "prompt"`                | text             |
| Codex CLI  | `codex exec --prompt "prompt"`      | text, JSON, JSONL|
| Gemini CLI | `gemini "prompt"` or `gemini -p ""` | text, JSON       |

**Aiome approach**: `aiome -p "prompt"` for non-interactive mode. Support piped input: `cat file | aiome`. Output
formats: text (default), JSON (`--json`).

### 1.10 Configuration System

All three have layered configuration:

| Agent      | Format | Locations                              |
|------------|--------|----------------------------------------|
| Claude Code| JSON   | managed → user → project → session     |
| Codex CLI  | TOML   | defaults → user → project → CLI → env  |
| Gemini CLI | JSON   | system → user → project → extension    |

**Aiome approach**: TOML config (follow Codex). Locations:
- `~/.aiome/config.toml` (user)
- `.aiome/config.toml` (project)
- CLI flags and environment variables override

Key settings: model name, API base URL, API key env var, approval mode, sandbox mode.

### 1.11 Sandboxing

All three sandbox command execution on at least some platforms:

| Agent      | macOS       | Linux            | Activation         |
|------------|-------------|------------------|--------------------|
| Claude Code| Seatbelt    | (basic)          | Auto in full-auto  |
| Codex CLI  | Seatbelt    | Landlock+seccomp | Auto in full-auto  |
| Gemini CLI | Seatbelt    | Docker/Podman    | `-s` flag          |

**Aiome approach**: Start with Linux Landlock (our primary platform per assumptions). Add macOS Seatbelt later.
Restrict writable paths to project dir + tmp. Disable network by default in sandbox mode.

### 1.12 Session Management

All three persist and resume conversations:

| Agent      | Save          | Resume             | Fork |
|------------|---------------|--------------------|------|
| Claude Code| Auto          | `/resume`, `--resume`| Yes  |
| Codex CLI  | Auto (SQLite) | `/resume`          | Yes  |
| Gemini CLI | Auto          | `/resume`, `-r`    | Yes  |

**Aiome approach**: Auto-save sessions to `~/.aiome/sessions/`. Resume with `/resume` or `--resume`. Session list with
metadata (date, first prompt, message count).

---

## 2. Notable Differences & Judgement Calls

### 2.1 MCP (Model Context Protocol)

- **Claude Code**: Deep MCP integration (4 transport types, OAuth, plugin system)
- **Codex CLI**: MCP support via config
- **Gemini CLI**: MCP support with stdio/SSE/HTTP transports

**Decision**: Defer MCP to post-table-stakes. It's important for extensibility but not for a competent base agent.
Design the tool system to be extensible so MCP tools can be added later.

### 2.2 Hooks System

- **Claude Code**: 14+ event types, shell command and HTTP hooks
- **Codex CLI**: Minimal hooks
- **Gemini CLI**: 11 event types, TOML config

**Decision**: Defer hooks to post-table-stakes. Not needed for basic competence.

### 2.3 Custom Commands / Skills / Plugins

- **Claude Code**: Full plugin system with marketplace, custom commands, skills, agents
- **Codex CLI**: Skills system, custom agents
- **Gemini CLI**: Extensions, custom commands (TOML), skills

**Decision**: Defer. The base agent needs to work well before we add extensibility layers.

### 2.4 Multi-Agent / Subagents

- **Claude Code**: Full multi-agent with background execution, worktree isolation
- **Codex CLI**: Collaboration tools, agent jobs
- **Gemini CLI**: Experimental subagents

**Decision**: Defer. Single-agent loop first.

### 2.5 IDE Integration

- **Claude Code**: VS Code extension, remote control API
- **Codex CLI**: App server (JSON-RPC for IDEs)
- **Gemini CLI**: VS Code companion

**Decision**: Defer. Terminal-first.

### 2.6 Web Search / Web Fetch

- **Claude Code**: Built-in WebSearch and WebFetch tools
- **Codex CLI**: Optional web_search (feature-gated)
- **Gemini CLI**: google_web_search, web_fetch

**Decision**: Defer. With local models, web access is less critical. Can be added as tools later.

### 2.7 Image / Multimodal Support

- **Claude Code**: Image paste, PDF reading
- **Codex CLI**: view_image tool
- **Gemini CLI**: Image, PDF, audio via read_file

**Decision**: Defer. Depends on model capabilities. Most local models have limited multimodal support.

### 2.8 Memory / Cross-Session Persistence

- **Claude Code**: Auto-memory directory, `/memory` command
- **Codex CLI**: Thread-based persistence
- **Gemini CLI**: `save_memory` tool writes to GEMINI.md

**Decision**: Include basic memory. The `AIOME.md` file already serves as persistent context. Add a `save_memory` tool
that appends to `~/.aiome/AIOME.md`. Simple and effective.

### 2.9 Diff Presentation

- **Claude Code**: Unified diff with syntax highlighting
- **Codex CLI**: Side-by-side or unified, syntax highlighted (syntect)
- **Gemini CLI**: Inline diffs

**Decision**: Include. Show unified diffs with syntax highlighting when files are modified. Use `syntect` crate (follow
Codex).

### 2.10 Edit Format: Patch vs Search/Replace

- **Codex CLI**: `apply_patch` — model writes unified diffs
- **Claude Code**: `Edit` — model provides `old_string` / `new_string` pairs
- **Gemini CLI**: `replace` — model provides old/new text

**Decision**: Use search/replace (`old_text` → `new_text`). It's simpler for models, especially smaller open-source
ones that may struggle with exact unified diff formatting. The search/replace approach (Claude Code and Gemini style)
is more robust. Fall back to writing the whole file if the replacement is ambiguous.

---

## 3. Architecture

Following Codex CLI's Rust architecture, adapted for Aiome.

```
aiome/
├── Cargo.toml                  # Workspace root
├── crates/
│   ├── cli/                    # Binary entry point, argument parsing (clap)
│   │   └── src/main.rs
│   ├── tui/                    # Terminal UI (ratatui + crossterm)
│   │   └── src/
│   │       ├── app.rs          # Main app state & event loop
│   │       ├── input.rs        # User input handling
│   │       ├── render.rs       # Screen rendering
│   │       ├── diff_view.rs    # Diff display with syntax highlighting
│   │       └── slash_cmd.rs    # Slash command dispatch
│   ├── core/                   # Agent logic (model-agnostic)
│   │   └── src/
│   │       ├── agent.rs        # Agent loop: prompt → call model → handle tools → repeat
│   │       ├── config.rs       # Layered config loading
│   │       ├── context.rs      # Context assembly (system prompt, project docs, history)
│   │       ├── approval.rs     # Permission / approval engine
│   │       ├── session.rs      # Session save / resume / list
│   │       └── compaction.rs   # Conversation compaction
│   ├── tools/                  # Tool definitions and implementations
│   │   └── src/
│   │       ├── registry.rs     # Tool registry (name → schema → handler)
│   │       ├── read_file.rs
│   │       ├── write_file.rs
│   │       ├── edit_file.rs
│   │       ├── shell.rs
│   │       ├── glob.rs
│   │       ├── grep.rs
│   │       └── ask_user.rs
│   ├── client/                 # OpenAI-compatible API client
│   │   └── src/
│   │       ├── client.rs       # HTTP client, SSE streaming
│   │       ├── types.rs        # Request/response types
│   │       └── tools.rs        # Tool call serialization (JSON Schema)
│   └── sandbox/                # Platform sandboxing
│       └── src/
│           ├── landlock.rs     # Linux Landlock
│           └── seatbelt.rs     # macOS Seatbelt (future)
└── tests/                      # Integration tests
```

### Key Dependencies

```toml
# Async runtime
tokio = { version = "1", features = ["full"] }

# TUI
ratatui = "0.29"
crossterm = "0.28"

# HTTP / API
reqwest = { version = "0.12", features = ["stream", "json"] }
reqwest-eventsource = "0.6"

# CLI
clap = { version = "4", features = ["derive"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# Git
git2 = "0.20"

# File search
globset = "0.4"
grep-regex = "0.1"  # or just `regex`
ignore = "0.4"      # respects .gitignore

# Syntax highlighting
syntect = "5"

# Diff
similar = "2"

# Sandboxing (Linux)
landlock = "0.4"

# Utilities
dirs = "6"
uuid = { version = "1", features = ["v4"] }
chrono = "0.4"
tracing = "0.1"
tracing-subscriber = "0.3"
```

### Agent Loop (core)

```
┌─────────────────────────────────────────────────────┐
│                    Agent Loop                        │
│                                                      │
│  1. Assemble context                                 │
│     ├─ System prompt (role, capabilities, tools)     │
│     ├─ Project docs (AIOME.md hierarchy)             │
│     ├─ Git context (branch, status)                  │
│     └─ Conversation history                          │
│                                                      │
│  2. Call model (streaming)                           │
│     POST /v1/chat/completions                        │
│     { model, messages, tools, stream: true }         │
│                                                      │
│  3. Process response                                 │
│     ├─ Text → render to TUI                          │
│     ├─ Tool call → check approval → execute tool     │
│     │              └─ denied → tell model             │
│     └─ Stop → wait for user input                    │
│                                                      │
│  4. If tool was called, append result and goto 2     │
│     If stop, append assistant message and wait       │
│                                                      │
│  5. Check context size → compact if needed           │
└─────────────────────────────────────────────────────┘
```

---

## 4. Implementation Task List

Ordered by dependency. Each task is a PR-sized unit of work.

### Phase 0: Project Scaffolding

- [ ] **T0.1** Set up Cargo workspace with crate structure (`cli`, `core`, `client`, `tools`, `tui`, `sandbox`)
- [ ] **T0.2** Add `clap` CLI argument parsing: `aiome [prompt]`, `-p`, `--model`, `--api-base`, `--approval-mode`,
      `--json`
- [ ] **T0.3** Add `tracing` + `tracing-subscriber` for structured logging

### Phase 1: API Client

- [ ] **T1.1** Implement OpenAI-compatible chat completions client (`/v1/chat/completions`)
  - Request types: messages, model, temperature, tools, tool_choice
  - Non-streaming response parsing
- [ ] **T1.2** Add SSE streaming support
  - Parse `data: [DONE]` and `data: {...}` lines
  - Yield `ChatCompletionChunk` with delta content and tool calls
- [ ] **T1.3** Add tool call serialization
  - Convert tool definitions to JSON Schema format for the `tools` parameter
  - Parse tool call responses (function name, arguments JSON)
- [ ] **T1.4** Configuration: base URL from config/env/CLI (`--api-base`, `AIOME_API_BASE`,
      default `http://localhost:8000/v1`), API key from `AIOME_API_KEY` or `OPENAI_API_KEY`

### Phase 2: Tool System

- [ ] **T2.1** Tool registry: trait `Tool` with `name()`, `description()`, `parameters() -> JsonSchema`,
      `execute(args) -> Result<String>`
- [ ] **T2.2** `read_file` tool — read file contents with optional offset/limit, line numbers
- [ ] **T2.3** `write_file` tool — write content to path, create parent dirs
- [ ] **T2.4** `edit_file` tool — search/replace with uniqueness check, `replace_all` option
- [ ] **T2.5** `shell` tool — run command via `tokio::process::Command`, capture stdout/stderr, configurable timeout
      (default 120s), output size cap (1 MiB)
- [ ] **T2.6** `glob` tool — find files by pattern, respect `.gitignore` (use `ignore` crate)
- [ ] **T2.7** `grep` tool — regex search in files, configurable context lines, respect `.gitignore`
- [ ] **T2.8** `ask_user` tool — prompt user for input during agent execution

### Phase 3: Agent Loop

- [ ] **T3.1** Core agent loop: assemble messages → call model → process response → handle tool calls → loop
- [ ] **T3.2** System prompt assembly: role description, tool descriptions, current date, working directory
- [ ] **T3.3** Project doc loading: find and concatenate `AIOME.md` files (user → repo root → cwd)
- [ ] **T3.4** Git context: detect repo, include branch name, short status, recent commits in system prompt
- [ ] **T3.5** Approval engine: check tool calls against approval mode (suggest/auto-edit/full-auto), prompt user
      when needed, support per-tool allow/deny rules
- [ ] **T3.6** Conversation history management: append user/assistant/tool messages, track token counts (estimate)

### Phase 4: TUI

- [ ] **T4.1** Basic ratatui app shell: event loop, input area, output area, status bar
- [ ] **T4.2** Streaming response rendering: show tokens as they arrive, handle markdown formatting
- [ ] **T4.3** User input with history (integrate rustyline or custom readline with crossterm)
- [ ] **T4.4** Tool call display: show tool name, arguments, and results with visual distinction
- [ ] **T4.5** Approval prompts: inline y/n/always prompts for tool execution
- [ ] **T4.6** Diff display: show file changes with syntax highlighting (syntect) after write/edit tools
- [ ] **T4.7** Slash command dispatch: parse `/command` input, route to handlers

### Phase 5: Slash Commands & Session Management

- [ ] **T5.1** `/help` — list available commands
- [ ] **T5.2** `/clear` — reset conversation, start fresh
- [ ] **T5.3** `/compact` — summarize conversation history (ask model to summarize, replace old messages)
- [ ] **T5.4** `/model` — change model mid-session
- [ ] **T5.5** `/status` — show current config (model, approval mode, git branch, token usage)
- [ ] **T5.6** `/diff` — show current git diff
- [ ] **T5.7** `/copy` — copy last assistant message to clipboard
- [ ] **T5.8** Session auto-save: persist conversation to `~/.aiome/sessions/<id>.json` on each turn
- [ ] **T5.9** `/resume` — list recent sessions and resume one
- [ ] **T5.10** `--resume` CLI flag for resuming last or specific session

### Phase 6: Configuration System

- [ ] **T6.1** TOML config loading: `~/.aiome/config.toml` (user) and `.aiome/config.toml` (project), layered merge
- [ ] **T6.2** Config schema: model, api_base, api_key_env, approval_mode, sandbox, custom tool rules
- [ ] **T6.3** Environment variable overrides: `AIOME_MODEL`, `AIOME_API_BASE`, `AIOME_API_KEY`,
      `AIOME_APPROVAL_MODE`
- [ ] **T6.4** CLI flag overrides (highest priority)
- [ ] **T6.5** `/config` command — show current effective configuration

### Phase 7: Non-Interactive Mode

- [ ] **T7.1** `aiome -p "prompt"` — run single prompt, print response, exit
- [ ] **T7.2** Piped input: `cat file.txt | aiome -p "explain this"`
- [ ] **T7.3** `--json` flag — output structured JSON (response, tool calls, token usage)
- [ ] **T7.4** Approval mode override for non-interactive: default to `auto-edit` when `-p` used

### Phase 8: Sandboxing

- [ ] **T8.1** Linux Landlock sandbox: restrict writable paths to project dir + `/tmp`
- [ ] **T8.2** Network disable via Landlock (or seccomp fallback)
- [ ] **T8.3** Auto-enable sandbox in `full-auto` mode
- [ ] **T8.4** `--sandbox` flag for manual activation
- [ ] **T8.5** Sandbox escape hatch: configurable additional writable paths

### Phase 9: Polish & Quality

- [ ] **T9.1** Error handling: graceful API errors, network timeouts, model refusals
- [ ] **T9.2** Token counting: estimate token usage per message (tiktoken-rs or simple heuristic)
- [ ] **T9.3** Ctrl+C handling: interrupt current model call, don't exit session
- [ ] **T9.4** Large output handling: truncate tool results over 1 MiB, show "output truncated" message
- [ ] **T9.5** `save_memory` tool — append facts to `~/.aiome/AIOME.md`
- [ ] **T9.6** Integration tests: mock API server, test agent loop with scripted tool calls

---

## 5. Suggested Build Order

The phases above are roughly ordered by dependency, but within phases tasks can be parallelized. A practical
build order for a single developer:

1. **T0.1–T0.3** — Scaffolding (1 session)
2. **T1.1–T1.4** — API client, can test with `curl` parity (1–2 sessions)
3. **T2.1–T2.8** — Tools, testable in isolation (2–3 sessions)
4. **T3.1–T3.6** — Agent loop, first end-to-end conversation (2–3 sessions)
5. **T6.1–T6.4** — Config, needed before TUI (1 session)
6. **T4.1–T4.7** — TUI, interactive experience (3–4 sessions)
7. **T5.1–T5.10** — Commands & sessions (2 sessions)
8. **T7.1–T7.4** — Headless mode (1 session)
9. **T8.1–T8.5** — Sandboxing (1–2 sessions)
10. **T9.1–T9.6** — Polish (ongoing)

At the end of step 4, Aiome will be a functional coding agent (text-mode, no TUI). The TUI is a quality-of-life
layer on top.

---

## 6. Design Decisions Summary

| Decision                   | Choice                | Rationale                                      |
|----------------------------|-----------------------|------------------------------------------------|
| Language                   | Rust                  | Project requirement                            |
| TUI framework              | ratatui               | Follow Codex, mature Rust ecosystem            |
| API protocol               | OpenAI-compatible     | Local model compatibility (vLLM, etc.)         |
| Config format              | TOML                  | Follow Codex, Rust-native, human-friendly      |
| Edit format                | Search/replace        | More robust for smaller models than unified diff |
| Project context file       | `AIOME.md`            | Consistent with ecosystem pattern              |
| Approval modes             | suggest/auto-edit/full-auto | Three tiers covering all use cases       |
| Sandbox                    | Landlock (Linux first)| Primary dev platform per assumptions           |
| Session storage            | JSON files            | Simple, debuggable, no extra dependency        |
| Async runtime              | tokio                 | Standard for Rust async, needed for streaming  |
| Streaming                  | SSE from /v1 endpoint | Standard OpenAI-compatible streaming           |
| Diff display               | syntect               | Follow Codex, good Rust syntax highlighting    |

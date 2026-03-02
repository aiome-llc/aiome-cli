# Aiome table stakes

Before embarking on the renormalization engine, let's make Aiome a competent coding agent in its own right.

Study the "big 3" agents:
* Claude Code
  * repo: https://github.com/anthropics/claude-code (although this doesn't contain the full source)
  * docs: https://code.claude.com/docs/en/overview
* Codex CLI
  * repo: https://github.com/openai/codex
  * docs: https://developers.openai.com/codex/cli/
* Gemini CLI
  * repo: https://github.com/google-gemini/gemini-cli
  * docs: https://geminicli.com/docs/

Do the following:
* Find the common features and document them.
* Since we are also using Rust, the Codex CLI can be the base reference.
* For differences between them, use your best judgement on whether to include and how to homogenize.

Assumptions:
* We'll mostly rely on a locally deployed open source model for testing.
* Connect to it via http://localhost:8000/v1, OpenAI API compatible.
* Model can be GLM-5, Kimi K2.5, MiniMax M2.5, or Qwen 3.5.
* Tokens are free.

Document your findings and create a plan in `table-stakes.md`. Add a task list for implementation.

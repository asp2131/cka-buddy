# AGENTS.md

## Cursor Cloud specific instructions

This is a single-crate Rust TUI application (no monorepo, no services, no Docker).

### Quick reference

| Task | Command |
|---|---|
| Build | `cargo build` |
| Lint | `cargo clippy` |
| Test | `cargo test` (no tests exist yet — 0 test cases) |
| Run (embedded shell) | `cargo run -- --shell embedded` |
| Run (default external) | `cargo run` |
| Run with LLM hints | `cargo run --features llm` |

### Gotchas

- **Rust edition 2024** requires rustc >= 1.85. The VM ships with 1.83; the update script runs `rustup update stable && rustup default stable` to upgrade.
- **zsh is a hard dependency** — the app hardcodes `/bin/zsh` for PTY and verify commands. The update script installs it if missing.
- The default `cargo run` (external shell mode) expects a second terminal to pair via a 4-digit PIN. Use `--shell embedded` to run the full TUI in a single terminal without pairing.
- Progress is stored at `~/.cka-coach/progress.json` (local filesystem, no database).
- Content lives in `docs/cka-app-content/` — Markdown files with `cka-step` fenced blocks.
- The `extensions/` directory contains IDE agent extensions (TypeScript) and is **not** part of the Rust build.

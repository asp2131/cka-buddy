# CKA Terminal App Content Pack (v1.34)

This folder is a ready-to-use markdown curriculum for a Rust terminal app that teaches CKA through project-based labs and realistic break/fix scenarios.

The intended UX is a coaching loop: open app -> see readiness progress -> read current objective -> run guided commands directly in-app -> verify -> advance or jump back.

## Design Goals

- Focus on post-2025 CKA scope (Gateway API, Helm, Kustomize, CRDs/Operators).
- Bias toward troubleshooting and cluster admin workflows (55% combined exam weight).
- Teach with short projects, then inject exam-like bugs and time pressure.
- Keep all content markdown-first so the app can render in TUI views.

## Suggested App Flow

1. Start app and load `docs/cka-app-content/app-loop-spec.md`.
2. Show learner readiness score and current section from saved progress.
3. Show short objective blurb + command checklist for current step.
4. Execute typed commands inside the app terminal pane.
5. Verify outcome and mark step complete.
6. Allow jump-back navigation to any previous completed step.

## Parsing Convention

Each project and bug file starts with a small metadata block:

```text
id: ...
type: project|bug|exam
domains: ...
difficulty: beginner|intermediate|advanced
timebox_min: ...
```

This is intentionally simple to parse with Rust (line-based split on `:`) before you add richer frontmatter support.

## App Runtime Note

You do not need to build a terminal emulator from scratch. Use a PTY-backed shell inside your TUI:

- `ratatui` for layout/state.
- `crossterm` for input/events.
- `portable-pty` (or `tokio-pty-process`) to run shell commands in a pseudo-terminal.
- Optional `vt100` parser to render ANSI output cleanly.

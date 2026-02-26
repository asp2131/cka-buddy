# CKA Buddy TUI (Rust MVP)

Local-first CKA coaching terminal app with guided steps and in-app command execution.

## Run

```bash
cargo run
```

## Keys

- `q`: quit
- `j`/`k` or arrow keys: next/previous step
- `b`: jump back to previous completed step
- `r`: jump to recommended next step
- `TAB`: load first suggested command
- `Enter`: run current command in embedded shell
- `v`: run deterministic verify commands for current step
- `h`: request a coaching hint
- `! <command>`: force-run a high-risk command that would otherwise require confirmation

## Notes

- Content source: `docs/cka-app-content/`
- Progress save file: `~/.cka-coach/progress.json`
- Verification is deterministic and based on step `verify` commands.

## Strict Runnable Blocks

Project and bug markdown files now support `cka-step` fenced blocks that define exact runnable steps.

## Optional LLM Coach

Default mode is deterministic coaching (no API dependency).

To enable LLM hints:

```bash
cargo run --features llm
```

Set environment variables:

```bash
export CKA_COACH_LLM_API_KEY="<key>"
export CKA_COACH_LLM_ENDPOINT="https://openrouter.ai/api/v1/chat/completions"
export CKA_COACH_LLM_MODEL="openai/gpt-4.1-mini"
```

If you use a different OpenAI-compatible endpoint, set `CKA_COACH_LLM_ENDPOINT` accordingly.

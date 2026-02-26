# ratatui-rebels skill

This directory contains a reusable OpenCode skill based on the UI architecture used in `rebels-in-the-sky`.

## Files

- `SKILL.md`: main skill instructions.
- `references/rebels-ratatui-patterns.md`: source-derived architecture and pattern map.

## Registering in OpenCode

If your OpenCode instance only loads skills from `~/.config/opencode/skills`, copy this folder there:

```bash
mkdir -p ~/.config/opencode/skills
cp -R skills/ratatui-rebels ~/.config/opencode/skills/
```

Then restart your session so the skill list refreshes.

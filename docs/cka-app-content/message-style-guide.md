# Message Style Guide (Coach Voice)

id: message-style-v1
type: style
domains: all
difficulty: n/a
timebox_min: 0

## Rules

- Keep instructions to 1-3 lines.
- Prefer command-first guidance.
- Avoid multi-paragraph explanations by default.
- Use plain language and exam-focused wording.
- Always include optional verification when relevant.

## Templates

### Step Prompt

```text
Objective: <one sentence>
Run:
1) <command>
2) <command>
```

### Success Prompt

```text
Done: <one sentence>
Next:
1) <command>
Verify (optional):
- <read-only command>
```

### Failure Prompt

```text
Not yet: <one sentence>
Hint: <one short hint>
Retry:
1) <command>
Verify (optional):
- <read-only command>
```

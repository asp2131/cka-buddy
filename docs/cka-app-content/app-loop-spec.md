# App Loop Spec (Coach Mode)

id: app-loop-v1
type: app-spec
domains: all
difficulty: n/a
timebox_min: 0

## Product Vision

The app behaves like a live pair-programming coach for CKA prep.

## Required Loop

1. App starts and loads progress state.
2. App shows current readiness to pass CKA (0-100%).
3. App shows current section and allows jumping to previous completed sections.
4. App shows a short objective blurb for the active step.
5. App shows command(s) to run next.
6. Learner runs command in the in-app terminal pane.
7. App checks expected output/state and marks step complete.
8. Readiness score updates and next step is suggested.

## Minimal Data Contract Per Step

Use this structure for each runnable step your app loads:

```text
step_id: p04-s02
title: create gateway listener
objective: expose HTTP listener on port 80 for team1 namespace
ready_weight: 2
commands:
  - kubectl apply -f gateway.yaml
  - kubectl get gateway -n team1
success_check:
  - kubectl get gateway -n team1 team1-gw -o jsonpath='{.status.conditions[0].status}'
success_contains: True
fallback_hint: check gatewayClassName and listener port/protocol
```

The parser also supports fenced strict markdown blocks:

```text
```cka-step
step_id: p04-s02
title: attach httproute
objective: attach route to gateway
commands:
  - kubectl apply -f httproute.yaml
success_check:
  - kubectl describe httproute -n team1 app-route
success_contains:
  - Accepted
```
```

## Progress Model

- `readiness_score`: weighted completed steps / total weighted steps.
- `current_track`: `foundations|networking|admin|troubleshooting|exam`.
- `current_step_id`: active step.
- `completed_steps[]`: replayable history for jump-back.

## Navigation Rules

- Allow moving to any completed step at any time.
- Allow skipping forward only when current step is explicitly marked `optional`.
- Keep one-click return to `recommended next` step.

## Terminal Execution Rules

- Run commands in PTY shell owned by app process.
- Stream stdout/stderr live in output pane.
- Preserve shell session state (`kubectl config`, exported vars, aliases).
- Block destructive host-level commands outside training scope.

## Safety Guardrails

- Confirm before running commands matching: `rm -rf`, `shutdown`, `reboot`, `mkfs`, `:(){:|:&};:`.
- Show warning when command touches cluster-wide resources in non-lab namespace.

## UX Copy Pattern

- `Progress`: "CKA readiness: 47% (Networking track in progress)"
- `Objective`: one sentence max.
- `Next command`: one to three commands max, numbered.
- `Why this matters`: optional one-line exam relevance.

## Response Contract (Short + Verifiable)

After each successful step, the app must show a compact completion card:

- `Done`: one short sentence.
- `What changed`: max 2 bullets.
- `Next`: max 2 commands.
- `Verify (optional)`: 1 to 2 read-only commands.

Keep every line short. Do not show long explanations unless learner requests `explain`.

### Example Success Card

```text
Done: Gateway listener is active in team1.
What changed:
- team1-gw accepted by controller
- HTTPRoute attached to parent gateway
Next:
1) kubectl apply -f httproute.yaml
2) kubectl get httproute -n team1
Verify (optional):
- kubectl describe httproute -n team1 app-route
```

## Verification Modes

- `auto_verify`: run hidden success checks after command execution.
- `show_verify`: display optional verify commands for learner confidence.
- `strict_verify`: require learner to run verify command before unlock.

Default mode: `auto_verify` + `show_verify`.

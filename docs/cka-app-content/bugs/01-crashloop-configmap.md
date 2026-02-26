# Bug 01 - CrashLoop from Missing Config Key

id: b01
type: bug
domains: troubleshooting,workloads
difficulty: intermediate
timebox_min: 12

## Broken State

App starts with `CONFIG_MODE` env var, but ConfigMap key is misspelled.

## Objective

Restore pod to `Running` without recreating namespace.

## Expected Workflow

- `kubectl describe pod` -> inspect events.
- `kubectl logs --previous` -> read startup error.
- Patch ConfigMap key or deployment env reference.
- Restart rollout and verify readiness.

## Runnable Steps (Strict)

```cka-step
step_id: b01-s01
title: crashloop root cause
objective: Identify config key mismatch and restore pod startup.
ready_weight: 2
commands:
  - kubectl describe pod <pod> -n <ns>
  - kubectl logs <pod> -n <ns> --previous
  - kubectl get configmap <cm> -n <ns> -o yaml
success_check:
  - kubectl get pods -n <ns>
success_contains:
  - Running
what_changed:
  - Bad config key corrected
  - Pod recovered from CrashLoop
fallback_hint: Compare env valueFrom key with exact ConfigMap data key.
```

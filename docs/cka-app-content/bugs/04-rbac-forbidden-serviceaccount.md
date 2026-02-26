# Bug 04 - RBAC Forbidden for ServiceAccount

id: b04
type: bug
domains: architecture,troubleshooting
difficulty: intermediate
timebox_min: 15

## Broken State

ServiceAccount exists, but binding references wrong subject namespace.

## Objective

Grant only required verb/resource in the correct namespace.

## Validation

- `kubectl auth can-i ...` returns yes for target action.
- Non-required action remains denied.

## Runnable Steps (Strict)

```cka-step
step_id: b04-s01
title: fix rolebinding subject namespace
objective: Bind correct serviceaccount subject and restore expected permission.
ready_weight: 2
commands:
  - kubectl get rolebinding -n <ns> <rb> -o yaml
  - kubectl apply -f rolebinding-fixed.yaml
  - kubectl auth can-i --as=system:serviceaccount:<ns>:<sa> get pods -n <ns>
success_check:
  - kubectl auth can-i --as=system:serviceaccount:<ns>:<sa> get pods -n <ns>
success_contains:
  - yes
what_changed:
  - Subject namespace corrected
  - ServiceAccount permission restored
fallback_hint: subjects[].namespace must match the serviceaccount namespace.
```

# Bug 03 - NetworkPolicy Blocks App Path

id: b03
type: bug
domains: networking,troubleshooting
difficulty: intermediate
timebox_min: 14

## Broken State

Default-deny policy exists, but required allow rule is missing namespace selector.

## Objective

Allow only approved frontend namespace to call backend on app port.

## Validation

- Approved namespace succeeds.
- Unapproved namespace remains blocked.

## Runnable Steps (Strict)

```cka-step
step_id: b03-s01
title: restore allowed traffic path
objective: Add missing namespace selector to allow frontend namespace.
ready_weight: 2
commands:
  - kubectl get netpol -n <ns>
  - kubectl apply -f allow-frontend.yaml
success_check:
  - kubectl describe netpol -n <ns> <policy>
success_contains:
  - namespaceSelector
what_changed:
  - Allow rule narrowed to approved namespace
  - Required app flow restored
fallback_hint: Keep default-deny in place; only add precise allow.
```

# Bug 06 - Gateway Route Attached but No Traffic

id: b06
type: bug
domains: networking,troubleshooting
difficulty: advanced
timebox_min: 18

## Broken State

`HTTPRoute` has incorrect `parentRefs` namespace and backend service port mismatch.

## Objective

Fix route attachment and backend forwarding without deleting GatewayClass.

## Validation

- `kubectl describe httproute` shows accepted + resolved refs.
- Host/path curl hits backend successfully.

## Runnable Steps (Strict)

```cka-step
step_id: b06-s01
title: fix gateway route references
objective: Correct parentRefs namespace and backend service port mapping.
ready_weight: 3
commands:
  - kubectl get httproute -n <ns> <route> -o yaml
  - kubectl apply -f httproute-fixed.yaml
success_check:
  - kubectl describe httproute -n <ns> <route>
success_contains:
  - Accepted
what_changed:
  - Route attached to intended gateway
  - BackendRef now points to correct service port
fallback_hint: parentRefs must reference the gateway namespace when cross-namespace is used.
```

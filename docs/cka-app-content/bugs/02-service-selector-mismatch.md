# Bug 02 - Service Has No Endpoints

id: b02
type: bug
domains: networking,troubleshooting
difficulty: beginner
timebox_min: 10

## Broken State

Service selector label does not match deployment pod labels.

## Objective

Re-establish internal traffic from caller pod to target service.

## Validation

- `kubectl get svc,endpoints -n <ns>` shows at least one endpoint.
- Curl from caller pod returns HTTP 200.

## Runnable Steps (Strict)

```cka-step
step_id: b02-s01
title: repair service selector
objective: Match service selector labels to target pods.
ready_weight: 2
commands:
  - kubectl get svc <svc> -n <ns> -o yaml
  - kubectl get pods -n <ns> --show-labels
  - kubectl patch svc <svc> -n <ns> -p '<patch-json>'
success_check:
  - kubectl get endpoints <svc> -n <ns>
success_contains:
  - addresses
what_changed:
  - Service selector corrected
  - Endpoints registered
fallback_hint: Use exact key/value label pairs from target pods.
```

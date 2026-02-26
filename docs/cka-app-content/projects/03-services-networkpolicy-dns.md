# Project 03 - Services, NetworkPolicy, and DNS

id: p03
type: project
domains: networking,troubleshooting
difficulty: intermediate
timebox_min: 210

## Mission

Create app-to-app connectivity with explicit policy and verify DNS behavior across namespaces.

## Deliverables

- Deploy `frontend` and `api` in separate namespaces.
- Expose `api` with ClusterIP service.
- Apply default-deny policy, then allow only frontend -> api on target port.
- Validate DNS using `nslookup` from a debug pod.

## Verification

- `kubectl get svc,endpoints -A`
- `kubectl get netpol -A`
- `kubectl exec -it <debug-pod> -- nslookup api.<ns>.svc.cluster.local`

## Exam Trap to Practice

Service selector labels do not match pod labels; endpoints list is empty.

## Runnable Steps (Strict)

```cka-step
step_id: p03-s01
title: service and endpoints
objective: Create api service and confirm endpoints are populated.
ready_weight: 2
commands:
  - kubectl -n api expose deployment api --port=80 --target-port=8080 --type=ClusterIP
  - kubectl -n api get endpoints api
success_check:
  - kubectl -n api get endpoints api
success_contains:
  - "addresses"
what_changed:
  - Service api exposed
  - Endpoints now map to pods
fallback_hint: Service selector must match deployment pod labels.
```

```cka-step
step_id: p03-s02
title: default deny and allow
objective: Apply default-deny, then allow frontend to api.
ready_weight: 3
commands:
  - kubectl -n api apply -f default-deny.yaml
  - kubectl -n api apply -f allow-frontend.yaml
success_check:
  - kubectl -n api get netpol
success_contains:
  - default-deny
what_changed:
  - Namespace now policy-controlled
  - Explicit allow path added
fallback_hint: Include namespaceSelector and podSelector together in allow rule.
```

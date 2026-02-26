# Project 04 - Ingress to Gateway API Migration

id: p04
type: project
domains: networking,architecture
difficulty: advanced
timebox_min: 240

## Mission

Migrate a legacy Ingress routing model to Gateway API resources.

## Deliverables

- Inspect an existing Ingress and map hosts/paths to route rules.
- Ensure Gateway API CRDs exist (`GatewayClass`, `Gateway`, `HTTPRoute`).
- Create a namespace-scoped `Gateway` and attach route via `parentRefs`.
- Configure `backendRefs` to the target service and validate traffic.

## Verification

- `kubectl get gatewayclass,gateway,httproute -A`
- `kubectl describe httproute -n <ns> <name>`
- Run a curl pod and test host/path routing.

## Exam Trap to Practice

`HTTPRoute` accepted but no traffic due to wrong `parentRefs` name/namespace.

## Runnable Steps (Strict)

```cka-step
step_id: p04-s01
title: create gateway listener
objective: Create a namespace gateway listening on HTTP port 80.
ready_weight: 3
commands:
  - kubectl apply -f gateway.yaml
success_check:
  - kubectl get gateway -n team1
success_contains:
  - team1-gw
what_changed:
  - Gateway resource applied
  - Listener created for HTTP traffic
fallback_hint: Confirm gatewayClassName and listener protocol/port.
```

```cka-step
step_id: p04-s02
title: attach httproute
objective: Attach HTTPRoute to gateway and forward to backend service.
ready_weight: 4
commands:
  - kubectl apply -f httproute.yaml
success_check:
  - kubectl describe httproute -n team1 app-route
success_contains:
  - Accepted
what_changed:
  - Route attached to parent gateway
  - Backend service forwarding active
fallback_hint: parentRefs namespace/name must match the gateway exactly.
```

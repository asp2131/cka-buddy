# Project 04 - Gateway API: Expose the App Externally

id: p04
type: project
domains: networking,architecture
difficulty: advanced
timebox_min: 240

## Mission

Expose the two-tier frontend to traffic outside the cluster using the Gateway API. This is the post-2025 CKA replacement for Ingress.

## Context

The `two-tier` app is fully internal: frontend and backend communicate via ClusterIP services, locked down with NetworkPolicy. Now you need external users to reach the frontend. On the CKA exam, you will be asked to create Gateway, GatewayClass, and HTTPRoute resources.

## Concepts

- **GatewayClass**: defines which controller implements gateways (like IngressClass). Usually pre-installed.
- **Gateway**: a load balancer entry point. Declares listeners (protocol + port + hostname).
- **HTTPRoute**: routes traffic from a Gateway listener to backend services. Replaces Ingress path rules.
- **parentRefs**: how an HTTPRoute attaches to a Gateway. Must match name and namespace exactly.
- **backendRefs**: where the HTTPRoute sends traffic. Points to a Service.

## Deliverables

- Install Gateway API CRDs (if not present).
- Create a GatewayClass and Gateway with an HTTP listener.
- Create an HTTPRoute that sends traffic to the frontend service.
- Verify end-to-end traffic flow.
- Understand the Ingress -> Gateway API mental model.

## Runnable Steps (Strict)

```cka-step
step_id: p04-s01
title: install gateway api crds
objective: Ensure Gateway API CRDs are available in the cluster.
ready_weight: 2
commands:
  - kubectl apply -f https://github.com/kubernetes-sigs/gateway-api/releases/download/v1.2.0/standard-install.yaml
  - kubectl get crd gatewayclasses.gateway.networking.k8s.io
success_check:
  - kubectl get crd gatewayclasses.gateway.networking.k8s.io
success_contains:
  - gatewayclasses.gateway.networking.k8s.io
what_changed:
  - Gateway API CRDs installed (GatewayClass, Gateway, HTTPRoute, etc.)
  - These are just definitions — no controller running yet
fallback_hint: The CRDs must exist before you can create any Gateway resources.
```

```cka-step
step_id: p04-s02
title: create gatewayclass
objective: Define which controller will implement gateways.
ready_weight: 2
commands:
  - "kubectl apply -f - <<EOF\napiVersion: gateway.networking.k8s.io/v1\nkind: GatewayClass\nmetadata:\n  name: lab-gateway-class\nspec:\n  controllerName: example.com/gateway-controller\nEOF"
success_check:
  - kubectl get gatewayclass lab-gateway-class
success_contains:
  - lab-gateway-class
what_changed:
  - GatewayClass lab-gateway-class created
  - On the exam the GatewayClass is usually pre-created; here you practice making one
fallback_hint: controllerName points to the implementing controller. For lab purposes any string works.
```

```cka-step
step_id: p04-s03
title: create gateway listener
objective: Create a Gateway with an HTTP listener on port 80 in the two-tier namespace.
ready_weight: 3
commands:
  - "kubectl -n two-tier apply -f - <<EOF\napiVersion: gateway.networking.k8s.io/v1\nkind: Gateway\nmetadata:\n  name: two-tier-gw\n  namespace: two-tier\nspec:\n  gatewayClassName: lab-gateway-class\n  listeners:\n    - name: http\n      protocol: HTTP\n      port: 80\n      allowedRoutes:\n        namespaces:\n          from: Same\nEOF"
success_check:
  - kubectl -n two-tier get gateway two-tier-gw
success_contains:
  - two-tier-gw
what_changed:
  - Gateway two-tier-gw created with HTTP listener on port 80
  - allowedRoutes restricts HTTPRoutes to the same namespace
fallback_hint: gatewayClassName must match the GatewayClass name exactly.
```

```cka-step
step_id: p04-s04
title: create httproute
objective: Route external traffic to the frontend service via the gateway.
ready_weight: 4
commands:
  - "kubectl -n two-tier apply -f - <<EOF\napiVersion: gateway.networking.k8s.io/v1\nkind: HTTPRoute\nmetadata:\n  name: frontend-route\n  namespace: two-tier\nspec:\n  parentRefs:\n    - name: two-tier-gw\n      namespace: two-tier\n  rules:\n    - matches:\n        - path:\n            type: PathPrefix\n            value: /\n      backendRefs:\n        - name: frontend\n          port: 80\nEOF"
success_check:
  - kubectl -n two-tier get httproute frontend-route
success_contains:
  - frontend-route
what_changed:
  - HTTPRoute frontend-route attached to two-tier-gw
  - All traffic on / path forwarded to frontend service port 80
fallback_hint: parentRefs name and namespace must exactly match the Gateway.
```

```cka-step
step_id: p04-s05
title: inspect the route chain
objective: Verify the full Gateway -> HTTPRoute -> Service -> Pod chain.
ready_weight: 3
commands:
  - kubectl -n two-tier describe gateway two-tier-gw
  - kubectl -n two-tier describe httproute frontend-route
  - kubectl -n two-tier get svc frontend
  - kubectl -n two-tier get endpoints frontend
success_check:
  - kubectl -n two-tier get httproute frontend-route -o jsonpath='{.spec.rules[0].backendRefs[0].name}'
success_contains:
  - frontend
what_changed:
  - Nothing changed — you traced the full routing chain
  - Gateway -> HTTPRoute (parentRefs) -> Service (backendRefs) -> Pods (endpoints)
fallback_hint: describe shows status conditions — look for Accepted and ResolvedRefs.
```

```cka-step
step_id: p04-s06
title: add a second route rule
objective: Add a path-based rule that sends /api traffic to the backend service.
ready_weight: 4
commands:
  - "kubectl -n two-tier apply -f - <<EOF\napiVersion: gateway.networking.k8s.io/v1\nkind: HTTPRoute\nmetadata:\n  name: frontend-route\n  namespace: two-tier\nspec:\n  parentRefs:\n    - name: two-tier-gw\n      namespace: two-tier\n  rules:\n    - matches:\n        - path:\n            type: PathPrefix\n            value: /api\n      backendRefs:\n        - name: backend\n          port: 80\n    - matches:\n        - path:\n            type: PathPrefix\n            value: /\n      backendRefs:\n        - name: frontend\n          port: 80\nEOF"
success_check:
  - kubectl -n two-tier get httproute frontend-route -o jsonpath='{.spec.rules[0].matches[0].path.value}'
success_contains:
  - /api
what_changed:
  - HTTPRoute updated with two rules
  - /api -> backend service, / -> frontend service
  - More specific paths must come first
fallback_hint: Order matters — put /api before / so the prefix match does not swallow it.
```

```cka-step
step_id: p04-s07
title: ingress to gateway mental model
objective: Compare the old Ingress approach to the new Gateway API approach.
ready_weight: 1
commands:
  - kubectl explain ingress.spec.rules --recursive 2>/dev/null || echo "Ingress CRD available for reference"
  - kubectl explain httproute.spec.rules --recursive
success_check:
  - kubectl explain httproute.spec.rules 2>&1
success_contains:
  - backendRefs
what_changed:
  - Nothing changed — this is a conceptual comparison
  - "Ingress: one resource does class + listener + routes. Gateway API: separated into GatewayClass + Gateway + HTTPRoute"
fallback_hint: The exam may ask you to migrate Ingress rules to HTTPRoute. Map host/path rules to matches and backendRefs.
```

## Exam Trap to Practice

`HTTPRoute` is Accepted but no traffic flows. Root cause: `parentRefs` name or namespace does not match the Gateway exactly. Always double-check with `kubectl describe httproute`.

## Reflection Prompts

- Why did Gateway API split Ingress into three separate resources?
- What does `allowedRoutes.namespaces.from: Same` control?
- On the exam, if a GatewayClass already exists, what two resources do you need to create?

## What You Now Know

- Install Gateway API CRDs
- Create GatewayClass, Gateway (with listeners), and HTTPRoute (with parentRefs + backendRefs)
- Path-based routing with multiple rules
- The Ingress -> Gateway API mental model for exam migration tasks

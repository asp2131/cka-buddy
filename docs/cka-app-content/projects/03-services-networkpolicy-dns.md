# Project 03 - Services, NetworkPolicy, and DNS

id: p03
type: project
domains: networking,troubleshooting
difficulty: intermediate
timebox_min: 210

## Mission

Lock down the two-tier app with network policies. Only the frontend should reach the backend. Verify everything through DNS and connectivity tests.

## Context

The `two-tier` namespace has frontend (3 replicas) and backend (1 replica with persistent storage) from Projects 01-02. Right now, any pod in the cluster can reach the backend. Time to fix that.

## Concepts

- **ClusterIP Service**: internal-only stable IP. The default service type.
- **Endpoints**: the actual pod IPs behind a service. If empty, your selector is wrong.
- **NetworkPolicy**: firewall rules for pod-to-pod traffic. Without one, everything is allowed.
- **Default Deny**: a NetworkPolicy that blocks all ingress (or egress) to a namespace. You then add explicit allows.
- **DNS**: Kubernetes DNS resolves `<service>.<namespace>.svc.cluster.local` to ClusterIP.

## Deliverables

- Deep-dive into service selectors and endpoints.
- Apply a default-deny ingress policy to the `two-tier` namespace.
- Add an explicit allow policy for frontend -> backend traffic only.
- Verify that unauthorized pods are blocked.
- Debug DNS resolution across namespaces.

## Runnable Steps (Strict)

```cka-step
step_id: p03-s01
title: inspect service selectors and endpoints
objective: Understand how services find pods through label selectors.
ready_weight: 2
commands:
  - kubectl -n two-tier get svc backend -o yaml
  - kubectl -n two-tier get endpoints backend
  - kubectl -n two-tier get pods -l app=backend -o wide
success_check:
  - kubectl -n two-tier get endpoints backend
success_contains:
  - ":"
what_changed:
  - Nothing changed — you inspected the service -> endpoint -> pod chain
  - The endpoint IPs match the pod IPs (service selector matches pod labels)
fallback_hint: If endpoints show <none>, the service selector does not match any pod labels.
```

```cka-step
step_id: p03-s02
title: test current open access
objective: Prove that any pod can currently reach the backend (no policy yet).
ready_weight: 2
commands:
  - kubectl create namespace intruder
  - kubectl -n intruder run attacker --image=busybox --command -- sleep 3600
  - kubectl -n intruder wait --for=condition=ready pod/attacker --timeout=60s
  - kubectl -n intruder exec attacker -- wget -qO- --timeout=5 http://backend.two-tier.svc.cluster.local/get
success_check:
  - kubectl -n intruder exec attacker -- wget -qO- --timeout=5 http://backend.two-tier.svc.cluster.local/get 2>&1
success_contains:
  - origin
what_changed:
  - Namespace intruder created with an attacker pod
  - Attacker successfully reached backend — this is the problem we will fix
fallback_hint: Cross-namespace DNS uses <svc>.<namespace>.svc.cluster.local format.
```

```cka-step
step_id: p03-s03
title: apply default deny ingress
objective: Block all incoming traffic to pods in the two-tier namespace.
ready_weight: 3
commands:
  - "kubectl -n two-tier apply -f - <<EOF\napiVersion: networking.k8s.io/v1\nkind: NetworkPolicy\nmetadata:\n  name: default-deny-ingress\n  namespace: two-tier\nspec:\n  podSelector: {}\n  policyTypes:\n    - Ingress\nEOF"
  - kubectl -n two-tier get netpol
success_check:
  - kubectl -n two-tier get netpol default-deny-ingress
success_contains:
  - default-deny-ingress
what_changed:
  - All ingress traffic to two-tier namespace is now blocked
  - This includes frontend -> backend (we will fix this next)
fallback_hint: Empty podSelector {} means "all pods in this namespace".
```

```cka-step
step_id: p03-s04
title: verify attacker is blocked
objective: Confirm the attacker pod can no longer reach the backend.
ready_weight: 2
commands:
  - kubectl -n intruder exec attacker -- wget -qO- --timeout=3 http://backend.two-tier.svc.cluster.local/get 2>&1 || echo "BLOCKED"
success_check:
  - kubectl -n intruder exec attacker -- wget -qO- --timeout=3 http://backend.two-tier.svc.cluster.local/get 2>&1 || echo "BLOCKED"
success_contains:
  - BLOCKED
what_changed:
  - Nothing changed — verified that the default-deny policy is working
  - The attacker pod times out trying to reach backend
fallback_hint: A timeout means the traffic is being dropped by the network policy.
```

```cka-step
step_id: p03-s05
title: allow frontend to backend
objective: Create a policy that allows only frontend pods to reach backend on port 80.
ready_weight: 4
commands:
  - "kubectl -n two-tier apply -f - <<EOF\napiVersion: networking.k8s.io/v1\nkind: NetworkPolicy\nmetadata:\n  name: allow-frontend-to-backend\n  namespace: two-tier\nspec:\n  podSelector:\n    matchLabels:\n      app: backend\n  policyTypes:\n    - Ingress\n  ingress:\n    - from:\n        - podSelector:\n            matchLabels:\n              app: frontend\n      ports:\n        - protocol: TCP\n          port: 80\nEOF"
  - kubectl -n two-tier get netpol
success_check:
  - kubectl -n two-tier get netpol allow-frontend-to-backend
success_contains:
  - allow-frontend-to-backend
what_changed:
  - Ingress policy added allowing frontend -> backend on TCP/80
  - Attacker pod is still blocked (no matching label)
fallback_hint: podSelector on the policy selects the TARGET pods. ingress.from selects the SOURCE pods.
```

```cka-step
step_id: p03-s06
title: verify frontend access restored
objective: Confirm frontend can reach backend again but attacker still cannot.
ready_weight: 3
commands:
  - kubectl -n two-tier exec deploy/frontend -- curl -s --max-time 5 http://backend.two-tier.svc.cluster.local/get
  - kubectl -n intruder exec attacker -- wget -qO- --timeout=3 http://backend.two-tier.svc.cluster.local/get 2>&1 || echo "STILL BLOCKED"
success_check:
  - kubectl -n two-tier exec deploy/frontend -- curl -s --max-time 5 -o /dev/null -w '%{http_code}' http://backend.two-tier.svc.cluster.local/get
success_contains:
  - "200"
what_changed:
  - Frontend -> backend connectivity restored (policy allows it)
  - Attacker -> backend still blocked (no matching labels)
fallback_hint: If frontend is also blocked, check that frontend pods have label app=frontend.
```

```cka-step
step_id: p03-s07
title: dns deep dive
objective: Explore Kubernetes DNS resolution from inside a pod.
ready_weight: 2
commands:
  - kubectl -n two-tier exec deploy/frontend -- cat /etc/resolv.conf
  - kubectl -n two-tier run dns-debug --image=busybox:1.36 --rm -it --restart=Never -- nslookup backend.two-tier.svc.cluster.local
success_check:
  - kubectl -n two-tier run dns-check --image=busybox:1.36 --rm --restart=Never -- nslookup backend.two-tier.svc.cluster.local 2>&1
success_contains:
  - Address
what_changed:
  - Nothing changed — you inspected DNS configuration
  - resolv.conf shows the cluster DNS server and search domains
  - nslookup resolved the service name to its ClusterIP
fallback_hint: The search domain in resolv.conf is why short names like "backend" work within the same namespace.
```

```cka-step
step_id: p03-s08
title: clean up intruder namespace
objective: Remove the test namespace used for attack simulation.
ready_weight: 1
commands:
  - kubectl delete namespace intruder
success_check:
  - kubectl get namespace intruder 2>&1
success_contains:
  - NotFound
what_changed:
  - Intruder namespace and attacker pod deleted
  - two-tier namespace is now properly secured with network policies
fallback_hint: Namespace deletion removes all resources inside it.
```

## Exam Trap to Practice

Service selector labels do not match pod labels. Endpoints list is empty even though pods are running. Debug with `kubectl get endpoints` and compare `kubectl get pods --show-labels`.

## Reflection Prompts

- Why do we need a default-deny AND an explicit allow? Why not just the allow?
- What is the difference between podSelector on the policy itself vs in the ingress.from block?
- If you add a new deployment `monitoring` to two-tier, can it reach the backend? Why or why not?

## What You Now Know

- How services select pods via labels and populate endpoints
- Default-deny NetworkPolicy pattern (block all, then allow explicitly)
- Ingress policy with podSelector source filtering
- Kubernetes DNS resolution and the `<svc>.<ns>.svc.cluster.local` pattern
- Cross-namespace access control

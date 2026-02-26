# Project 08 - Troubleshooting Bootcamp

id: p08
type: project
domains: troubleshooting
difficulty: advanced
timebox_min: 300

## Mission

Run timed break/fix scenarios across the two-tier app and cluster infrastructure. This is the highest-weight CKA domain (30%) and the one that separates pass from fail.

## Context

You have built, configured, secured, and administered the two-tier app across Projects 00-07. Now you will intentionally break things and fix them under time pressure, using a systematic diagnostic flow.

## The 60-Second Diagnostic Flow

Use this every single time, in this order:

1. `kubectl get pods -A` — find what is broken (CrashLoop, Pending, Error)
2. `kubectl describe pod <pod> -n <ns>` — read Events section at bottom
3. `kubectl logs <pod> -n <ns> --previous` — read container error output
4. If node issue: `systemctl status kubelet` + `journalctl -u kubelet`

Do NOT skip steps or jump to guessing. This flow catches 80% of issues in under 60 seconds.

## Drill Rules

- 8 scenarios, 8 minutes each, hard stop per scenario.
- Write a one-line root cause statement after each fix.
- If stuck at 4 minutes, read the hint. If stuck at 6 minutes, read the solution and move on.
- Target: >= 6/8 solved within time across two separate sessions before moving to mock exam.

## Scenario 1: CrashLoop from Bad Config Key

```cka-step
step_id: p08-s01
title: inject crashloop bug
objective: Break the backend by misspelling a ConfigMap key reference.
ready_weight: 1
commands:
  - kubectl -n two-tier create configmap broken-config --from-literal=WRNG_KEY=bad-value
  - kubectl -n two-tier set env deployment/backend --from=configmap/broken-config
  - kubectl -n two-tier set env deployment/backend CONFIG_MODE=required
success_check:
  - kubectl -n two-tier get pods -l app=backend
success_contains:
  - ""
what_changed:
  - Backend injected with a misspelled config key
  - This simulates a real-world config drift bug
fallback_hint: The bug is now live. Use the diagnostic flow to find and fix it.
```

```cka-step
step_id: p08-s02
title: diagnose and fix crashloop
objective: Find the config key mismatch and restore the backend to Running.
ready_weight: 4
commands:
  - kubectl -n two-tier describe pod -l app=backend
  - kubectl -n two-tier logs deploy/backend --previous
  - kubectl -n two-tier get configmap broken-config -o yaml
success_check:
  - kubectl -n two-tier get pods -l app=backend -o jsonpath='{.items[0].status.phase}'
success_contains:
  - Running
what_changed:
  - Identified misspelled config key
  - Fixed by patching ConfigMap or deployment env reference
  - Backend restored to Running
fallback_hint: Compare the env var name the app expects with the ConfigMap key names. Fix the mismatch and rollout restart.
```

## Scenario 2: Service Selector Mismatch

```cka-step
step_id: p08-s03
title: inject selector mismatch bug
objective: Break the frontend service by changing its selector to a non-existent label.
ready_weight: 1
commands:
  - kubectl -n two-tier patch svc frontend -p '{"spec":{"selector":{"app":"frontnd"}}}'
success_check:
  - kubectl -n two-tier get endpoints frontend
success_contains:
  - ""
what_changed:
  - Frontend service selector changed to app=frontnd (typo)
  - Endpoints are now empty — no pods match
fallback_hint: The bug is live. Start with kubectl get endpoints.
```

```cka-step
step_id: p08-s04
title: diagnose and fix selector mismatch
objective: Find the empty endpoints and fix the service selector.
ready_weight: 3
commands:
  - kubectl -n two-tier get endpoints frontend
  - kubectl -n two-tier get pods --show-labels
  - kubectl -n two-tier get svc frontend -o yaml
success_check:
  - kubectl -n two-tier get endpoints frontend -o jsonpath='{.subsets[0].addresses[0].ip}'
success_contains:
  - ""
what_changed:
  - Empty endpoints identified
  - Selector fixed to match actual pod labels
  - Service routing restored
fallback_hint: Compare svc selector labels with pod labels. Fix the typo with kubectl patch svc frontend -p '{"spec":{"selector":{"app":"frontend"}}}'
```

## Scenario 3: NetworkPolicy Blocking All Traffic

```cka-step
step_id: p08-s05
title: inject netpol lockout bug
objective: Apply an overly restrictive NetworkPolicy that blocks frontend from reaching backend.
ready_weight: 1
commands:
  - kubectl -n two-tier delete netpol allow-frontend-to-backend 2>/dev/null; true
  - "kubectl -n two-tier apply -f - <<EOF\napiVersion: networking.k8s.io/v1\nkind: NetworkPolicy\nmetadata:\n  name: block-all\n  namespace: two-tier\nspec:\n  podSelector:\n    matchLabels:\n      app: backend\n  policyTypes:\n    - Ingress\n  ingress: []\nEOF"
success_check:
  - kubectl -n two-tier get netpol
success_contains:
  - block-all
what_changed:
  - All ingress to backend is now blocked (empty ingress list = deny all to selected pods)
  - Frontend can no longer reach backend
fallback_hint: Diagnose with a curl from frontend to backend.
```

```cka-step
step_id: p08-s06
title: diagnose and fix netpol lockout
objective: Identify the blocking policy and restore frontend access.
ready_weight: 4
commands:
  - kubectl -n two-tier exec deploy/frontend -- curl -s --max-time 3 http://backend/get 2>&1 || echo "TIMEOUT"
  - kubectl -n two-tier get netpol -o yaml
  - kubectl -n two-tier get pods --show-labels
success_check:
  - kubectl -n two-tier exec deploy/frontend -- curl -s --max-time 5 -o /dev/null -w '%{http_code}' http://backend/get
success_contains:
  - "200"
what_changed:
  - Identified empty ingress list as the cause
  - Fixed by adding proper ingress rule or replacing the policy
  - Frontend -> backend connectivity restored
fallback_hint: Delete block-all and re-apply the allow-frontend-to-backend policy from Project 03.
```

## Scenario 4: RBAC Forbidden

```cka-step
step_id: p08-s07
title: inject rbac bug
objective: Break the deploy-bot permissions by deleting the RoleBinding.
ready_weight: 1
commands:
  - kubectl -n two-tier delete rolebinding deploy-bot-binding
success_check:
  - kubectl auth can-i get pods -n two-tier --as=system:serviceaccount:two-tier:deploy-bot
success_contains:
  - "no"
what_changed:
  - RoleBinding deleted — deploy-bot now has no permissions
  - Role still exists but is not bound to anything
fallback_hint: The SA exists, the Role exists, but the binding is missing.
```

```cka-step
step_id: p08-s08
title: diagnose and fix rbac
objective: Find the missing binding and restore deploy-bot access.
ready_weight: 3
commands:
  - kubectl auth can-i get pods -n two-tier --as=system:serviceaccount:two-tier:deploy-bot
  - kubectl -n two-tier get role,rolebinding
  - kubectl -n two-tier get serviceaccount deploy-bot
success_check:
  - kubectl auth can-i get pods -n two-tier --as=system:serviceaccount:two-tier:deploy-bot
success_contains:
  - "yes"
what_changed:
  - Missing RoleBinding identified (Role exists but not bound)
  - Recreated binding connecting deploy-bot SA to deploy-bot-role
  - Permissions restored
fallback_hint: kubectl create rolebinding deploy-bot-binding --role=deploy-bot-role --serviceaccount=two-tier:deploy-bot -n two-tier
```

## Scenario 5: Pod Stuck Pending (Resource Limits)

```cka-step
step_id: p08-s09
title: inject pending pod bug
objective: Create a pod requesting more resources than the node has available.
ready_weight: 1
commands:
  - kubectl -n two-tier run resource-hog --image=nginx --overrides='{"spec":{"containers":[{"name":"resource-hog","image":"nginx","resources":{"requests":{"cpu":"100","memory":"512Gi"}}}]}}'
success_check:
  - kubectl -n two-tier get pod resource-hog
success_contains:
  - Pending
what_changed:
  - Pod resource-hog requesting 100 CPUs and 512Gi memory
  - No node can satisfy this — pod will be Pending forever
fallback_hint: Use the diagnostic flow starting with describe pod.
```

```cka-step
step_id: p08-s10
title: diagnose and fix pending pod
objective: Find the scheduling failure and fix the resource request.
ready_weight: 3
commands:
  - kubectl -n two-tier describe pod resource-hog
  - kubectl -n two-tier get pod resource-hog -o jsonpath='{.spec.containers[0].resources.requests}'
success_check:
  - kubectl -n two-tier get pod resource-hog 2>&1
success_contains:
  - NotFound
what_changed:
  - describe Events showed "Insufficient cpu" / "Insufficient memory"
  - Deleted the impossible pod (or recreated with sane limits)
  - On the exam, fix the YAML and re-apply rather than delete+recreate
fallback_hint: Delete the pod and recreate with reasonable requests — kubectl delete pod resource-hog -n two-tier
```

## Scenario 6: Wrong Container Image

```cka-step
step_id: p08-s11
title: inject imagepull bug
objective: Set a non-existent image on the frontend deployment.
ready_weight: 1
commands:
  - kubectl -n two-tier set image deployment/frontend nginx=nginx:doesnotexist999
success_check:
  - kubectl -n two-tier get pods -l app=frontend
success_contains:
  - ImagePull
what_changed:
  - Frontend pods trying to pull nginx:doesnotexist999
  - New pods stuck in ImagePullBackOff, old pods may still be running (rolling update)
fallback_hint: Use describe to see the image pull error in Events.
```

```cka-step
step_id: p08-s12
title: diagnose and fix imagepull
objective: Identify the bad image and rollback.
ready_weight: 3
commands:
  - kubectl -n two-tier describe pod -l app=frontend | tail -20
  - kubectl -n two-tier rollout history deployment/frontend
  - kubectl -n two-tier rollout undo deployment/frontend
  - kubectl -n two-tier rollout status deployment/frontend
success_check:
  - kubectl -n two-tier get pods -l app=frontend -o jsonpath='{.items[0].status.phase}'
success_contains:
  - Running
what_changed:
  - Identified ImagePullBackOff from non-existent tag
  - Rolled back to previous working revision
  - All frontend pods Running again
fallback_hint: rollout undo is the fastest fix. Alternatively use set image to correct the tag.
```

## Scenario 7: Node NotReady Simulation

```cka-step
step_id: p08-s13
title: understand node troubleshooting
objective: Review the diagnostic flow for node-level issues.
ready_weight: 2
commands:
  - kubectl get nodes
  - kubectl describe node $(kubectl get nodes -o name | head -1 | cut -d/ -f2) | grep -A5 Conditions
  - "echo 'On the exam for NotReady nodes:'"
  - "echo '1. SSH to the node'"
  - "echo '2. sudo systemctl status kubelet'"
  - "echo '3. sudo journalctl -u kubelet --no-pager | tail -40'"
  - "echo '4. Check /etc/cni/net.d/ for CNI config'"
  - "echo '5. sudo systemctl restart kubelet'"
success_check:
  - kubectl get nodes
success_contains:
  - Ready
what_changed:
  - Nothing changed — reviewed node diagnostic flow
  - "Key checks: kubelet service, journalctl logs, CNI config, cert expiry"
fallback_hint: On the exam you will SSH to the node. Always start with systemctl status kubelet.
```

## Scenario 8: Deployment Not Scaling

```cka-step
step_id: p08-s14
title: inject scaling bug
objective: Set maxUnavailable and maxSurge to 0 to deadlock a rollout.
ready_weight: 1
commands:
  - kubectl -n two-tier patch deployment frontend -p '{"spec":{"strategy":{"rollingUpdate":{"maxSurge":0,"maxUnavailable":0}}}}'
  - kubectl -n two-tier set image deployment/frontend nginx=nginx:1.25-alpine
success_check:
  - kubectl -n two-tier rollout status deployment/frontend --timeout=10s 2>&1
success_contains:
  - ""
what_changed:
  - Rollout deadlocked — cannot create new pods (maxSurge=0) and cannot remove old pods (maxUnavailable=0)
  - Rollout will hang indefinitely
fallback_hint: Use describe deployment and check the strategy section.
```

```cka-step
step_id: p08-s15
title: diagnose and fix deadlocked rollout
objective: Identify the impossible strategy and fix it.
ready_weight: 3
commands:
  - kubectl -n two-tier describe deployment frontend | grep -A5 Strategy
  - kubectl -n two-tier patch deployment frontend -p '{"spec":{"strategy":{"rollingUpdate":{"maxSurge":1,"maxUnavailable":1}}}}'
  - kubectl -n two-tier rollout status deployment/frontend --timeout=60s
success_check:
  - kubectl -n two-tier get deploy frontend -o jsonpath='{.status.updatedReplicas}'
success_contains:
  - "3"
what_changed:
  - Identified maxSurge=0 + maxUnavailable=0 deadlock
  - Fixed strategy to allow rolling updates
  - Rollout completed successfully
fallback_hint: At least one of maxSurge or maxUnavailable must be > 0 for a rollout to progress.
```

## Promotion Gate

Only move to the mock exam (`exam/mock-exam-blueprint.md`) when you can:
- Solve >= 6/8 scenarios within time in a single session
- Write a clear one-line root cause for each fix
- Complete the full diagnostic flow without skipping steps

## Reflection Prompts

- Which scenario types were hardest? Those are your weak domains to review.
- Did the 60-second diagnostic flow catch the issue before you started guessing?
- What patterns do you see across the bugs? (labels, selectors, typos, resource limits)

## What You Now Know

- Systematic troubleshooting flow for any Kubernetes failure
- CrashLoopBackOff: config key mismatches, missing env vars
- Empty endpoints: service selector mismatches
- NetworkPolicy lockouts: empty ingress vs missing allow rules
- RBAC failures: missing bindings, wrong SA references
- Pending pods: resource requests exceeding node capacity
- ImagePullBackOff: wrong tags, rollback as fast fix
- Deadlocked rollouts: impossible strategy parameters

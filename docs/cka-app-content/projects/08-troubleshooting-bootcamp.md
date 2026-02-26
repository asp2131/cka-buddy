# Project 08 - Troubleshooting Bootcamp

id: p08
type: project
domains: troubleshooting
difficulty: advanced
timebox_min: 300

## Mission

Run timed break/fix reps across pods, services, RBAC, kubelet, and control plane static pods.

## Drill Rules

- 8 scenarios, 8 minutes each, hard stop per scenario.
- Use this 60-second start flow every time:
  1) `kubectl get pods -A`
  2) `kubectl describe pod ...`
  3) `kubectl logs ... --previous`
  4) if node issue: `systemctl` + `journalctl`

## Success Metric

- >= 6/8 scenarios solved within time.
- Root cause statement written for each fix.

## Promotion Gate

Only move to full mock exam when this project passes twice in separate sessions.

## Runnable Steps (Strict)

```cka-step
step_id: p08-s01
title: 60-second diagnostic open
objective: Execute the first-pass diagnostic flow on a broken workload.
ready_weight: 3
commands:
  - kubectl get pods -A
  - kubectl describe pod <pod-name> -n <ns>
  - kubectl logs <pod-name> -n <ns> --previous
success_check:
  - kubectl get pods -A
success_contains:
  - Running
what_changed:
  - Cluster health snapshot taken
  - Root-cause hypothesis formed
fallback_hint: Replace placeholders with actual failing pod and namespace.
```

```cka-step
step_id: p08-s02
title: node-level recovery drill
objective: Confirm kubelet health and recover NotReady node scenarios.
ready_weight: 4
commands:
  - ssh <node>
  - sudo -i
  - systemctl status kubelet
  - journalctl -u kubelet --no-pager | tail -n 40
success_check:
  - kubectl get nodes
success_contains:
  - Ready
what_changed:
  - Node issue diagnosed from kubelet logs
  - Cluster node restored to Ready
fallback_hint: Check /etc/cni/net.d when kubelet logs mention CNI load failures.
```

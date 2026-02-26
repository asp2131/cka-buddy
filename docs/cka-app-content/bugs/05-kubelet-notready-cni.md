# Bug 05 - Node NotReady from CNI Misconfiguration

id: b05
type: bug
domains: troubleshooting,architecture
difficulty: advanced
timebox_min: 20

## Broken State

Worker node is `NotReady`; kubelet logs show CNI config load errors.

## Objective

Recover node readiness and schedule a test pod successfully.

## Expected Workflow

- SSH node -> `systemctl status kubelet`.
- `journalctl -u kubelet` for failure details.
- Inspect `/etc/cni/net.d/` and repair config.

## Runnable Steps (Strict)

```cka-step
step_id: b05-s01
title: recover notready node
objective: Diagnose kubelet CNI errors and return node to Ready.
ready_weight: 3
commands:
  - ssh <node>
  - sudo -i
  - systemctl status kubelet
  - journalctl -u kubelet --no-pager | tail -n 50
success_check:
  - kubectl get nodes
success_contains:
  - Ready
what_changed:
  - CNI issue diagnosed from kubelet logs
  - Worker node recovered to Ready
fallback_hint: Verify valid CNI config files under /etc/cni/net.d.
```

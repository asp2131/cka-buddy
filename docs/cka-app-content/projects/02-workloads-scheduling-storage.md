# Project 02 - Workloads, Scheduling, and Storage

id: p02
type: project
domains: workloads,storage
difficulty: intermediate
timebox_min: 210

## Mission

Control pod placement with taints/tolerations + affinity and mount persistent storage correctly.

## Deliverables

- Label one node for stateful workloads.
- Taint a second node (`NoSchedule`) and deploy a tolerant pod.
- Create PV + PVC and mount into a deployment.
- Create a StorageClass and PVC using `WaitForFirstConsumer`.

## Verification

- `kubectl get pods -o wide`
- `kubectl describe node <node>`
- `kubectl get pv,pvc`

## Exam Trap to Practice

PVC remains `Pending`. Debug capacity, access mode, and `storageClassName` mismatch.

## Runnable Steps (Strict)

```cka-step
step_id: p02-s01
title: taints and tolerations drill
objective: Taint one node and verify scheduling behavior.
ready_weight: 2
commands:
  - kubectl taint nodes <node-name> lab=blocked:NoSchedule
success_check:
  - kubectl describe node <node-name>
success_contains:
  - lab=blocked:NoSchedule
what_changed:
  - Node taint applied
  - Scheduling guard introduced
fallback_hint: Replace <node-name> with an actual node from kubectl get nodes.
```

```cka-step
step_id: p02-s02
title: pv pvc binding
objective: Create PV and PVC and confirm Bound status.
ready_weight: 3
commands:
  - kubectl apply -f pv.yaml
  - kubectl apply -f pvc.yaml
success_check:
  - kubectl get pvc -A
success_contains:
  - Bound
what_changed:
  - PersistentVolume available
  - PersistentVolumeClaim bound
fallback_hint: AccessModes and storageClassName must match exactly.
```

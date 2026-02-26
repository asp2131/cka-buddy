# Project 07 - Day-2 Admin: kubeadm, etcd, RBAC

id: p07
type: project
domains: architecture,troubleshooting
difficulty: advanced
timebox_min: 240

## Mission

Execute critical admin workflows: permission debugging, node maintenance, and control-plane safety operations.

## Deliverables

- Debug a failing ServiceAccount action with `kubectl auth can-i`.
- Create least-privilege Role + RoleBinding for namespace operations.
- Simulate control-plane maintenance: drain/uncordon flow.
- Perform etcd snapshot save and restore to an alternate data dir.

## Verification

- `kubectl auth can-i --as=system:serviceaccount:<ns>:<sa> <verb> <resource>`
- `kubectl get role,rolebinding -n <ns>`
- `ETCDCTL_API=3 etcdctl snapshot status <snapshot-file> -w table`

## Exam Trap to Practice

Incorrect etcd cert/key paths from memory instead of reading `etcd.yaml`.

## Runnable Steps (Strict)

```cka-step
step_id: p07-s01
title: rbac can-i debug
objective: Validate serviceaccount permission path with can-i and bindings.
ready_weight: 3
commands:
  - kubectl auth can-i --as=system:serviceaccount:team1:builder get pods -n team1
  - kubectl get role,rolebinding -n team1
success_check:
  - kubectl auth can-i --as=system:serviceaccount:team1:builder get pods -n team1
success_contains:
  - yes
what_changed:
  - Access path validated
  - Least-privilege binding confirmed
fallback_hint: Verify RoleBinding subject namespace matches serviceaccount namespace.
```

```cka-step
step_id: p07-s02
title: etcd snapshot
objective: Save etcd snapshot using cert paths from static pod manifest.
ready_weight: 4
commands:
  - ETCDCTL_API=3 etcdctl snapshot save /tmp/cka-snap.db --endpoints=https://127.0.0.1:2379 --cacert=<ca> --cert=<cert> --key=<key>
success_check:
  - ETCDCTL_API=3 etcdctl snapshot status /tmp/cka-snap.db -w table
success_contains:
  - HASH
what_changed:
  - Snapshot file created
  - Snapshot metadata verified
fallback_hint: Pull cert paths from /etc/kubernetes/manifests/etcd.yaml, not memory.
```

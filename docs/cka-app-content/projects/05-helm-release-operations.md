# Project 05 - Helm Release Operations

id: p05
type: project
domains: architecture,troubleshooting
difficulty: intermediate
timebox_min: 180

## Mission

Operate Helm releases the way CKA expects: install, inspect, upgrade, rollback.

## Deliverables

- Add a chart repo and search for a web chart.
- Install release in non-default namespace with custom values.
- Upgrade image tag or replica value.
- Roll back to prior revision and confirm service health.

## Verification

- `helm list -n <ns>`
- `helm history <release> -n <ns>`
- `kubectl get deploy,svc -n <ns>`

## Exam Trap to Practice

Release accidentally deployed to `default`; re-target with namespace discipline.

## Runnable Steps (Strict)

```cka-step
step_id: p05-s01
title: install helm release in target ns
objective: Install chart into non-default namespace with explicit flags.
ready_weight: 3
commands:
  - helm repo add bitnami https://charts.bitnami.com/bitnami
  - helm install web bitnami/nginx -n cka-lab --create-namespace
success_check:
  - helm list -n cka-lab
success_contains:
  - web
what_changed:
  - Helm repo added
  - Release deployed in cka-lab
fallback_hint: Always set -n for helm install/list/history.
```

```cka-step
step_id: p05-s02
title: upgrade and rollback
objective: Upgrade values then rollback to previous revision.
ready_weight: 3
commands:
  - helm upgrade web bitnami/nginx -n cka-lab --set replicaCount=2
  - helm rollback web 1 -n cka-lab
success_check:
  - helm history web -n cka-lab
success_contains:
  - superseded
what_changed:
  - Upgrade executed
  - Rollback validated
fallback_hint: Use helm history to identify target revision before rollback.
```

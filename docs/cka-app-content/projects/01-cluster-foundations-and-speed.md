# Project 01 - Cluster Foundations and Speed

id: p01
type: project
domains: architecture,workloads
difficulty: beginner
timebox_min: 180

## Mission

Build speed with imperative `kubectl`, dry-run YAML generation, ConfigMaps, and Secrets.

## Deliverables

- Create a namespace `p01-lab`.
- Deploy `web` with 3 replicas and expose as `ClusterIP`.
- Create one ConfigMap + one Secret and inject both into pods.
- Save generated manifests to files and re-apply them idempotently.

## Required Commands

- `kubectl create deployment ... --dry-run=client -o yaml`
- `kubectl create configmap ... --from-literal=...`
- `kubectl create secret generic ... --from-literal=...`

## Verification

- `kubectl get all -n p01-lab`
- `kubectl describe pod -n p01-lab <pod-name>`
- `kubectl logs -n p01-lab <pod-name>`

## Reflection Prompts

- What command saved you the most time?
- Which fields were easier to edit in YAML vs imperative flags?

## Runnable Steps (Strict)

```cka-step
step_id: p01-s01
title: bootstrap namespace and deployment
objective: Create p01-lab and deploy web with 3 replicas.
ready_weight: 2
commands:
  - kubectl create namespace p01-lab
  - kubectl -n p01-lab create deployment web --image=nginx --replicas=3
success_check:
  - kubectl get deploy -n p01-lab web -o jsonpath='{.spec.replicas}'
success_contains:
  - "3"
what_changed:
  - Namespace p01-lab created
  - Deployment web scaled to 3 replicas
fallback_hint: Use -n p01-lab on every command.
```

```cka-step
step_id: p01-s02
title: expose and inject config
objective: Expose web service and inject configmap/secret values.
ready_weight: 2
commands:
  - kubectl -n p01-lab expose deployment web --port=80 --target-port=80 --type=ClusterIP
  - kubectl -n p01-lab create configmap appcfg --from-literal=CONFIG_MODE=lab
  - kubectl -n p01-lab create secret generic appsec --from-literal=PASSWORD=s3cret
success_check:
  - kubectl get svc -n p01-lab web
success_contains:
  - ClusterIP
what_changed:
  - Service web created
  - Config and secret objects ready
fallback_hint: Verify service name is exactly web.
```

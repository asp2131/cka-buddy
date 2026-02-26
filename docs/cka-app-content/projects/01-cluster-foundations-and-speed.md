# Project 01 - Cluster Foundations and Speed

id: p01
type: project
domains: architecture,workloads
difficulty: beginner
timebox_min: 180

## Mission

Configure the two-tier app you deployed in Phase 1 with ConfigMaps, Secrets, replicas, and resource limits. Build speed with imperative kubectl and the dry-run workflow under light time pressure.

## Context

You already have the `two-tier` namespace with `frontend` (nginx) and `backend` (httpbin) deployments and services from Project 00b. Now you will make them production-like.

## Deliverables

- Inject configuration into both deployments via ConfigMap and Secret.
- Scale frontend to 3 replicas and set resource requests/limits.
- Save all generated manifests to files and re-apply idempotently.
- Complete all steps within the timebox to start building exam speed.

## Runnable Steps (Strict)

```cka-step
step_id: p01-s01
title: verify your starting point
objective: Confirm the two-tier app from Phase 1 is still running.
ready_weight: 1
commands:
  - kubectl get deploy,svc -n two-tier
success_check:
  - kubectl get deploy frontend -n two-tier -o jsonpath='{.status.availableReplicas}'
success_contains:
  - "2"
what_changed:
  - Nothing changed — verified starting state
  - frontend (2 replicas) and backend (1 replica) are running
fallback_hint: If the two-tier namespace is missing, go back and complete Project 00b first.
```

```cka-step
step_id: p01-s02
title: create app configmap
objective: Create a ConfigMap with shared app settings and inspect it.
ready_weight: 2
commands:
  - kubectl -n two-tier create configmap app-config --from-literal=APP_ENV=staging --from-literal=LOG_LEVEL=info --from-literal=BACKEND_URL=http://backend.two-tier.svc.cluster.local
  - kubectl -n two-tier get configmap app-config -o yaml
success_check:
  - kubectl -n two-tier get configmap app-config -o jsonpath='{.data.APP_ENV}'
success_contains:
  - staging
what_changed:
  - ConfigMap app-config created with 3 keys
  - BACKEND_URL points frontend to the backend service via DNS
fallback_hint: Use --from-literal for each key=value pair.
```

```cka-step
step_id: p01-s03
title: create app secret
objective: Create a Secret with sensitive credentials.
ready_weight: 2
commands:
  - kubectl -n two-tier create secret generic app-secret --from-literal=DB_PASSWORD=exam-pass-42 --from-literal=API_KEY=cka-secret-key
  - kubectl -n two-tier get secret app-secret -o jsonpath='{.data.DB_PASSWORD}' | base64 -d
success_check:
  - kubectl -n two-tier get secret app-secret
success_contains:
  - app-secret
what_changed:
  - Secret app-secret created with 2 keys
  - Values are base64-encoded at rest
fallback_hint: Secrets store data base64-encoded. Use base64 -d to decode and verify.
```

```cka-step
step_id: p01-s04
title: inject configmap into frontend
objective: Mount the ConfigMap as environment variables in the frontend deployment.
ready_weight: 3
commands:
  - kubectl -n two-tier set env deployment/frontend --from=configmap/app-config
  - kubectl -n two-tier rollout status deployment/frontend
success_check:
  - kubectl -n two-tier exec deploy/frontend -- env
success_contains:
  - APP_ENV=staging
what_changed:
  - Frontend pods restarted with ConfigMap env vars injected
  - Rollout completed with zero downtime (rolling update)
fallback_hint: set env --from=configmap/<name> injects all keys as env vars.
```

```cka-step
step_id: p01-s05
title: inject secret into backend
objective: Mount the Secret as environment variables in the backend deployment.
ready_weight: 3
commands:
  - kubectl -n two-tier set env deployment/backend --from=secret/app-secret
  - kubectl -n two-tier rollout status deployment/backend
success_check:
  - kubectl -n two-tier exec deploy/backend -- env
success_contains:
  - DB_PASSWORD
what_changed:
  - Backend pods restarted with Secret env vars injected
  - Secret values are available as plaintext inside the container
fallback_hint: set env --from=secret/<name> works the same as configmap injection.
```

```cka-step
step_id: p01-s06
title: scale frontend and set resources
objective: Scale frontend to 3 replicas and add resource requests/limits via a patch.
ready_weight: 3
commands:
  - kubectl -n two-tier scale deployment/frontend --replicas=3
  - kubectl -n two-tier set resources deployment/frontend --requests=cpu=50m,memory=64Mi --limits=cpu=200m,memory=128Mi
  - kubectl -n two-tier rollout status deployment/frontend
success_check:
  - kubectl -n two-tier get deploy frontend -o jsonpath='{.spec.replicas}'
success_contains:
  - "3"
what_changed:
  - Frontend scaled to 3 replicas
  - Resource requests and limits set on all frontend pods
fallback_hint: Use kubectl describe deploy frontend to verify resource fields in the pod template.
```

```cka-step
step_id: p01-s07
title: export manifests with dry-run
objective: Save current state as YAML files you could re-apply from scratch.
ready_weight: 2
commands:
  - kubectl -n two-tier get deployment frontend -o yaml > /tmp/frontend-deploy.yaml
  - kubectl -n two-tier get deployment backend -o yaml > /tmp/backend-deploy.yaml
  - kubectl -n two-tier get configmap app-config -o yaml > /tmp/app-config.yaml
  - kubectl -n two-tier get secret app-secret -o yaml > /tmp/app-secret.yaml
success_check:
  - cat /tmp/frontend-deploy.yaml
success_contains:
  - "kind: Deployment"
what_changed:
  - Four YAML manifests exported to /tmp
  - These can recreate the app from scratch with kubectl apply -f
fallback_hint: Use -o yaml to export any resource. Remove status/metadata.resourceVersion for clean re-apply.
```

```cka-step
step_id: p01-s08
title: idempotent re-apply
objective: Apply the exported manifests to prove idempotency.
ready_weight: 2
commands:
  - kubectl apply -f /tmp/app-config.yaml
  - kubectl apply -f /tmp/frontend-deploy.yaml
success_check:
  - kubectl -n two-tier get deploy frontend -o jsonpath='{.spec.replicas}'
success_contains:
  - "3"
what_changed:
  - Resources unchanged — apply is idempotent
  - This proves your exported YAML is valid and reusable
fallback_hint: '"configured" or "unchanged" in output both mean success.'
```

## Reflection Prompts

- What is the difference between a ConfigMap and a Secret in practice?
- Why does `kubectl set env` trigger a rolling update?
- What would happen if you applied the exported YAML to a different namespace?

## What You Now Know

- Create and inject ConfigMaps and Secrets into deployments
- Scale replicas and set resource requests/limits
- Export live resources as YAML and re-apply idempotently
- The rolling update mechanism when pod template changes

# Project 06 - Kustomize Overlays for Live Patching

id: p06
type: project
domains: architecture,workloads
difficulty: intermediate
timebox_min: 180

## Mission

Patch base manifests via overlays (replicas, image, env vars) and deploy with `kubectl -k`.

## Deliverables

- Build `base/` with deployment + service.
- Build `overlays/dev` and `overlays/prod`.
- Patch replica count and container image per overlay.
- Add env var patch in one overlay only.

## Verification

- `kubectl kustomize overlays/dev`
- `kubectl apply -k overlays/prod`
- `kubectl get deploy -n <ns> -o yaml`

## Exam Trap to Practice

Patch target selector misses kind/name; overlay applies but no changes appear.

## Runnable Steps (Strict)

```cka-step
step_id: p06-s01
title: preview overlay render
objective: Render overlay output and confirm patched fields before apply.
ready_weight: 2
commands:
  - kubectl kustomize overlays/dev
success_check:
  - kubectl kustomize overlays/dev
success_contains:
  - replicas
what_changed:
  - Overlay render inspected
  - Patch output validated
fallback_hint: Ensure kustomization.yaml includes resources and patches entries.
```

```cka-step
step_id: p06-s02
title: apply prod overlay
objective: Apply production overlay and verify rollout changes.
ready_weight: 3
commands:
  - kubectl apply -k overlays/prod
  - kubectl get deploy -n <ns>
success_check:
  - kubectl get deploy -n <ns> -o wide
success_contains:
  - "2/2"
what_changed:
  - Prod overlay applied
  - Deployment revision updated
fallback_hint: Replace <ns> with deployment namespace used by overlay manifests.
```

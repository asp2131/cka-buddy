# Project 06 - Kustomize Overlays for Environment Management

id: p06
type: project
domains: architecture,workloads
difficulty: intermediate
timebox_min: 180

## Mission

Manage dev and prod variants of the two-tier app using Kustomize overlays. No Helm templates, no duplication — just patches on a shared base.

## Context

You have been deploying the two-tier app with imperative commands and raw YAML. Kustomize lets you maintain one set of base manifests and patch them per environment. The CKA exam tests `kubectl kustomize` and `kubectl apply -k`.

## Concepts

- **Base**: the shared, unmodified manifests. A `kustomization.yaml` lists which resources are in the base.
- **Overlay**: a directory that references a base and applies patches on top. Each overlay = one environment.
- **Patch**: a partial YAML that modifies specific fields. Can be strategic merge or JSON patch.
- **kustomization.yaml**: the control file in each base/overlay directory. Lists resources, patches, and common labels/annotations.

## Deliverables

- Extract the two-tier app into a Kustomize base directory.
- Create dev and prod overlays with different replicas, images, and env vars.
- Preview rendered output with `kubectl kustomize`.
- Apply overlays to separate namespaces.

## Runnable Steps (Strict)

```cka-step
step_id: p06-s01
title: create base manifests
objective: Set up the Kustomize base directory with deployment and service manifests.
ready_weight: 3
commands:
  - mkdir -p /tmp/two-tier-kustomize/base
  - "kubectl -n two-tier create deployment frontend --image=nginx --replicas=1 --dry-run=client -o yaml > /tmp/two-tier-kustomize/base/frontend-deploy.yaml"
  - "kubectl -n two-tier create deployment backend --image=kennethreitz/httpbin --replicas=1 --dry-run=client -o yaml > /tmp/two-tier-kustomize/base/backend-deploy.yaml"
  - "cat > /tmp/two-tier-kustomize/base/frontend-svc.yaml <<EOF\napiVersion: v1\nkind: Service\nmetadata:\n  name: frontend\nspec:\n  selector:\n    app: frontend\n  ports:\n    - port: 80\n      targetPort: 80\nEOF"
  - "cat > /tmp/two-tier-kustomize/base/backend-svc.yaml <<EOF\napiVersion: v1\nkind: Service\nmetadata:\n  name: backend\nspec:\n  selector:\n    app: backend\n  ports:\n    - port: 80\n      targetPort: 80\nEOF"
success_check:
  - ls /tmp/two-tier-kustomize/base/
success_contains:
  - frontend-deploy.yaml
what_changed:
  - Base manifests created for frontend and backend (deploys + services)
  - These are the shared starting point for all environments
fallback_hint: dry-run generates clean YAML without cluster-specific metadata.
```

```cka-step
step_id: p06-s02
title: create base kustomization
objective: Write the kustomization.yaml that lists all base resources.
ready_weight: 2
commands:
  - "cat > /tmp/two-tier-kustomize/base/kustomization.yaml <<EOF\napiVersion: kustomize.config.k8s.io/v1beta1\nkind: Kustomization\nresources:\n  - frontend-deploy.yaml\n  - backend-deploy.yaml\n  - frontend-svc.yaml\n  - backend-svc.yaml\nEOF"
  - kubectl kustomize /tmp/two-tier-kustomize/base/
success_check:
  - kubectl kustomize /tmp/two-tier-kustomize/base/
success_contains:
  - "kind: Deployment"
what_changed:
  - kustomization.yaml created listing all 4 resources
  - kubectl kustomize renders the combined output (nothing applied yet)
fallback_hint: Every directory used by kustomize must have a kustomization.yaml file.
```

```cka-step
step_id: p06-s03
title: create dev overlay
objective: Create a dev overlay that sets 1 replica and adds a DEV label.
ready_weight: 3
commands:
  - mkdir -p /tmp/two-tier-kustomize/overlays/dev
  - "cat > /tmp/two-tier-kustomize/overlays/dev/kustomization.yaml <<EOF\napiVersion: kustomize.config.k8s.io/v1beta1\nkind: Kustomization\nnamespace: dev\nresources:\n  - ../../base\ncommonLabels:\n  env: dev\npatches:\n  - target:\n      kind: Deployment\n      name: frontend\n    patch: |\n      - op: replace\n        path: /spec/replicas\n        value: 1\nEOF"
  - kubectl kustomize /tmp/two-tier-kustomize/overlays/dev/
success_check:
  - kubectl kustomize /tmp/two-tier-kustomize/overlays/dev/
success_contains:
  - "env: dev"
what_changed:
  - Dev overlay created with namespace override, env label, and 1 replica patch
  - Preview shows all resources tagged with env=dev
fallback_hint: resources points to the base using a relative path from the overlay directory.
```

```cka-step
step_id: p06-s04
title: create prod overlay
objective: Create a prod overlay with 3 frontend replicas and a different image tag.
ready_weight: 3
commands:
  - mkdir -p /tmp/two-tier-kustomize/overlays/prod
  - "cat > /tmp/two-tier-kustomize/overlays/prod/kustomization.yaml <<EOF\napiVersion: kustomize.config.k8s.io/v1beta1\nkind: Kustomization\nnamespace: prod\nresources:\n  - ../../base\ncommonLabels:\n  env: prod\nimages:\n  - name: nginx\n    newTag: 1.25-alpine\npatches:\n  - target:\n      kind: Deployment\n      name: frontend\n    patch: |\n      - op: replace\n        path: /spec/replicas\n        value: 3\nEOF"
  - kubectl kustomize /tmp/two-tier-kustomize/overlays/prod/
success_check:
  - kubectl kustomize /tmp/two-tier-kustomize/overlays/prod/
success_contains:
  - "env: prod"
what_changed:
  - Prod overlay created with 3 replicas, prod label, and pinned image tag
  - Same base manifests, completely different output
fallback_hint: The images transformer changes image tags without patching the full deployment spec.
```

```cka-step
step_id: p06-s05
title: compare overlays side by side
objective: See how the same base produces different outputs per environment.
ready_weight: 1
commands:
  - "echo '=== DEV ===' && kubectl kustomize /tmp/two-tier-kustomize/overlays/dev/ | grep -E 'replicas|namespace|env:'"
  - "echo '=== PROD ===' && kubectl kustomize /tmp/two-tier-kustomize/overlays/prod/ | grep -E 'replicas|namespace|env:|image:'"
success_check:
  - kubectl kustomize /tmp/two-tier-kustomize/overlays/prod/ | grep replicas
success_contains:
  - "3"
what_changed:
  - Nothing changed — compared rendered output
  - "Dev: 1 replica, dev label. Prod: 3 replicas, prod label, alpine image."
fallback_hint: kubectl kustomize is read-only — it renders but does not apply.
```

```cka-step
step_id: p06-s06
title: apply dev overlay
objective: Deploy the dev overlay to a dev namespace.
ready_weight: 2
commands:
  - kubectl create namespace dev
  - kubectl apply -k /tmp/two-tier-kustomize/overlays/dev/
  - kubectl get deploy,svc -n dev
success_check:
  - kubectl -n dev get deploy frontend -o jsonpath='{.spec.replicas}'
success_contains:
  - "1"
what_changed:
  - Dev environment deployed with 1 frontend replica
  - All resources labeled env=dev
fallback_hint: kubectl apply -k applies the rendered kustomize output to the cluster.
```

```cka-step
step_id: p06-s07
title: apply prod overlay
objective: Deploy the prod overlay to a prod namespace.
ready_weight: 2
commands:
  - kubectl create namespace prod
  - kubectl apply -k /tmp/two-tier-kustomize/overlays/prod/
  - kubectl get deploy,svc -n prod
success_check:
  - kubectl -n prod get deploy frontend -o jsonpath='{.spec.replicas}'
success_contains:
  - "3"
what_changed:
  - Prod environment deployed with 3 frontend replicas and alpine image
  - Same base, completely different deployment
fallback_hint: Both dev and prod namespaces exist independently — no conflicts.
```

```cka-step
step_id: p06-s08
title: clean up kustomize labs
objective: Remove the dev and prod namespaces used for testing.
ready_weight: 1
commands:
  - kubectl delete namespace dev prod
success_check:
  - kubectl get namespace dev 2>&1
success_contains:
  - NotFound
what_changed:
  - Dev and prod namespaces deleted
  - two-tier namespace still intact with the original app
fallback_hint: Deleting a namespace removes all resources in it.
```

## Exam Trap to Practice

Patch target misses `kind` and `name` — overlay applies but no changes appear. Always specify the exact target in your patch.

## Reflection Prompts

- When would you choose Kustomize over Helm (or vice versa)?
- What happens if you add a new resource to base — do overlays automatically include it?
- How would you add an environment-specific ConfigMap in only the prod overlay?

## What You Now Know

- Kustomize base + overlay directory structure
- kustomization.yaml with resources, patches, commonLabels, images, namespace
- Render previews with `kubectl kustomize`
- Apply overlays with `kubectl apply -k`
- JSON patches for targeted field changes

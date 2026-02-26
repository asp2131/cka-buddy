# Project 05 - Helm Release Operations

id: p05
type: project
domains: architecture,troubleshooting
difficulty: intermediate
timebox_min: 180

## Mission

Learn to operate Helm releases the way the CKA expects: install, inspect, upgrade, rollback, and uninstall. You will also package the two-tier app as a basic Helm chart to understand what charts contain.

## Context

You have been deploying the two-tier app with raw `kubectl apply`. Helm is how teams manage releases in practice, and it is now on the CKA exam. You need to be comfortable with `helm install`, `helm upgrade`, `helm rollback`, and `helm list`.

## Concepts

- **Chart**: a package of Kubernetes YAML templates + a `values.yaml` file.
- **Release**: a deployed instance of a chart. Same chart can be installed multiple times with different names.
- **Revision**: each install/upgrade creates a new revision. You can rollback to any previous revision.
- **Values**: configuration that overrides chart defaults. Set via `--set` flag or `-f values.yaml`.
- **Repository**: a hosted collection of charts (like npm registry for Kubernetes).

## Deliverables

- Add a chart repo, search, and install a release in a specific namespace.
- Inspect release details with `helm list`, `helm get`, and `helm history`.
- Upgrade a release with new values.
- Rollback to a previous revision.
- Understand chart structure by creating a minimal chart for the two-tier app.

## Runnable Steps (Strict)

```cka-step
step_id: p05-s01
title: add helm repo
objective: Add the Bitnami chart repository and search for available charts.
ready_weight: 1
commands:
  - helm repo add bitnami https://charts.bitnami.com/bitnami
  - helm repo update
  - helm search repo bitnami/nginx
success_check:
  - helm search repo bitnami/nginx
success_contains:
  - bitnami/nginx
what_changed:
  - Bitnami repo added to local Helm config
  - Chart index downloaded and searchable
fallback_hint: If helm is not installed, run "brew install helm" (macOS) or check https://helm.sh/docs/intro/install/
```

```cka-step
step_id: p05-s02
title: install a release
objective: Install the nginx chart as a release in a dedicated namespace.
ready_weight: 3
commands:
  - helm install web-release bitnami/nginx -n helm-lab --create-namespace --set service.type=ClusterIP
  - helm list -n helm-lab
success_check:
  - helm list -n helm-lab -o json
success_contains:
  - web-release
what_changed:
  - Release web-release deployed in helm-lab namespace
  - Helm tracks this as revision 1
fallback_hint: Always specify -n namespace for helm commands. Missing it deploys to default.
```

```cka-step
step_id: p05-s03
title: inspect the release
objective: View what Helm deployed and its current values.
ready_weight: 2
commands:
  - helm get values web-release -n helm-lab
  - helm get manifest web-release -n helm-lab | head -40
  - kubectl get deploy,svc,pods -n helm-lab
success_check:
  - helm get values web-release -n helm-lab
success_contains:
  - ClusterIP
what_changed:
  - Nothing changed — you inspected the release
  - '"helm get values" shows overridden values, "helm get manifest" shows rendered YAML'
fallback_hint: helm get manifest shows the actual Kubernetes YAML that was applied.
```

```cka-step
step_id: p05-s04
title: upgrade the release
objective: Upgrade the release to increase replica count.
ready_weight: 3
commands:
  - helm upgrade web-release bitnami/nginx -n helm-lab --set replicaCount=3 --set service.type=ClusterIP
  - helm history web-release -n helm-lab
  - kubectl get pods -n helm-lab
success_check:
  - helm history web-release -n helm-lab
success_contains:
  - "2"
what_changed:
  - Release upgraded to revision 2 with 3 replicas
  - Helm history shows both revisions
fallback_hint: Each upgrade creates a new revision. Previous values are NOT carried forward unless you use --reuse-values.
```

```cka-step
step_id: p05-s05
title: rollback the release
objective: Roll back to revision 1 (single replica).
ready_weight: 3
commands:
  - helm rollback web-release 1 -n helm-lab
  - helm history web-release -n helm-lab
  - kubectl get pods -n helm-lab
success_check:
  - helm history web-release -n helm-lab
success_contains:
  - superseded
what_changed:
  - Release rolled back to revision 1 values (creates revision 3)
  - Rollback is itself a new revision, not a deletion of history
fallback_hint: Specify the target revision number after the release name.
```

```cka-step
step_id: p05-s06
title: create a minimal chart
objective: Scaffold a Helm chart and understand the file structure.
ready_weight: 2
commands:
  - helm create /tmp/two-tier-chart
  - ls /tmp/two-tier-chart/
  - cat /tmp/two-tier-chart/Chart.yaml
  - ls /tmp/two-tier-chart/templates/
success_check:
  - cat /tmp/two-tier-chart/Chart.yaml
success_contains:
  - two-tier-chart
what_changed:
  - Scaffold chart created at /tmp/two-tier-chart
  - "Structure: Chart.yaml (metadata), values.yaml (defaults), templates/ (YAML templates)"
fallback_hint: helm create generates a full working chart. Inspect each file to understand the structure.
```

```cka-step
step_id: p05-s07
title: template render without install
objective: Preview what a chart would deploy without actually installing it.
ready_weight: 2
commands:
  - helm template test-render /tmp/two-tier-chart --set replicaCount=2
success_check:
  - helm template test-render /tmp/two-tier-chart --set replicaCount=2
success_contains:
  - "kind: Deployment"
what_changed:
  - Nothing changed — template renders locally without cluster access
  - This is useful for debugging chart output before install
fallback_hint: helm template is purely local — it does not contact the cluster.
```

```cka-step
step_id: p05-s08
title: uninstall and clean up
objective: Uninstall the release and delete the lab namespace.
ready_weight: 1
commands:
  - helm uninstall web-release -n helm-lab
  - kubectl delete namespace helm-lab
success_check:
  - helm list -n helm-lab
success_contains:
  - ""
what_changed:
  - Release web-release uninstalled (all resources deleted)
  - Namespace helm-lab removed
fallback_hint: helm uninstall removes the release AND all Kubernetes resources it created.
```

## Exam Trap to Practice

Release accidentally deployed to `default` namespace instead of the target. Always use `-n <namespace>` with every Helm command.

## Reflection Prompts

- What is the difference between `helm upgrade` and `kubectl apply`?
- Why does a rollback create a new revision instead of deleting the old one?
- When would you use `--reuse-values` vs passing all values again?

## What You Now Know

- Add repos, search, and install Helm charts
- Inspect releases with `helm list`, `helm get values`, `helm get manifest`
- Upgrade with `--set` and rollback to specific revisions
- Chart structure: Chart.yaml, values.yaml, templates/
- Render templates locally with `helm template`

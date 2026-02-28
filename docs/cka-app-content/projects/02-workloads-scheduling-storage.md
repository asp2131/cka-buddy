# Project 02 - Workloads, Scheduling, and Storage

id: p02
type: project
domains: workloads,storage
difficulty: intermediate
timebox_min: 210

## Mission

Give the backend a persistent database, control where pods land with scheduling constraints, and practice rollout strategies. Your two-tier app is growing up.

## Context

The `two-tier` namespace has frontend (3 replicas, ConfigMap) and backend (1 replica, Secret) from Project 01. Now you will add persistent storage for the backend and learn to control pod placement.

## Concepts

- **PersistentVolume (PV)**: a chunk of storage provisioned in the cluster.
- **PersistentVolumeClaim (PVC)**: a request for storage. Pods reference PVCs, not PVs directly.
- **StorageClass**: defines how storage is dynamically provisioned (on-demand PV creation).
- **Taint/Toleration**: taints repel pods from a node; tolerations let specific pods override that.
- **Node Affinity**: soft or hard rules that attract pods to specific nodes by label.

## Deliverables

- Create a PV and PVC, mount into the backend deployment.
- Create a StorageClass with `WaitForFirstConsumer` binding.
- Label a node and use nodeAffinity to pin the backend.
- Taint a node and deploy a tolerant workload.
- Practice a rollout and rollback on the frontend.

## Runnable Steps (Strict)

```cka-step
step_id: p02-s01
title: create persistent volume
objective: Create a hostPath PV for the backend database.
ready_weight: 2
commands:
  - "kubectl apply -f - <<EOF\napiVersion: v1\nkind: PersistentVolume\nmetadata:\n  name: backend-pv\nspec:\n  capacity:\n    storage: 256Mi\n  accessModes:\n    - ReadWriteOnce\n  hostPath:\n    path: /data/backend-db\n  storageClassName: manual\nEOF"
success_check:
  - kubectl get pv backend-pv -o jsonpath='{.status.phase}'
success_contains:
  - Available
what_changed:
  - PV backend-pv created with 256Mi capacity
  - Status is Available (not yet claimed)
fallback_hint: hostPath PVs are fine for local clusters. In production you would use a CSI driver.
```

```cka-step
step_id: p02-s02
title: create pvc and bind
objective: Create a PVC that binds to the PV.
ready_weight: 3
commands:
  - "kubectl -n two-tier apply -f - <<EOF\napiVersion: v1\nkind: PersistentVolumeClaim\nmetadata:\n  name: backend-pvc\n  namespace: two-tier\nspec:\n  accessModes:\n    - ReadWriteOnce\n  resources:\n    requests:\n      storage: 256Mi\n  storageClassName: manual\nEOF"
  - kubectl -n two-tier get pvc backend-pvc
success_check:
  - kubectl -n two-tier get pvc backend-pvc -o jsonpath='{.status.phase}'
success_contains:
  - Bound
what_changed:
  - PVC backend-pvc created and bound to backend-pv
  - Storage is now claimable by pods
fallback_hint: accessModes and storageClassName must match the PV exactly.
```

```cka-step
step_id: p02-s03
title: mount pvc into backend
objective: Add a volume mount to the backend deployment so it can persist data.
ready_weight: 3
commands:
  - kubectl -n two-tier get deployment backend -o yaml > /tmp/backend-vol.yaml
  - "echo 'In /tmp/backend-vol.yaml, add these exact blocks (copy/paste-safe):'"
  - "echo 'Under spec.template.spec (same level as containers:):'"
  - "echo '      volumes:'"
  - "echo '      - name: db-storage'"
  - "echo '        persistentVolumeClaim:'"
  - "echo '          claimName: backend-pvc'"
  - "echo 'Under spec.template.spec.containers[0] (same level as image:):'"
  - "echo '        volumeMounts:'"
  - "echo '        - name: db-storage'"
  - "echo '          mountPath: /data'"
  - vi /tmp/backend-vol.yaml
  - kubectl apply -f /tmp/backend-vol.yaml
success_check:
  - kubectl -n two-tier get pod -l app=backend -o jsonpath='{.items[0].spec.containers[0].volumeMounts[0].mountPath}'
success_contains:
  - /data
what_changed:
  - Backend pod restarted with PVC mounted at /data
  - Any data written to /data survives pod restarts
fallback_hint: Indentation must be spaces, not tabs. volumeMounts.name must exactly match volumes.name.
```

```cka-step
step_id: p02-s04
title: verify data persistence
objective: Write a file inside the backend, delete the pod, and confirm data survives.
ready_weight: 3
commands:
  - kubectl -n two-tier exec deploy/backend -- sh -c "echo 'persisted-data' > /data/test.txt"
  - kubectl -n two-tier delete pod -l app=backend
  - kubectl -n two-tier wait --for=condition=ready pod -l app=backend --timeout=60s
  - kubectl -n two-tier exec deploy/backend -- cat /data/test.txt
success_check:
  - kubectl -n two-tier exec deploy/backend -- cat /data/test.txt
success_contains:
  - persisted-data
what_changed:
  - Pod was deleted and recreated by the deployment controller
  - Data in /data survived because it lives on the PV, not the container
fallback_hint: The deployment creates a new pod automatically. Wait for it to be Ready.
```

```cka-step
step_id: p02-s05
title: label node for affinity
objective: Label a node and pin the backend to it with nodeAffinity.
ready_weight: 3
commands:
  - kubectl get nodes -o name | head -1 | xargs -I{} kubectl label {} workload=stateful
  - kubectl -n two-tier patch deployment backend --type=json -p '[{"op":"add","path":"/spec/template/spec/affinity","value":{"nodeAffinity":{"requiredDuringSchedulingIgnoredDuringExecution":{"nodeSelectorTerms":[{"matchExpressions":[{"key":"workload","operator":"In","values":["stateful"]}]}]}}}}]'
  - kubectl -n two-tier rollout status deployment/backend
success_check:
  - kubectl get nodes -l workload=stateful
success_contains:
  - Ready
what_changed:
  - Node labeled workload=stateful
  - Backend pods required to schedule on labeled nodes
fallback_hint: Use kubectl describe pod to check Events if the pod is Pending.
```

```cka-step
step_id: p02-s06
title: taint and toleration drill
objective: Taint a node and verify that only tolerant pods schedule on it.
ready_weight: 2
commands:
  - "echo 'WARNING: On single-node clusters, this can block scheduling for new pods until you remove the taint in the next step.'"
  - kubectl get nodes -o name | head -1 | xargs -I{} kubectl taint {} maintenance=true:NoSchedule
  - kubectl -n two-tier run taint-test --image=busybox --command -- sleep 3600
  - kubectl -n two-tier get pod taint-test
success_check:
  - kubectl -n two-tier get pod taint-test -o jsonpath='{.status.phase}'
success_contains:
  - Pending
what_changed:
  - Node tainted with maintenance=true:NoSchedule
  - Pod taint-test is Pending because it has no toleration
  - You must remove this taint in p02-s07 before continuing normal work
fallback_hint: On a single-node kind cluster, any taint blocks all new pods without tolerations.
```

```cka-step
step_id: p02-s07
title: add toleration and clean up taint
objective: Remove the taint so the cluster returns to normal. Clean up the test pod.
ready_weight: 2
commands:
  - kubectl get nodes -o name | head -1 | xargs -I{} kubectl taint {} maintenance=true:NoSchedule-
  - kubectl -n two-tier delete pod taint-test --ignore-not-found
  - kubectl get nodes -o jsonpath='{range .items[*]}{.metadata.name}{" taints="}{.spec.taints}{"\n"}{end}'
success_check:
  - kubectl get nodes -o custom-columns=NAME:.metadata.name,TAINTS:.spec.taints --no-headers
success_contains:
  - <none>
what_changed:
  - Taint removed (trailing minus sign removes a taint)
  - Test pod cleaned up, app pods unaffected
  - Verified maintenance taint is no longer present on nodes
fallback_hint: 'If taints are still present, rerun the remove command exactly with the trailing minus: maintenance=true:NoSchedule-'
```

```cka-step
step_id: p02-s08
title: rollout and rollback frontend
objective: Update the frontend image, then rollback to the previous version.
ready_weight: 3
commands:
  - kubectl -n two-tier set image deployment/frontend nginx=nginx:1.25-alpine --record
  - kubectl -n two-tier rollout status deployment/frontend
  - kubectl -n two-tier rollout history deployment/frontend
  - kubectl -n two-tier rollout undo deployment/frontend
  - kubectl -n two-tier rollout status deployment/frontend
success_check:
  - kubectl -n two-tier rollout history deployment/frontend
success_contains:
  - "1"
what_changed:
  - Frontend updated to nginx:1.25-alpine then rolled back
  - Rollout history shows multiple revisions
fallback_hint: rollout undo reverts to the previous revision. Use --to-revision=N for a specific one.
```

## Exam Trap to Practice

PVC remains `Pending`. Debug by checking: capacity mismatch, accessMode mismatch, and `storageClassName` mismatch between PV and PVC.

## Reflection Prompts

- What happens to data if you delete the PVC but not the PV?
- Why does nodeAffinity use `requiredDuringSchedulingIgnoredDuringExecution` instead of something simpler?
- When would you use a taint vs nodeAffinity to control scheduling?

## What You Now Know

- Create and bind PV/PVC, mount into deployments
- Verify data persistence across pod restarts
- Control scheduling with node labels, affinity, taints, and tolerations
- Perform deployment rollouts and rollbacks

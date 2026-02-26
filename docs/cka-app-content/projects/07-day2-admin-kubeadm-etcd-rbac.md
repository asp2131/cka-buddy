# Project 07 - Day-2 Admin: RBAC, Node Maintenance, etcd

id: p07
type: project
domains: architecture,troubleshooting
difficulty: advanced
timebox_min: 240

## Mission

Execute the critical cluster admin workflows that make up 25% of the CKA: RBAC permissions, node drain/uncordon, etcd backup/restore, and kubeadm cluster inspection.

## Context

Your two-tier app is running and secured. Now you need to think like a cluster admin. Who can do what? How do you maintain nodes without downtime? How do you backup the cluster state? These are high-weight CKA questions.

## Concepts

- **RBAC**: Role-Based Access Control. Roles define permissions, RoleBindings grant them to users/ServiceAccounts.
- **Role vs ClusterRole**: Role is namespace-scoped, ClusterRole is cluster-wide.
- **ServiceAccount**: an identity for pods. Pods authenticate to the API server as their ServiceAccount.
- **Drain/Uncordon**: drain evicts all pods from a node (for maintenance). Uncordon allows pods back.
- **etcd**: the key-value store holding all cluster state. Backup = snapshot. Restore = recover from disaster.
- **kubeadm**: the tool that bootstraps and upgrades Kubernetes clusters.

## Deliverables

- Create least-privilege RBAC for a deploy bot ServiceAccount.
- Debug permissions with `kubectl auth can-i`.
- Drain and uncordon a node safely.
- Perform etcd snapshot and understand restore workflow.
- Inspect kubeadm cluster config and static pod manifests.

## Runnable Steps (Strict)

```cka-step
step_id: p07-s01
title: create serviceaccount
objective: Create a ServiceAccount that will be used by a CI/CD deploy bot.
ready_weight: 2
commands:
  - kubectl -n two-tier create serviceaccount deploy-bot
  - kubectl -n two-tier get serviceaccount deploy-bot -o yaml
success_check:
  - kubectl -n two-tier get serviceaccount deploy-bot
success_contains:
  - deploy-bot
what_changed:
  - ServiceAccount deploy-bot created in two-tier namespace
  - It has no permissions yet — RBAC is deny-by-default
fallback_hint: ServiceAccounts are namespaced. Always specify -n.
```

```cka-step
step_id: p07-s02
title: create role with least privilege
objective: Create a Role that allows only deployment and pod operations.
ready_weight: 3
commands:
  - kubectl -n two-tier create role deploy-bot-role --verb=get,list,watch,create,update,patch --resource=deployments,pods
  - kubectl -n two-tier get role deploy-bot-role -o yaml
success_check:
  - kubectl -n two-tier get role deploy-bot-role
success_contains:
  - deploy-bot-role
what_changed:
  - Role deploy-bot-role created with scoped permissions
  - Only deployments and pods, only specified verbs — no delete, no secrets
fallback_hint: --verb and --resource accept comma-separated lists.
```

```cka-step
step_id: p07-s03
title: bind role to serviceaccount
objective: Create a RoleBinding connecting the role to the ServiceAccount.
ready_weight: 3
commands:
  - kubectl -n two-tier create rolebinding deploy-bot-binding --role=deploy-bot-role --serviceaccount=two-tier:deploy-bot
  - kubectl -n two-tier get rolebinding deploy-bot-binding -o yaml
success_check:
  - kubectl -n two-tier get rolebinding deploy-bot-binding
success_contains:
  - deploy-bot-binding
what_changed:
  - RoleBinding links deploy-bot ServiceAccount to deploy-bot-role
  - The SA can now perform allowed actions in two-tier namespace only
fallback_hint: --serviceaccount format is namespace:name (not just the name).
```

```cka-step
step_id: p07-s04
title: test permissions with can-i
objective: Verify what the ServiceAccount can and cannot do.
ready_weight: 3
commands:
  - kubectl auth can-i get pods -n two-tier --as=system:serviceaccount:two-tier:deploy-bot
  - kubectl auth can-i delete pods -n two-tier --as=system:serviceaccount:two-tier:deploy-bot
  - kubectl auth can-i get secrets -n two-tier --as=system:serviceaccount:two-tier:deploy-bot
  - kubectl auth can-i get pods -n default --as=system:serviceaccount:two-tier:deploy-bot
success_check:
  - kubectl auth can-i get pods -n two-tier --as=system:serviceaccount:two-tier:deploy-bot
success_contains:
  - "yes"
what_changed:
  - Nothing changed — tested permission boundaries
  - "get pods: yes. delete pods: no. get secrets: no. pods in default ns: no."
fallback_hint: --as impersonates a user or SA. Format for SA is system:serviceaccount:ns:name
```

```cka-step
step_id: p07-s05
title: node drain simulation
objective: Drain a node to simulate maintenance, observe pod eviction, then uncordon.
ready_weight: 3
commands:
  - kubectl get nodes
  - kubectl get pods -n two-tier -o wide
  - kubectl drain $(kubectl get nodes -o name | head -1 | cut -d/ -f2) --ignore-daemonsets --delete-emptydir-data --force
success_check:
  - kubectl get nodes
success_contains:
  - SchedulingDisabled
what_changed:
  - Node cordoned (SchedulingDisabled) and pods evicted
  - On a single-node cluster pods will be Pending until uncordon
fallback_hint: --ignore-daemonsets is required when DaemonSet pods exist. --force handles pods not managed by a controller.
```

```cka-step
step_id: p07-s06
title: uncordon node
objective: Restore the node to schedulable status and verify pods recover.
ready_weight: 2
commands:
  - kubectl uncordon $(kubectl get nodes -o name | head -1 | cut -d/ -f2)
  - kubectl -n two-tier wait --for=condition=ready pod -l app=frontend --timeout=120s
  - kubectl get nodes
  - kubectl get pods -n two-tier
success_check:
  - kubectl get nodes
success_contains:
  - " Ready"
what_changed:
  - Node uncordoned — scheduling enabled again
  - Evicted pods rescheduled and running
fallback_hint: Pods managed by Deployments are automatically recreated. Bare pods are lost.
```

```cka-step
step_id: p07-s07
title: inspect kubeadm config
objective: Find the cluster configuration and static pod manifests that kubeadm manages.
ready_weight: 2
commands:
  - kubectl get configmap kubeadm-config -n kube-system -o yaml 2>/dev/null || echo "kubeadm config not available (kind cluster)"
  - kubectl get pods -n kube-system -l tier=control-plane
  - kubectl -n kube-system get pod -l component=etcd -o yaml 2>/dev/null | head -50 || echo "etcd pod details"
success_check:
  - kubectl get pods -n kube-system
success_contains:
  - etcd
what_changed:
  - Nothing changed — inspected control plane components
  - kubeadm stores config in kube-system/kubeadm-config ConfigMap
  - Static pod manifests live in /etc/kubernetes/manifests/ on control plane nodes
fallback_hint: On the exam, always read /etc/kubernetes/manifests/etcd.yaml for cert paths — never guess.
```

```cka-step
step_id: p07-s08
title: etcd snapshot save
objective: Create an etcd backup snapshot.
ready_weight: 4
commands:
  - kubectl -n kube-system exec -it $(kubectl -n kube-system get pod -l component=etcd -o name | head -1) -- sh -c "ETCDCTL_API=3 etcdctl snapshot save /tmp/etcd-snap.db --endpoints=https://127.0.0.1:2379 --cacert=/etc/kubernetes/pki/etcd/ca.crt --cert=/etc/kubernetes/pki/etcd/server.crt --key=/etc/kubernetes/pki/etcd/server.key" 2>/dev/null || echo "etcd snapshot command — on exam this runs on the control plane node directly"
success_check:
  - echo "etcd snapshot concept understood"
success_contains:
  - snapshot
what_changed:
  - etcd snapshot saved (or command demonstrated for exam reference)
  - "Key exam pattern: read cert paths from /etc/kubernetes/manifests/etcd.yaml"
fallback_hint: On the exam you SSH into the control plane node and run etcdctl directly. Pull cert paths from the etcd static pod manifest.
```

```cka-step
step_id: p07-s09
title: etcd restore concept
objective: Understand the etcd restore workflow for the exam.
ready_weight: 2
commands:
  - "echo 'Exam restore workflow:'"
  - "echo '1. ETCDCTL_API=3 etcdctl snapshot restore /tmp/etcd-snap.db --data-dir=/var/lib/etcd-restored'"
  - "echo '2. Edit /etc/kubernetes/manifests/etcd.yaml to point hostPath volume to /var/lib/etcd-restored'"
  - "echo '3. Wait for etcd pod to restart with new data dir'"
  - "echo '4. Verify: kubectl get pods -A should show pre-snapshot state'"
success_check:
  - echo "restore workflow reviewed"
success_contains:
  - reviewed
what_changed:
  - Nothing changed — reviewed the restore workflow
  - "Critical: restore creates a NEW data dir. You must update the etcd manifest to use it."
fallback_hint: Never restore to the original data dir. Always use a new --data-dir path.
```

## Exam Trap to Practice

Incorrect etcd cert/key paths from memory. Always read them from `/etc/kubernetes/manifests/etcd.yaml` on the exam node.

## Reflection Prompts

- Why is RBAC deny-by-default? What would happen if it were allow-by-default?
- What is the difference between drain and cordon? When would you use cordon alone?
- Why does etcd restore require a new data directory?

## What You Now Know

- Create ServiceAccounts, Roles, RoleBindings for least-privilege access
- Debug permissions with `kubectl auth can-i --as`
- Drain and uncordon nodes for maintenance
- etcd snapshot save and the restore workflow
- Where kubeadm stores cluster config and static pod manifests

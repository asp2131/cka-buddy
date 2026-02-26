# CKA Mastery Path

id: cka-path-v1-34
type: index
domains: all
difficulty: progressive
timebox_min: 4800

## What this path optimizes for

- Passing score strategy: maximize Troubleshooting + Architecture performance first.
- Exam reality: SSH context discipline, timeboxing, and fast documentation lookup.
- Current blueprint: Kubernetes v1.34 with post-2025 additions.

## App Runtime Docs

- `app-loop-spec.md` (exact coaching loop contract)
- `learning-map.md` (readiness milestones and rewind policy)

## Project Sequence

0a. `projects/00-environment-and-first-contact.md`
0b. `projects/00b-kubectl-fluency-and-yaml.md`
1. `projects/01-cluster-foundations-and-speed.md`
2. `projects/02-workloads-scheduling-storage.md`
3. `projects/03-services-networkpolicy-dns.md`
4. `projects/04-gateway-api-migration.md`
5. `projects/05-helm-release-operations.md`
6. `projects/06-kustomize-overlays.md`
7. `projects/07-day2-admin-kubeadm-etcd-rbac.md`
8. `projects/08-troubleshooting-bootcamp.md`

## Bug Drill Pool

- `bugs/01-crashloop-configmap.md`
- `bugs/02-service-selector-mismatch.md`
- `bugs/03-default-deny-networkpolicy.md`
- `bugs/04-rbac-forbidden-serviceaccount.md`
- `bugs/05-kubelet-notready-cni.md`
- `bugs/06-gateway-parentref-backendref.md`

## Exam Layer

- `exam/runbook.md`
- `exam/speed-drills.md`
- `exam/mock-exam-blueprint.md`

## Completion Criteria

- You can solve common break/fix in under 8 minutes each.
- You can migrate Ingress to Gateway API from memory.
- You can perform kubeadm upgrade + etcd snapshot/restore safely.
- You score >= 80% on a full timed simulation before booking exam week.

# CKA Buddy Curriculum Evaluation & Restructuring Plan

## Date: 2025-02-25

## Current State

The curriculum in `docs/cka-app-content/` is well-designed for someone with existing kubectl familiarity (~20% knowledge) aiming for CKA readiness. The coaching loop UX, step verification format, bug drills, and exam-weighted focus are all strong.

## Critical Gap

The curriculum assumes a working Kubernetes cluster and kubectl fluency. For a zero-to-CKA learner, the jump from "never used kubectl" to Project 01 ("create namespace, deploy with 3 replicas, expose as ClusterIP, inject ConfigMap and Secret") is too large.

### What's Missing

1. **No environment setup** — no cluster provisioning (minikube/kind/k3s), no kubectl install, no kubeconfig setup, no "hello world" pod
2. **No conceptual grounding** — pod, node, namespace, deployment are used but never explained
3. **No progressive kubectl skill building** — no `get` -> `describe` -> `logs` -> `exec` progression, no output formatting practice, no `kubectl explain` or `--help` navigation
4. **No YAML literacy phase** — Projects reference `kubectl apply -f <file>.yaml` but never teach how to read/write/modify manifests or use the `dry-run -o yaml > file` workflow
5. **No cohesive example project** — each project is isolated; a single evolving app would create continuity

### What's Done Well

- Coaching loop UX (objective -> command -> verify -> advance)
- Exam-weighted domain focus (troubleshooting + architecture first)
- Bug drill injection for break/fix practice
- Message style guide (short, command-first, no fluff)
- Step format with `success_check`, `fallback_hint`, `what_changed`

## Restructured Curriculum

```
Phase 0: Environment & First Contact (NEW)
  - Install kubectl, set up kind/minikube cluster
  - Run first pod, see it in kubectl get pods
  - kubectl describe, logs, exec into it
  - Understand pod vs deployment vs service (conceptual)

Phase 1: kubectl Fluency & YAML Literacy (NEW)
  - kubectl explain, --help, -o yaml, -o jsonpath
  - dry-run workflow: generate -> edit -> apply
  - Basic YAML anatomy: apiVersion, kind, metadata, spec
  - Deploy a real 2-tier app (frontend + backend)

Phase 2: Foundations (current Project 01, with app context)
  - ConfigMaps, Secrets, replicas applied to the learner's app

Phase 3-8: Current Projects 02-08 (building on same app)
  - Storage = give the backend a persistent database
  - NetworkPolicy = lock down who talks to the database
  - Gateway API = expose the frontend to external traffic

Bug Drills + Exam Layer: Keep as-is
```

## Implementation Status

- [x] Evaluation complete
- [x] Phase 0: Environment & First Contact (00-environment-and-first-contact.md — 8 steps)
- [x] Phase 1: kubectl Fluency & YAML Literacy (00b-kubectl-fluency-and-yaml.md — 11 steps)
- [x] Phase 2: ConfigMaps, Secrets, Scaling (01 rewritten — 8 steps, builds on two-tier app)
- [x] Phase 3: Storage, Scheduling, Rollouts (02 rewritten — 8 steps, adds PV/PVC to backend)
- [x] Phase 4: NetworkPolicy, DNS, Security (03 rewritten — 8 steps, locks down two-tier app)
- [x] Phase 5: Gateway API (04 rewritten — 7 steps, exposes frontend externally)
- [x] Phase 6: Helm Operations (05 rewritten — 8 steps, with chart structure)
- [x] Phase 7: Kustomize Overlays (06 rewritten — 8 steps, dev/prod from same base)
- [x] Phase 8: Day-2 Admin (07 rewritten — 9 steps, RBAC/drain/etcd on the app)
- [x] Phase 9: Troubleshooting Bootcamp (08 rewritten — 15 steps/8 break-fix scenarios)
- [x] Learning map and index updated with all phases

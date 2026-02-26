# Mock Exam Blueprint (Project-Based)

id: e03
type: exam
domains: all
difficulty: exam
timebox_min: 120

## Structure

- 18 tasks total.
- Weights: Troubleshooting 30, Architecture 25, Networking 20, Workloads 15, Storage 10.
- Include at least 6 post-2025 tasks (Gateway API, Helm, Kustomize, CRDs/Operator touchpoints).

## Required Scenario Mix

- 4 troubleshooting break/fix (pod, service, node, control-plane).
- 3 cluster admin tasks (RBAC, kubeadm workflow, etcd snapshot).
- 3 networking tasks (NetworkPolicy, DNS, Gateway API route).
- 3 workloads tasks (rollout, sidecar, scheduling constraints).
- 2 storage tasks (static bind + dynamic class behavior).
- 3 flex tasks from bug bank.

## Passing Rule in App

Mark mock as pass when score >= 66 and no critical workflow violations (wrong node/context, no verification step).

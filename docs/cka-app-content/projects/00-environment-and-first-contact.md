# Project 00 - Environment and First Contact

id: p00
type: project
domains: architecture
difficulty: beginner
timebox_min: 90

## Mission

Get a working Kubernetes cluster on your machine and run your first commands. By the end you will have a cluster, understand what you are looking at, and feel comfortable poking around with `kubectl`.

## Concepts (Keep It Simple)

- **Cluster**: a set of machines (nodes) that run containers for you.
- **Node**: one machine in the cluster. Your laptop cluster has one node that plays every role.
- **Pod**: the smallest thing Kubernetes runs. A pod wraps one (or more) containers.
- **Namespace**: a folder-like boundary inside a cluster to keep resources organized.
- **kubectl**: the CLI you use to talk to the cluster API.

## Prerequisites

- A machine with Docker Desktop (macOS/Windows) or Docker Engine (Linux) installed and running.

## Deliverables

- A running `kind` cluster named `cka-lab`.
- Confirmation that `kubectl` can reach the cluster.
- One nginx pod running and accessible via `kubectl exec`.
- Comfort with: `get`, `describe`, `logs`, `exec`, `delete`.

## Runnable Steps (Strict)

```cka-step
step_id: p00-s01
title: install kind
objective: Install the kind CLI so you can create local clusters.
ready_weight: 1
commands:
  - 'echo "OS check: $(uname -s)"'
  - '[ "$(uname -s)" = "Darwin" ] && brew install kind || true'
  - '[ "$(uname -s)" = "Linux" ] && curl -Lo ./kind https://kind.sigs.k8s.io/dl/v0.23.0/kind-linux-amd64 && chmod +x ./kind && sudo mv ./kind /usr/local/bin/kind || true'
success_check:
  - kind --version
success_contains:
  - kind
what_changed:
  - kind CLI available in PATH
fallback_hint: 'Windows (PowerShell): winget install Kubernetes.kind. If your architecture is not amd64, download the correct kind binary from the official releases page and place it in your PATH.'
```

```cka-step
step_id: p00-s02
title: install kubectl
objective: Install kubectl so you can talk to any Kubernetes cluster.
ready_weight: 1
commands:
  - 'echo "OS check: $(uname -s)"'
  - '[ "$(uname -s)" = "Darwin" ] && brew install kubectl || true'
  - '[ "$(uname -s)" = "Linux" ] && curl -LO https://dl.k8s.io/release/$(curl -sL https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl && chmod +x kubectl && sudo mv kubectl /usr/local/bin/ || true'
success_check:
  - kubectl version --client --short 2>/dev/null || kubectl version --client
success_contains:
  - Client Version
what_changed:
  - kubectl CLI available in PATH
fallback_hint: 'Windows (PowerShell): winget install -e --id Kubernetes.kubectl. If your architecture is not amd64, use the matching kubectl binary for your OS/CPU from dl.k8s.io and move it into your PATH.'
```

```cka-step
step_id: p00-s03
title: create your first cluster
objective: Create a local Kubernetes cluster named cka-lab using kind.
ready_weight: 2
commands:
  - kind create cluster --name cka-lab
success_check:
  - kubectl cluster-info --context kind-cka-lab
success_contains:
  - is running
what_changed:
  - Single-node Kubernetes cluster running in Docker
  - kubeconfig context kind-cka-lab set as current
fallback_hint: Make sure Docker is running. Run "docker ps" to verify.
```

```cka-step
step_id: p00-s04
title: explore the cluster
objective: Look around the cluster to see what exists by default.
ready_weight: 1
commands:
  - kubectl get nodes
  - kubectl get namespaces
  - kubectl get pods -A
success_check:
  - kubectl get nodes
success_contains:
  - Ready
what_changed:
  - Nothing changed — these are read-only commands
  - You saw the node, default namespaces, and system pods
fallback_hint: If node shows NotReady, wait 30 seconds and retry — the cluster is still starting.
```

```cka-step
step_id: p00-s05
title: run your first pod
objective: Run a single nginx pod and confirm it starts.
ready_weight: 2
commands:
  - kubectl run hello --image=nginx
  - kubectl get pods
success_check:
  - kubectl get pod hello -o jsonpath='{.status.phase}'
success_contains:
  - Running
what_changed:
  - Pod hello created in default namespace
  - nginx container running inside the pod
fallback_hint: If status is ContainerCreating, wait a few seconds — the image is downloading.
```

```cka-step
step_id: p00-s06
title: inspect the pod
objective: Use describe and logs to see what is happening inside the pod.
ready_weight: 2
commands:
  - kubectl describe pod hello
  - kubectl logs hello
success_check:
  - kubectl get pod hello -o jsonpath='{.status.containerStatuses[0].ready}'
success_contains:
  - "true"
what_changed:
  - Nothing changed — these are read-only inspection commands
  - You saw events (scheduling, pulling, starting) and container stdout
fallback_hint: If Ready is false, run "kubectl describe pod hello" and check the Events section for pull/start errors.
```

```cka-step
step_id: p00-s07
title: exec into the pod
objective: Open a shell inside the running pod to prove it is a real container.
ready_weight: 2
commands:
  - kubectl exec -it hello -- /bin/sh -c "hostname && cat /etc/os-release | head -2"
success_check:
  - kubectl exec hello -- hostname
success_contains:
  - hello
what_changed:
  - Nothing changed — you ran a command inside the container
  - The hostname matches the pod name
fallback_hint: If exec hangs, make sure the pod is Running first.
```

```cka-step
step_id: p00-s08
title: delete and confirm
objective: Delete the pod and verify it is gone.
ready_weight: 1
commands:
  - kubectl delete pod hello
  - kubectl get pods
success_check:
  - kubectl get pod hello 2>&1
success_contains:
  - NotFound
what_changed:
  - Pod hello deleted
  - No pods remain in default namespace
fallback_hint: Deletion can take a few seconds. Add --wait=false to return immediately.
```

## Reflection Prompts

- What did `kubectl describe` show you that `kubectl get` did not?
- What is the difference between `kubectl logs` and `kubectl exec`?
- Why did the pod start running even though you never told Kubernetes which node to use?

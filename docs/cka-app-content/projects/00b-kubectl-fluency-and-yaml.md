# Project 00b - kubectl Fluency and YAML Literacy

id: p00b
type: project
domains: architecture,workloads
difficulty: beginner
timebox_min: 120

## Mission

Build the kubectl muscle memory and YAML reading/writing skills that every CKA task depends on. By the end you will generate manifests from the CLI, edit them confidently, and deploy a real two-tier app.

## Concepts

- **Deployment**: manages a set of identical pods. If a pod dies, the deployment replaces it.
- **Service**: a stable network address that routes traffic to pods matching a label selector.
- **Label**: a key-value tag on any resource. Services and other controllers use labels to find pods.
- **Manifest (YAML file)**: a text file describing the desired state of a resource. You apply it with `kubectl apply -f`.
- **dry-run**: a flag that makes kubectl generate the YAML without actually creating anything.
- **kubectl explain**: built-in docs for any field in any resource — your cheat sheet during the exam.

## Deliverables

- Confidence using `kubectl explain`, `--help`, `-o yaml`, `-o jsonpath`.
- Ability to generate, edit, and apply YAML manifests using the dry-run workflow.
- A running two-tier app: `frontend` (nginx) + `backend` (httpbin) connected via a Service.

## Part 1: kubectl Power Tools

### Runnable Steps (Strict)

```cka-step
step_id: p00b-s01
title: kubectl explain
objective: Use kubectl explain to discover the spec fields of a Deployment.
ready_weight: 1
commands:
  - kubectl explain deployment
  - kubectl explain deployment.spec.replicas
  - kubectl explain pod.spec.containers
success_check:
  - kubectl explain deployment.spec.replicas
success_contains:
  - replicas
what_changed:
  - Nothing changed — explain is a documentation lookup
  - You can drill into any field path (e.g. pod.spec.containers.ports)
fallback_hint: Use dot notation to go deeper — kubectl explain pod.spec.containers.image
```

```cka-step
step_id: p00b-s02
title: help flags
objective: Discover command flags using --help instead of memorizing.
ready_weight: 1
commands:
  - kubectl create deployment --help
  - kubectl run --help | head -20
success_check:
  - kubectl create deployment --help 2>&1
success_contains:
  - "--image"
what_changed:
  - Nothing changed — --help shows flags and examples
  - On the exam you cannot Google; --help is your lifeline
fallback_hint: Pipe to grep to find specific flags — kubectl run --help | grep -i port
```

```cka-step
step_id: p00b-s03
title: output formats
objective: View resources in different output formats.
ready_weight: 2
commands:
  - kubectl create namespace fluency-lab
  - kubectl -n fluency-lab run probe --image=busybox --command -- sleep 3600
  - kubectl -n fluency-lab get pod probe -o wide
  - kubectl -n fluency-lab get pod probe -o yaml
  - kubectl -n fluency-lab get pod probe -o jsonpath='{.metadata.name} on {.spec.nodeName}'
success_check:
  - kubectl -n fluency-lab get pod probe -o jsonpath='{.metadata.name}'
success_contains:
  - probe
what_changed:
  - Namespace fluency-lab created
  - Pod probe running for output format practice
  - You saw wide, yaml, and jsonpath output modes
fallback_hint: Wait for the pod to be Running before trying -o yaml (status fields need time to populate).
```

## Part 2: The dry-run Workflow (Exam Essential)

This is the single most important workflow for the CKA exam. Instead of writing YAML from scratch, you let kubectl generate it, then edit what you need.

```cka-step
step_id: p00b-s04
title: generate yaml with dry-run
objective: Generate a deployment manifest without creating anything.
ready_weight: 3
commands:
  - kubectl -n fluency-lab create deployment web --image=nginx --replicas=2 --dry-run=client -o yaml
  - kubectl -n fluency-lab create deployment web --image=nginx --replicas=2 --dry-run=client -o yaml > /tmp/web-deploy.yaml
success_check:
  - cat /tmp/web-deploy.yaml
success_contains:
  - "kind: Deployment"
what_changed:
  - YAML saved to /tmp/web-deploy.yaml
  - Nothing was created in the cluster — dry-run only generates output
fallback_hint: The > redirect saves stdout to a file. Use cat to verify contents.
```

```cka-step
step_id: p00b-s05
title: edit and apply the manifest
objective: Add a container port to the generated YAML and apply it.
ready_weight: 3
commands:
  - "echo '    Add under containers[0].spec:' && echo '      ports:' && echo '      - containerPort: 80'"
  - vi /tmp/web-deploy.yaml
  - kubectl apply -f /tmp/web-deploy.yaml
  - kubectl -n fluency-lab get deploy web
success_check:
  - kubectl -n fluency-lab get deploy web -o jsonpath='{.spec.replicas}'
success_contains:
  - "2"
what_changed:
  - Deployment web created from your edited YAML
  - 2 nginx pods running with containerPort 80 declared
fallback_hint: In vi press i to enter insert mode, add the ports block under containers, press Esc then :wq to save.
```

```cka-step
step_id: p00b-s06
title: generate a service with dry-run
objective: Generate and apply a ClusterIP service for the web deployment.
ready_weight: 2
commands:
  - kubectl -n fluency-lab expose deployment web --port=80 --target-port=80 --dry-run=client -o yaml > /tmp/web-svc.yaml
  - cat /tmp/web-svc.yaml
  - kubectl apply -f /tmp/web-svc.yaml
success_check:
  - kubectl -n fluency-lab get svc web
success_contains:
  - ClusterIP
what_changed:
  - Service web created with ClusterIP type
  - Traffic to port 80 routes to pod port 80
fallback_hint: expose reads the deployment labels automatically and sets the selector.
```

## Part 3: Deploy a Two-Tier App

Now put it all together. You will deploy a frontend and backend, connect them with a service, and verify end-to-end.

```cka-step
step_id: p00b-s07
title: create the app namespace
objective: Create a dedicated namespace for the two-tier app.
ready_weight: 1
commands:
  - kubectl create namespace two-tier
success_check:
  - kubectl get namespace two-tier
success_contains:
  - two-tier
what_changed:
  - Namespace two-tier created
fallback_hint: Namespace names must be lowercase with no spaces.
```

```cka-step
step_id: p00b-s08
title: deploy backend
objective: Deploy an httpbin backend and expose it as a ClusterIP service.
ready_weight: 3
commands:
  - kubectl -n two-tier create deployment backend --image=kennethreitz/httpbin --dry-run=client -o yaml > /tmp/backend.yaml
  - kubectl apply -f /tmp/backend.yaml
  - kubectl -n two-tier expose deployment backend --port=80 --target-port=80
success_check:
  - kubectl -n two-tier get endpoints backend
success_contains:
  - ":"
what_changed:
  - Deployment backend running with httpbin image
  - Service backend routing to pod on port 80
fallback_hint: If endpoints show <none>, the pod is not ready yet — wait and retry.
```

```cka-step
step_id: p00b-s09
title: deploy frontend
objective: Deploy an nginx frontend that will proxy to the backend.
ready_weight: 2
commands:
  - kubectl -n two-tier create deployment frontend --image=nginx --replicas=2
  - kubectl -n two-tier expose deployment frontend --port=80 --target-port=80
success_check:
  - kubectl -n two-tier get deploy frontend -o jsonpath='{.status.readyReplicas}'
success_contains:
  - "2"
what_changed:
  - Deployment frontend running with 2 replicas
  - Service frontend created
fallback_hint: Check pod status with kubectl -n two-tier get pods if replicas are not ready.
```

```cka-step
step_id: p00b-s10
title: verify connectivity
objective: Prove that frontend pods can reach the backend service by DNS name.
ready_weight: 3
commands:
  - kubectl -n two-tier exec deploy/frontend -- curl -s http://backend.two-tier.svc.cluster.local/get
success_check:
  - kubectl -n two-tier exec deploy/frontend -- curl -s -o /dev/null -w '%{http_code}' http://backend.two-tier.svc.cluster.local/get
success_contains:
  - "200"
what_changed:
  - Nothing changed — this is a connectivity test
  - Frontend pod resolved backend via cluster DNS and got HTTP 200
fallback_hint: The DNS pattern is <service>.<namespace>.svc.cluster.local — check service name and namespace.
```

```cka-step
step_id: p00b-s11
title: cleanup fluency-lab
objective: Delete the practice namespace to keep the cluster tidy.
ready_weight: 1
commands:
  - kubectl delete namespace fluency-lab
success_check:
  - kubectl get namespace fluency-lab 2>&1
success_contains:
  - NotFound
what_changed:
  - Namespace fluency-lab and all its resources deleted
  - Namespace two-tier kept — you will build on it in later projects
fallback_hint: Namespace deletion can take 30 seconds. Be patient.
```

## Reflection Prompts

- How is `kubectl explain` different from `kubectl --help`? When would you use each?
- Why is the dry-run workflow faster than writing YAML from scratch?
- What DNS name did the frontend use to reach the backend? What are the parts of that name?
- What would happen if you deleted the backend service but not the deployment?

## What You Now Know

After completing this project you can:

- Look up any resource field with `kubectl explain`
- Generate manifests with `--dry-run=client -o yaml` (exam essential)
- Read and edit YAML manifests confidently
- Deploy multi-component apps and verify connectivity
- Use `-o wide`, `-o yaml`, `-o jsonpath` to extract information

You are ready for Project 01.

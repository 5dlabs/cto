Implement subtask 7015: Create Kubernetes deployment manifest for Morgan multi-container pod

## Objective
Write the Kubernetes Deployment, PVC, Service, and ConfigMap manifests for the Morgan agent pod with OpenClaw agent container and Signal-CLI sidecar, including resource limits, environment variable injection from sigma1-infra-endpoints and sigma1-external-secrets.

## Steps
1. Create Kubernetes Deployment manifest:
   - Namespace: `sigma1`
   - Deployment name: `morgan-agent`
   - Pod spec with 2 containers:
     a. `morgan` container (OpenClaw agent runtime):
        - Image: configured via environment variable or values file
        - Port: 8000 (HTTP for webhooks, WebSocket for chat)
        - Resource limits: 1Gi memory, 500m CPU
        - Resource requests: 512Mi memory, 250m CPU
        - envFrom: sigma1-infra-endpoints ConfigMap, sigma1-external-secrets Secret
        - Volume mounts: morgan-workspace at /data
     b. `signal-cli` container (sidecar):
        - Image: `bbernhard/signal-cli-rest-api:latest`
        - Port: 8080 (internal only)
        - Resource limits: 512Mi memory, 250m CPU
        - Resource requests: 256Mi memory, 100m CPU
        - Volume mounts: signal-cli-data at /home/.local/share/signal-cli
        - Environment: SIGNAL_CLI_WEBHOOK_URL=http://localhost:8000/api/signal/incoming
2. Create PersistentVolumeClaim:
   - `morgan-workspace-pvc`: 10Gi, ReadWriteOnce
   - Used by both morgan (agent workspace) and signal-cli (device data) via subPath mounts
3. Create Service:
   - `morgan-agent-svc`: ClusterIP
   - Port 8000 → morgan container port 8000
   - No external exposure of signal-cli port 8080
4. Health probes for morgan container:
   - Readiness: GET /health/ready → 200 (checks LLM, Signal-CLI, critical MCP tools)
   - Liveness: GET /health/live → 200 (basic process alive check)
   - Startup probe: 60 second initialDelaySeconds (LLM connection warmup)
5. Pod labels: app=morgan-agent, component=ai-agent, part-of=sigma1
6. Anti-affinity: prefer scheduling away from other AI workloads if any.

## Validation
Apply manifests to test cluster with `kubectl apply --dry-run=server`. Verify pod starts with both containers running. Verify PVC is bound. Verify Service routes traffic to port 8000. Verify health probes pass when dependencies are available and fail appropriately when they're not. Verify envFrom injects expected environment variables.
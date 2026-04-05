Implement subtask 7001: Deploy OpenClaw agent in Kubernetes with workspace volume and configuration

## Objective
Create and apply the Kubernetes deployment manifest for the OpenClaw Morgan agent, including AGENT_ID, MODEL env vars, workspace persistent volume, resource requests/limits, and service account.

## Steps
1. Create a Deployment manifest for the OpenClaw agent container image.
2. Configure environment variables: AGENT_ID, MODEL (referencing the chosen LLM), MCP_TOOL_SERVER_URL, and any API keys via Secret references.
3. Attach a PersistentVolumeClaim for the agent workspace (conversation state, logs).
4. Define resource requests (cpu: 500m, memory: 1Gi) and limits (cpu: 2, memory: 4Gi).
5. Create a ClusterIP Service exposing the agent's HTTP/WebSocket ports.
6. Create a ConfigMap for agent configuration (system prompt, skill definitions path, tool registry URL).
7. Apply manifests and verify the pod reaches Running state with readiness probe passing.

## Validation
Pod is Running and Ready. `kubectl logs` shows successful agent initialization with AGENT_ID logged. Readiness probe endpoint returns 200. Workspace volume is mounted and writable.
Implement subtask 7001: Deploy OpenClaw agent on Kubernetes with sigma1-infra-endpoints configuration

## Objective
Create the Kubernetes Deployment, Service, and ConfigMap references for the OpenClaw Morgan agent, pulling all service URLs and credentials from the sigma1-infra-endpoints ConfigMap and associated Secrets.

## Steps
1. Create a Deployment manifest for the OpenClaw agent container image with resource requests/limits appropriate for an LLM-orchestrating agent.
2. Reference 'sigma1-infra-endpoints' ConfigMap via envFrom to inject service URLs for catalog, RMS, finance, vetting, and social services.
3. Mount any required Secrets (API keys for ElevenLabs, Twilio, Signal credentials) as environment variables or volume mounts.
4. Create a ClusterIP Service exposing the agent's HTTP/WebSocket port for internal communication.
5. Configure liveness and readiness probes (e.g., /healthz endpoint).
6. Set up the OpenClaw agent configuration file (system prompt, model selection, temperature, max tokens) as a ConfigMap.
7. Verify the pod starts, passes readiness checks, and can resolve all service URLs from the injected environment.

## Validation
Pod reaches Running/Ready state; readiness probe passes; environment variables from sigma1-infra-endpoints are correctly injected (exec into pod and verify); agent /healthz endpoint returns 200.
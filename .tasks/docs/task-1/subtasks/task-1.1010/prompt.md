Implement subtask 1010: Define Signal-CLI sidecar container template

## Objective
Create a reusable Signal-CLI sidecar container specification as a Kustomize component or ConfigMap that downstream service Deployments can include.

## Steps
1. Create a Kustomize component (or ConfigMap with JSON/YAML container spec) defining the Signal-CLI sidecar:
   - Container name: `signal-cli-sidecar`
   - Image: `bbernhard/signal-cli-rest-api:latest`
   - Resource limits: `memory: 512Mi`, `cpu: 500m`
   - Resource requests: `memory: 256Mi`, `cpu: 250m`
   - Liveness probe: HTTP GET on Signal-CLI REST health endpoint (typically port 8080, path `/v1/health`)
   - Readiness probe: same endpoint
   - Restart policy inherited from pod (Always)
   - Environment variables: `MODE=json-rpc` or `MODE=native` as appropriate
   - Port: 8080 (containerPort)
2. If using Kustomize component, create `components/signal-cli-sidecar/kustomization.yaml` with a strategic merge patch.
3. If using ConfigMap, store the container JSON under key `signal-cli-sidecar-spec` in ConfigMap `sigma1-sidecar-templates`.
4. Document how downstream services should reference/include this template.

## Validation
The Kustomize component or ConfigMap exists and contains a valid container spec. Running `kustomize build` on a test overlay that includes the component produces a Deployment with the signal-cli-sidecar container correctly injected. Container spec includes liveness probe, resource limits, and correct image.
Implement subtask 7001: Deploy OpenClaw agent base in openclaw namespace

## Objective
Deploy the OpenClaw agent using the provided deployment manifest into the openclaw namespace. Configure base environment variables, resource limits, and wire up the sigma1-infra-endpoints ConfigMap via envFrom so all backend service URLs are available to the agent runtime.

## Steps
1. Create or verify the `openclaw` namespace exists.
2. Apply the provided OpenClaw deployment manifest (Deployment, Service, ServiceAccount).
3. Attach the `sigma1-infra-endpoints` ConfigMap via `envFrom` on the agent container.
4. Configure base agent settings: system prompt skeleton, model endpoint, temperature, max tokens.
5. Set resource requests/limits appropriate for a single-replica dev deployment.
6. Verify the agent pod reaches Running state and the health endpoint responds.
7. Confirm all ConfigMap environment variables are injected correctly by exec-ing into the pod and printing env.

## Validation
Pod is in Running state; `kubectl exec` into the pod and verify all sigma1-infra-endpoints env vars are present; agent health endpoint returns 200; logs show successful startup with no configuration errors.
Implement subtask 1005: Deploy Signal-CLI for Morgan agent integration

## Objective
Deploy Signal-CLI as a standalone pod in the sigma1 namespace with REST API access, enabling the Morgan agent to send and receive Signal messages.

## Steps
1. Author a Deployment manifest for signal-cli-rest-api (Docker image: bbernhard/signal-cli-rest-api or equivalent) in the sigma1 namespace.
2. Configure a PersistentVolumeClaim for Signal-CLI's data directory (~/.local/share/signal-cli) to persist registration state across restarts.
3. Expose the REST API on port 8080 via a ClusterIP Service 'signal-cli' in the sigma1 namespace.
4. Create a Secret 'signal-cli-config' containing the Signal phone number and any registration tokens.
5. Add liveness and readiness probes hitting the /v1/about endpoint.
6. Set resource requests (128Mi RAM / 100m CPU) and limits (512Mi RAM / 500m CPU).
7. Document the registration process (manual step: register the phone number via the API after first deploy).

## Validation
kubectl get pods -n sigma1 shows signal-cli pod Running and Ready; curl signal-cli.sigma1.svc:8080/v1/about returns version info; verify PVC is bound and data directory is writable.
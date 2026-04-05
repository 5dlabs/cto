Implement subtask 1005: Deploy Signal-CLI as a standalone pod for Morgan agent

## Objective
Deploy Signal-CLI as a dedicated Kubernetes Deployment in the sigma1 namespace, configured to receive and send Signal messages for the Morgan AI agent.

## Steps
1. Create a Deployment YAML for Signal-CLI using the `bbernhard/signal-cli-rest-api` container image (or equivalent).
2. Deploy in the `sigma1` namespace with resource limits (256Mi RAM, 250m CPU).
3. Mount a PersistentVolumeClaim for Signal-CLI's data directory (stores registration state).
4. Expose Signal-CLI via a ClusterIP Service on port 8080 (REST API).
5. Configure liveness and readiness probes against the Signal-CLI health endpoint.
6. Store any Signal registration credentials or phone number in a Kubernetes Secret `signal-cli-credentials`.
7. Apply the manifests and verify the pod is Running.
8. Record the SIGNAL_CLI_URL (e.g., `http://signal-cli.sigma1.svc.cluster.local:8080`) for ConfigMap creation.

## Validation
Confirm the Signal-CLI pod is Running with passing health probes. Curl the REST API endpoint from within the cluster and verify a valid response. Confirm the PVC is bound and the credentials secret exists.
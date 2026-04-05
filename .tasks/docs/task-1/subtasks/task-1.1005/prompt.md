Implement subtask 1005: Deploy Signal-CLI pod in openclaw namespace

## Objective
Deploy Signal-CLI as a standalone pod or Deployment in the openclaw namespace, configured for Morgan agent messaging integration.

## Steps
1. Create a Deployment manifest for Signal-CLI in the 'openclaw' namespace. 2. Use a community Signal-CLI REST API Docker image (e.g., bbernhard/signal-cli-rest-api). 3. Mount a PersistentVolumeClaim for Signal-CLI data directory (~/.local/share/signal-cli) to persist registration state. 4. Expose the Signal-CLI REST API via a ClusterIP Service on port 8080. 5. Configure resource limits (256Mi-512Mi RAM, 250m CPU). 6. Document the registration process: after deployment, the Signal number must be registered/linked via the REST API. 7. Record the Signal-CLI service URL (http://signal-cli.openclaw.svc.cluster.local:8080) for the aggregated ConfigMap.

## Validation
Signal-CLI pod is Running; curl the /v1/about endpoint from within the cluster and receive a valid JSON response; verify the PVC is bound.
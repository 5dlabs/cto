Implement subtask 1005: Deploy Signal-CLI pod in openclaw namespace

## Objective
Deploy Signal-CLI as a standalone pod (or deployment) in the openclaw namespace, configured for REST API access by downstream services.

## Steps
1. Create a Deployment manifest for Signal-CLI in the openclaw namespace using the bbernhard/signal-cli-rest-api image (or equivalent).
2. Mount a PersistentVolumeClaim for Signal-CLI data directory (registration, keys).
3. Expose the Signal-CLI REST API on port 8080 via a ClusterIP Service.
4. Configure environment variables for the Signal-CLI instance.
5. Apply the Deployment and Service manifests.
6. Record the SIGNAL_CLI_URL (e.g., http://signal-cli.openclaw.svc.cluster.local:8080) for ConfigMap aggregation.

## Validation
Verify the Signal-CLI pod is Running. Curl the health/about endpoint from a test pod in the openclaw namespace and confirm a valid JSON response. Verify the Service is reachable at the expected ClusterIP DNS name.
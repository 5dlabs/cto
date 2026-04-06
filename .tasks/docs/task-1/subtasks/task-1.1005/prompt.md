Implement subtask 1005: Deploy Signal-CLI pod in openclaw namespace

## Objective
Deploy Signal-CLI as a standalone pod (or Deployment) in the openclaw namespace to serve as the Signal messaging relay for the Morgan agent.

## Steps
1. Create a Deployment manifest in the openclaw namespace:
   - name: signal-cli
   - image: bbernhard/signal-cli-rest-api:latest (or equivalent)
   - replicas: 1
   - env: MODE=json-rpc or MODE=native (depending on integration needs)
   - volume: PersistentVolumeClaim for Signal-CLI data directory (~1Gi) to store registration state
2. Create a Service: signal-cli.openclaw.svc.cluster.local:8080.
3. Create a Secret sigma1-signal-cli-config for the Signal phone number and registration data if pre-registered.
4. Apply the manifests and wait for the pod to be Running.
5. Record the service URL: http://signal-cli.openclaw.svc.cluster.local:8080 for the ConfigMap.

## Validation
Verify the signal-cli pod is Running in openclaw namespace. Curl http://signal-cli.openclaw.svc.cluster.local:8080/v1/about from a debug pod and confirm a valid JSON response with version info.
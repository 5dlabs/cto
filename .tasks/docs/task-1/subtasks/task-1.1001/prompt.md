Implement subtask 1001: Create Kubernetes namespaces for Sigma-1 platform

## Objective
Create all required Kubernetes namespaces that will host the various infrastructure components and application services: databases, sigma1, openclaw, social, web. Apply standard labels and annotations for organizational tracking.

## Steps
1. Create a YAML manifest file `namespaces.yaml` defining all namespaces: `databases`, `sigma1`, `openclaw`, `social`, `web`.
2. Add standard labels to each namespace: `app.kubernetes.io/part-of: sigma1`, `app.kubernetes.io/managed-by: bolt`.
3. Apply the manifest using `kubectl apply -f namespaces.yaml`.
4. Verify all namespaces are Active with `kubectl get namespaces`.
5. Ensure no conflicting resource quotas or limit ranges exist that would block subsequent deployments.

## Validation
Run `kubectl get namespaces` and confirm all five namespaces (databases, sigma1, openclaw, social, web) exist with status Active. Verify labels are correctly applied using `kubectl get ns -l app.kubernetes.io/part-of=sigma1`.
Implement subtask 1001: Create Kubernetes namespaces: sigma1, databases, openclaw, signal

## Objective
Define and apply Namespace manifests for all four namespaces required by the Sigma-1 platform. These namespaces gate every subsequent resource deployment.

## Steps
Create a Helm chart at sigma1/infra/templates/namespaces.yaml (or equivalent). Define four Namespace resources: sigma1, databases, openclaw, signal. Apply labels: app.kubernetes.io/part-of=sigma1, managed-by=helm. Ensure the chart includes this template first in render order so dependent resources in the same chart do not fail. Add a Helm hook weight or ordering note if necessary. Apply with: helm upgrade --install sigma1-infra sigma1/infra --namespace sigma1 --create-namespace.

## Validation
Run: kubectl get namespaces sigma1 databases openclaw signal — all four must show STATUS=Active. Verify labels with: kubectl get ns sigma1 --show-labels.
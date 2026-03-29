Implement subtask 1001: Create notifycore namespace and Helm chart scaffold

## Objective
Create the `notifycore` Kubernetes namespace and initialize the Helm chart directory structure under `infra/notifycore/` with Chart.yaml, values-dev.yaml, and templates directory.

## Steps
1. Create `infra/notifycore/Chart.yaml` with name `notifycore`, version `0.1.0`, apiVersion `v2`.
2. Create `infra/notifycore/values-dev.yaml` with placeholder sections for postgres, redis, and configmap settings (single-replica sizing).
3. Create `infra/notifycore/templates/namespace.yaml` defining the `notifycore` namespace resource.
4. Ensure the Helm chart structure is valid by running `helm lint infra/notifycore/`.

## Validation
`helm lint infra/notifycore/` passes without errors. The namespace template renders correctly via `helm template`. Directory structure contains Chart.yaml, values-dev.yaml, and templates/ directory.
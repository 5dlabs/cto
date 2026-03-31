Implement subtask 1001: Create hermes-dev and hermes-staging namespaces with labels and Loki verification

## Objective
Create both Kubernetes namespaces with standard labels and verify Loki log shipping is accessible from the new namespaces.

## Steps
1. Create `hermes-dev` namespace with labels: `app.kubernetes.io/part-of: hermes`, `environment: dev`.
2. Create `hermes-staging` namespace with labels: `app.kubernetes.io/part-of: hermes`, `environment: staging`.
3. Define these as Helm templates in `charts/hermes-infra/templates/namespace.yaml` with environment-specific values.
4. Deploy a short-lived test pod in `hermes-dev` that emits a structured JSON log line (e.g., `{"level":"info","msg":"hermes-loki-test","timestamp":"..."}`). Verify the log is queryable in Loki via LogQL (`{namespace="hermes-dev"} |= "hermes-loki-test"`) within 30 seconds.
5. Clean up the test pod after verification.

## Validation
`kubectl get namespace hermes-dev hermes-staging` returns both namespaces in Active state with correct labels. A test pod log is queryable in Loki within 30 seconds.
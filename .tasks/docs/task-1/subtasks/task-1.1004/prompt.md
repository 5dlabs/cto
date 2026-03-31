Implement subtask 1004: Deploy NATS operator CRs and credential secrets

## Objective
Deploy single-replica NATS custom resources in both namespaces for future decoupling, with connection strings stored in secrets.

## Steps
1. Create a Helm template for the NATS operator CR in `charts/hermes-infra/templates/nats.yaml`.
2. Configure single-replica for both dev and staging.
3. Store NATS connection string in a secret named `hermes-nats-credentials` containing `url` key (e.g., `nats://hermes-nats:4222`).
4. This is not actively wired per D1 but must be provisioned and available.
5. Add values for resource limits in `values-dev.yaml` and `values-staging.yaml`.

## Validation
`kubectl get nats -n hermes-dev` (or equivalent CRD) shows a Ready instance. `kubectl get secret hermes-nats-credentials -n hermes-dev` exists with valid NATS URL. A test pod can establish a NATS connection without errors.
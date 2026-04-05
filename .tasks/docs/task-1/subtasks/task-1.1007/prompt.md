Implement subtask 1007: Provision Kubernetes Secrets for third-party API keys

## Objective
Create Kubernetes Secrets in the appropriate namespaces for all third-party API keys required by Sigma-1 services (Stripe, OpenCorporates, LinkedIn, Google, etc.).

## Steps
1. Identify all required API keys from the PRD: Stripe (secret key, publishable key, webhook secret), OpenCorporates API key, LinkedIn API credentials, Google API credentials, and any others.
2. For each set of credentials, create a Kubernetes Secret in the namespace where the consuming service runs:
   - `stripe-credentials` in sigma1 namespace
   - `opencorporates-credentials` in sigma1 namespace
   - `linkedin-credentials` in social namespace
   - `google-credentials` in sigma1 namespace
3. Use `kubectl create secret generic <name> --from-literal=KEY=value` or sealed-secrets if available.
4. Label all secrets with `app.kubernetes.io/part-of: sigma1` for discoverability.
5. Document each secret name, namespace, and key names in a markdown reference file.

## Validation
Verify each expected secret exists in its target namespace with `kubectl get secret`. Confirm each secret contains the expected keys (without revealing values). Validate that the documentation file lists all secrets with their namespaces and key names.
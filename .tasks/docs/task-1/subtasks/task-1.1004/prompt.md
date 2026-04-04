Implement subtask 1004: Create ExternalSecret CRDs for LINEAR_API_TOKEN and GITHUB_API_TOKEN

## Objective
Create ExternalSecret CRDs that reference the cluster's backing secret store to sync `LINEAR_API_TOKEN` and `GITHUB_API_TOKEN` into Kubernetes Secrets in the `sigma-1-dev` namespace.

## Steps
1. Create an ExternalSecret CR for `LINEAR_API_TOKEN`: name=`sigma-1-linear-api-token`, namespace=`sigma-1-dev`, secretStoreRef pointing to the cluster's SecretStore/ClusterSecretStore, remoteRef.key set to the path in the external store (e.g., `sigma-1/linear-api-token`), target secret name=`sigma-1-linear-api-token`.
2. Create an ExternalSecret CR for `GITHUB_API_TOKEN`: name=`sigma-1-github-api-token`, same pattern, remoteRef.key=`sigma-1/github-api-token`, target secret name=`sigma-1-github-api-token`.
3. Apply both manifests.
4. Verify each ExternalSecret reaches `Ready=True` in `status.conditions` within 2 minutes.
5. Verify the target Kubernetes Secrets exist and have non-empty data keys.

## Validation
`kubectl get externalsecret sigma-1-linear-api-token -n sigma-1-dev -o jsonpath='{.status.conditions[?(@.type=="Ready")].status}'` returns `True`. Same for `sigma-1-github-api-token`. `kubectl get secret sigma-1-linear-api-token -n sigma-1-dev -o jsonpath='{.data}'` is non-empty. Same for GitHub token secret.
## Decision Points

- Which SecretStore (ClusterSecretStore vs namespaced SecretStore) should the ExternalSecrets reference, and what is the exact store name in the cluster?
- What are the exact backing store paths (e.g., Vault paths, AWS SSM parameter names) for each of the 4 secrets — do they already exist or need manual pre-population?
- What is the correct NOUS_API_URL endpoint — is it `https://api.nous.com` or a different URL, and should HERMES_URL be left empty or pointed at a known dev endpoint?

## Coordination Notes

- Agent owner: bolt
- Primary stack: Kubernetes/Helm
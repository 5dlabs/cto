Implement subtask 10002: Deploy and configure External Secrets Operator for centralized secret management

## Objective
Install the Kubernetes External Secrets Operator (ESO) and configure SecretStore/ClusterSecretStore resources to sync secrets from the chosen external secrets backend into Kubernetes Secrets.

## Steps
1. Install the External Secrets Operator via Helm chart (`external-secrets/external-secrets`) into a dedicated namespace. 2. Create the authentication credential (e.g., IAM role, Vault token, API key) for the external secrets backend and store it as a bootstrap Kubernetes Secret. 3. Create a ClusterSecretStore (or namespace-scoped SecretStore) resource pointing to the external backend with the authentication reference. 4. For each existing application secret (database credentials, API keys for Stripe, LinkedIn, etc.), create an ExternalSecret CR that maps the external secret path to a Kubernetes Secret name and key. 5. Verify that each ExternalSecret syncs successfully and the resulting Kubernetes Secret contains the correct values. 6. Update all Deployments to reference the ESO-managed Secrets (if not already using the same Secret names). 7. Remove any manually-created Kubernetes Secrets that are now managed by ESO.

## Validation
Verify ESO pods are healthy with `kubectl get pods -n external-secrets`. Check each ExternalSecret status with `kubectl get externalsecret -A` and confirm all show `SecretSynced` condition. Verify application pods can access the synced secrets by checking logs for successful database/API connections. Manually update a value in the external backend and confirm the Kubernetes Secret updates within the configured refresh interval.
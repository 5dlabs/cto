Implement subtask 10003: Implement automated secret rotation for API keys and service credentials

## Objective
Set up automated rotation for external API keys (e.g., OpenAI, Cloudflare) and inter-service credentials, ensuring zero-downtime rotation with proper coordination.

## Steps
1. Inventory all non-database secrets: external API keys (OpenAI, Cloudflare, etc.), inter-service tokens, webhook secrets.
2. For each secret type, determine if the provider supports key rotation (e.g., creating a second API key before revoking the old one).
3. For API keys that support dual-key rotation:
   a. Create a rotation script/CronJob that generates a new key via the provider API, updates the Kubernetes Secret, waits for pod rollout, then revokes the old key.
4. For API keys that don't support dual-key:
   a. Create a rotation script that updates both the provider and the Kubernetes Secret atomically, then triggers a rolling restart.
5. For inter-service credentials (e.g., shared secrets, JWT signing keys):
   a. Implement a grace period where both old and new keys are accepted.
   b. Rotate the signing key, deploy to the issuer, wait, then deploy to all verifiers, then remove the old key.
6. Set appropriate rotation intervals per secret type.
7. Document the rotation procedure for each secret type.

## Validation
Trigger rotation for each secret type and verify: new credentials are provisioned, Kubernetes Secrets are updated, application pods use the new credentials successfully, and old credentials are revoked. Confirm no 401/403 errors during or after rotation. Verify external API calls succeed with the new key.
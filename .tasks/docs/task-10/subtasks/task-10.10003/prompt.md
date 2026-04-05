Implement subtask 10003: Automate secret rotation for API keys and service tokens

## Objective
Implement automated rotation for application-level API keys, inter-service tokens, and third-party API credentials, ensuring all consuming services are updated seamlessly.

## Steps
1. Inventory all API keys and service tokens used across services (e.g., OpenAI API key, inter-service auth tokens, external API keys).
2. Create ExternalSecret resources for each, syncing from the external secret store.
3. For inter-service tokens (e.g., JWT signing keys), implement a dual-key rotation strategy: both old and new keys are valid during a transition period.
4. Create a CronJob or operator-triggered rotation workflow for each key type.
5. For third-party API keys that can't be auto-rotated, create alerting when manual rotation is due.
6. Ensure all services reference these secrets via Kubernetes Secret mounts, not hardcoded values.

## Validation
Trigger rotation of an inter-service token. Verify both old and new tokens are accepted during the transition window. After transition, verify only the new token works. Verify third-party key rotation alerts fire at the configured interval.
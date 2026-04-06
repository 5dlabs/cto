Implement subtask 10003: Implement automated secret rotation for Redis credentials and external API keys

## Objective
Configure automated rotation of Redis passwords and external API keys (OpenAI, etc.) with zero-downtime rollover for consuming services.

## Steps
1. For Redis: Update the Redis/Valkey CR or Helm values to reference a Kubernetes Secret for the `requirepass` value. Create a CronJob that generates a new password, updates the Redis Secret, and triggers a Redis config reload (Redis supports `CONFIG SET requirepass` without restart). 2. Configure Reloader to watch the Redis secret and trigger rolling restarts for backend services that connect to Redis. 3. For external API keys (OpenAI, etc.): Store each API key in its own Kubernetes Secret. Create documentation for the manual rotation process (generate new key in provider dashboard, update the Secret, Reloader restarts consumers). 4. If using External Secrets Operator (ESO), configure SecretStore and ExternalSecret resources pointing to a vault or cloud secret manager for automated sync. 5. Test rotation for each secret type independently.

## Validation
Rotate the Redis password: update the secret, verify Redis accepts the new password, verify Reloader restarts dependent services, confirm zero dropped connections during the process. For API keys: update an API key secret, verify the consuming service picks up the new key and successfully authenticates with the external provider. Verify the old key no longer works in the application.
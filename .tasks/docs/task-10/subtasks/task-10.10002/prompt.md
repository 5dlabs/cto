Implement subtask 10002: Automate secret rotation for all external service credentials

## Objective
Set up automated secret rotation for external API credentials (Stripe, LinkedIn, Cloudflare, etc.) using a Kubernetes secret management operator, ensuring zero-downtime rotation.

## Steps
1. Choose and deploy the secret management solution (e.g., External Secrets Operator with a backing store, or Sealed Secrets with rotation pipeline).
2. Inventory all external credentials that need rotation: Stripe API keys, LinkedIn API credentials, Cloudflare API tokens, Signal-CLI credentials, any OAuth tokens.
3. For each credential, create an ExternalSecret (or equivalent) CR that syncs from the external store to a Kubernetes Secret.
4. Configure rotation schedules (e.g., refreshInterval in ESO) appropriate to each credential type.
5. Ensure application deployments reference secrets via `envFrom` or volume mounts so rotated secrets are picked up (may require pod restart strategy or file-watch).
6. Implement a rolling restart annotation or signal mechanism so pods pick up rotated secrets without full downtime.
7. Test rotation by manually rotating a credential in the backing store and verifying the Kubernetes Secret updates and the application continues operating.

## Validation
Rotate a test credential in the external store; verify the Kubernetes Secret updates within the configured refresh interval; verify the application pod picks up the new credential (via restart or file watch) without service interruption; confirm old credential is no longer present in the Secret.
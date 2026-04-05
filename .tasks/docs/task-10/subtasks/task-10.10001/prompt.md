Implement subtask 10001: Generate RSA-256 key pair and store as ExternalSecret for JWT signing

## Objective
Create the RSA-256 key pair that will be used for service-to-service JWT token signing. Store the private key as an ExternalSecret `sigma1-jwt-signing-key` and distribute the public key via ConfigMap `sigma1-jwt-public-key` so all services can verify tokens.

## Steps
1. Generate a 2048-bit RSA key pair using openssl or a Kubernetes Job manifest.
2. Define an ExternalSecret CR `sigma1-jwt-signing-key` that syncs the private key from the external secrets backend (e.g., Cloudflare or Vault) into a Kubernetes Secret in the sigma1 namespace.
3. Create a ConfigMap `sigma1-jwt-public-key` containing the PEM-encoded public key.
4. Ensure the ConfigMap is referenced by all service Deployments so they can verify JWT signatures.
5. Add labels and annotations for secret management tracking (e.g., rotation-schedule, managed-by).

## Validation
Verify the ExternalSecret CR syncs successfully (status condition Ready=True). Verify the `sigma1-jwt-public-key` ConfigMap exists and contains a valid PEM public key. Verify the private key Secret exists and is not empty. Confirm the public key can verify a test signature made with the private key.
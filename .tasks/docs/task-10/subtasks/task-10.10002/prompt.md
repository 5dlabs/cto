Implement subtask 10002: Create deploy-time Job for JWT service token generation for all 6 services

## Objective
Implement a Kubernetes Job that runs at deploy time, reads the JWT signing private key, and generates service tokens for all 6 services (equipment-catalog, rms, finance, customer-vetting, social-engine, morgan) with appropriate claims and 90-day expiry, storing each as a Kubernetes Secret.

## Steps
1. Create a Job manifest `jwt-token-generator` that runs a lightweight container (alpine + openssl/jwt-cli or a small Go/Rust binary).
2. The Job mounts the `sigma1-jwt-signing-key` Secret.
3. For each service, generate a JWT with: `sub` = service name, `roles` = ['service'] (or ['morgan-agent'] for morgan), `iat` = now, `exp` = now + 90 days, `iss` = 'sigma1-token-issuer'.
4. Store each token as a separate Kubernetes Secret: `equipment-catalog-token`, `rms-token`, `finance-token`, `vetting-token`, `social-engine-token`, `morgan-token`.
5. Use `kubectl create secret` or the Kubernetes API from within the Job (requires appropriate RBAC — ServiceAccount with Secret create/update permissions).
6. Create the ServiceAccount and Role/RoleBinding for the Job.
7. Configure the Job with `restartPolicy: OnFailure`, `backoffLimit: 3`.
8. Update all service Deployments to mount their respective token Secret as an env var (e.g., `SERVICE_JWT_TOKEN`).

## Validation
Run the Job manually, verify all 6 Secrets are created with non-empty data. Decode each JWT and confirm: correct `sub` claim, correct `roles`, `exp` is ~90 days in the future, signature verifies against the public key from `sigma1-jwt-public-key` ConfigMap.
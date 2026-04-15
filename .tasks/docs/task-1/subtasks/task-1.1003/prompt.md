<identity>
You are bolt working on subtask 1003 of task 1.
</identity>

<context>
<scope>
Generate devnet operator and treasury keypairs, store them as Kubernetes Secrets using the existing OpenBao pipeline, and make secret paths available for reference.
</scope>
</context>

<implementation_plan>
1. Use `solana-keygen new` to generate two keypairs: operator (for signing settle/pause transactions) and treasury (for receiving protocol fees). 2. Store keypair JSON files as Kubernetes Secrets following the pattern in docs/secrets-management.md via the OpenBao pipeline. 3. Name secrets `cto-billing-operator-keypair` and `cto-billing-treasury-keypair`. 4. Document the secret mount paths that will be referenced by the ConfigMap (e.g., /secrets/operator-keypair.json, /secrets/treasury-keypair.json). 5. Verify secrets are accessible from within pods in the dev namespace.
</implementation_plan>

<validation>
Kubernetes Secrets `cto-billing-operator-keypair` and `cto-billing-treasury-keypair` exist in the dev namespace. Secrets can be mounted into a test pod and read as valid Solana keypair JSON. Public keys derived from both keypairs are valid base58 Solana addresses.
</validation>
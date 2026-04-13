<identity>
You are bolt working on subtask 1003 of task 1.
</identity>

<context>
<scope>
Create the 1Password item for the operator keypair, configure OpenBao sync, and deploy the ExternalSecret CR that materializes the K8s Secret cto-pay-operator-keypair in the cto namespace.
</scope>
</context>

<implementation_plan>
1. Create a 1Password item named `cto-pay-operator-keypair` in the appropriate vault. The item should contain the Solana keypair JSON (byte array format, e.g., `[174,47,...]` — 64 integers representing the 32-byte private key + 32-byte public key).
2. Configure OpenBao to sync this 1Password item. Follow the existing CTO pipeline patterns for 1Password → OpenBao synchronization.
3. Create an `ExternalSecret` CR YAML at `infra/gitops/cto-pay-operator-keypair-externalsecret.yaml`:
   ```yaml
   apiVersion: external-secrets.io/v1beta1
   kind: ExternalSecret
   metadata:
     name: cto-pay-operator-keypair
     namespace: cto
   spec:
     refreshInterval: 1h
     secretStoreRef:
       name: openbao-backend
       kind: ClusterSecretStore
     target:
       name: cto-pay-operator-keypair
       creationPolicy: Owner
     data:
       - secretKey: operator-keypair.json
         remoteRef:
           key: cto-pay-operator-keypair
           property: keypair
   ```
4. Apply the ExternalSecret and verify the K8s Secret is created with key `operator-keypair.json`.
5. Document the volume mount path `/secrets/operator-keypair.json` for downstream pod specs.
</implementation_plan>

<validation>
Run `kubectl get externalsecret cto-pay-operator-keypair -n cto` — status shows `SecretSynced`. Run `kubectl get secret cto-pay-operator-keypair -n cto -o jsonpath='{.data.operator-keypair\.json}'` — base64-decoded value is a valid JSON array of 64 integers. Verify the secret refreshes on schedule (check ExternalSecret status conditions).
</validation>
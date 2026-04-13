<identity>
You are bolt working on subtask 1004 of task 1.
</identity>

<context>
<scope>
Deploy the cto-pay-infra-endpoints ConfigMap with all 5 required keys and a separate ExternalSecret for the Helius API key, ensuring sensitive values are not inlined in the ConfigMap.
</scope>
</context>

<implementation_plan>
1. Create a 1Password item `cto-pay-helius-api-key` containing the Helius devnet API key.
2. Configure OpenBao sync for this item.
3. Create an ExternalSecret CR at `infra/gitops/cto-pay-helius-api-key-externalsecret.yaml` that creates K8s Secret `cto-pay-helius-api-key` with key `HELIUS_API_KEY`.
4. Create the ConfigMap YAML at `infra/gitops/cto-pay-infra-endpoints-configmap.yaml`:
   ```yaml
   apiVersion: v1
   kind: ConfigMap
   metadata:
     name: cto-pay-infra-endpoints
     namespace: cto
   data:
     SEAWEEDFS_ENDPOINT: "http://seaweedfs-s3.cto.svc.cluster.local:8333"
     SEAWEEDFS_BUCKET: "cto-pay-receipts"
     SOLANA_RPC_URL: "https://devnet.helius-rpc.com"
     SOLANA_OPERATOR_KEYPAIR_PATH: "/secrets/operator-keypair.json"
     CTO_PAY_ENABLED: "false"
   ```
   Note: SOLANA_RPC_URL in ConfigMap does NOT contain the API key. The controller must read the Helius API key from the Secret and append it as a query parameter at runtime.
5. Apply both resources and verify all keys are present.
6. Document the expected pod spec pattern: `envFrom: [{configMapRef: {name: cto-pay-infra-endpoints}}, {secretRef: {name: cto-pay-helius-api-key}}]` plus volume mount for operator keypair.
</implementation_plan>

<validation>
Run `kubectl get configmap cto-pay-infra-endpoints -n cto -o yaml` — contains all 5 keys with correct values. Run `kubectl get secret cto-pay-helius-api-key -n cto` — exists and contains key `HELIUS_API_KEY`. Verify SOLANA_RPC_URL in ConfigMap does NOT contain an API key substring. Verify ExternalSecret status is SecretSynced.
</validation>
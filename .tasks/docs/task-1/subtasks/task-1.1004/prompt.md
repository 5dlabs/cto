<identity>
You are bolt working on subtask 1004 of task 1.
</identity>

<context>
<scope>
Create the ConfigMap cto-billing-infra-endpoints containing SOLANA_RPC_URL, SOLANA_WS_URL, IRYS_RPC_URL, IRYS_TOKEN, OPERATOR_KEYPAIR_PATH, and TREASURY_KEYPAIR_PATH keys.
</scope>
</context>

<implementation_plan>
1. Create a ConfigMap manifest `cto-billing-infra-endpoints` in the infra/solana-dev/ Helm chart templates. 2. Set `SOLANA_RPC_URL` to `http://solana-validator:8899` (pointing to the validator service from subtask 1002). 3. Set `SOLANA_WS_URL` to `ws://solana-validator:8900`. 4. Set `IRYS_RPC_URL` to the Irys devnet endpoint (https://devnet.irys.xyz). 5. Set `IRYS_TOKEN` to `solana`. 6. Set `OPERATOR_KEYPAIR_PATH` and `TREASURY_KEYPAIR_PATH` to the secret mount paths from subtask 1003. 7. Apply and verify ConfigMap exists with all expected keys.
</implementation_plan>

<validation>
ConfigMap `cto-billing-infra-endpoints` exists in dev namespace. Contains all 6 keys: SOLANA_RPC_URL, SOLANA_WS_URL, IRYS_RPC_URL, IRYS_TOKEN, OPERATOR_KEYPAIR_PATH, TREASURY_KEYPAIR_PATH. Values are non-empty and correctly reference the validator service and secret mount paths.
</validation>
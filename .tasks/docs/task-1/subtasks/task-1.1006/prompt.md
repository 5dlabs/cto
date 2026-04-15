<identity>
You are bolt working on subtask 1006 of task 1.
</identity>

<context>
<scope>
Create Makefile/justfile with all build targets, .env.example, env-setup.sh, and verify the entire bootstrap works end-to-end (anchor build, anchor test --skip-deploy, validator pod health).
</scope>
</context>

<implementation_plan>
1. Create `Makefile` (or `justfile`) at workspace root with targets: `build` (anchor build), `test` (anchor test), `deploy-devnet` (anchor deploy --provider.cluster devnet), `localnet-up` (helm install or kubectl apply for validator), `localnet-down` (helm uninstall or kubectl delete). 2. Create `.env.example` documenting all required environment variables with descriptions and example values: SOLANA_RPC_URL, SOLANA_WS_URL, IRYS_RPC_URL, IRYS_TOKEN, OPERATOR_KEYPAIR_PATH, TREASURY_KEYPAIR_PATH, USDC_MINT. 3. Create `scripts/env-setup.sh` that sources the ConfigMap values (or reads .env) and exports them to the shell. 4. Run full verification: `anchor build` succeeds, `anchor test --skip-deploy` initializes without error, validator pod is healthy, ConfigMap is populated, USDC script works. 5. Document the bootstrap sequence in a README.md at workspace root.
</implementation_plan>

<validation>
Makefile/justfile exists with all 5 targets (build, test, deploy-devnet, localnet-up, localnet-down). `.env.example` contains all documented variables. `make build` (or `just build`) runs `anchor build` successfully. `make test` runs `anchor test --skip-deploy` without errors. Full end-to-end: localnet-up → setup-devnet-usdc → anchor build → anchor test --skip-deploy all pass sequentially.
</validation>
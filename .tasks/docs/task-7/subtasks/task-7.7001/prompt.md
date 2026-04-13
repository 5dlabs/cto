<identity>
You are cipher working on subtask 7001 of task 7.
</identity>

<context>
<scope>
Run anchor verify against the deployed program ID to cryptographically confirm that the on-chain bytecode was compiled from the audited source tree. This is the foundational trust prerequisite — all subsequent analysis is meaningless if source does not match deployment.
</scope>
</context>

<implementation_plan>
1. Ensure the exact Anchor and Solana CLI versions used for deployment are installed (check Anchor.toml and rust-toolchain.toml). 2. Run `anchor verify <PROGRAM_ID> --provider-url <RPC_URL>` targeting the relevant cluster (devnet or mainnet). 3. If verification fails, document the mismatch: compare build hashes, check for non-deterministic compilation factors (Solana SDK version drift, feature flags). 4. Record the verified commit SHA, Anchor version, and Solana CLI version as audit metadata. 5. If anchor verify is not feasible (e.g., no deployment yet), document this gap and note that bytecode verification must occur before any mainnet deployment.
</implementation_plan>

<validation>
Verification passes: `anchor verify` exits 0 with a matching hash. If it fails, the mismatch is documented with root cause. Audit metadata file contains commit SHA, toolchain versions, and verification timestamp.
</validation>
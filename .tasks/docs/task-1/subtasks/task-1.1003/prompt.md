<identity>
You are rex working on subtask 1003 of task 1.
</identity>

<context>
<scope>
Create the utils/ module with the SHA-256 hash_seed helper for PDA derivation and define the program vault token account PDA.
</scope>
</context>

<implementation_plan>
1. Create `utils/mod.rs` with `pub fn hash_seed(input: &str) -> [u8; 32]` that computes SHA-256 of the input string and returns the 32-byte digest. Use the `sha2` crate: `use sha2::{Sha256, Digest}; let mut hasher = Sha256::new(); hasher.update(input.as_bytes()); hasher.finalize().into()`.
2. Add vault PDA seed constants: `pub const VAULT_SEED: &[u8] = b"vault";` — the full seed is `[VAULT_SEED, mint.key().as_ref()]`.
3. Add all PDA seed constants for consistency: `pub const OPERATOR_CONFIG_SEED: &[u8] = b"operator_config";`, `pub const CUSTOMER_BALANCE_SEED: &[u8] = b"customer_balance";`, `pub const AGENT_PACKAGE_SEED: &[u8] = b"agent_package";`, `pub const TASK_RECEIPT_SEED: &[u8] = b"task_receipt";`.
4. Add a `find_vault_authority` helper that derives the vault authority PDA if needed, or document the vault PDA signing pattern using `[VAULT_SEED, mint_key, &[bump]]`.
5. Verify the hash_seed function produces deterministic 32-byte output.
</implementation_plan>

<validation>
Write a unit test in the utils module: call hash_seed with a known input string and verify the output matches the expected SHA-256 digest. Verify hash_seed("test") produces the same output on repeated calls. Verify `anchor build` compiles the utils module.
</validation>
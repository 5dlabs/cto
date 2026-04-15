<identity>
You are rex working on subtask 5001 of task 5.
</identity>

<context>
<scope>
Add all required Solana ecosystem crates to the controller's Cargo.toml so that subsequent billing module subtasks can compile. This includes solana-sdk, solana-client, anchor-client (or equivalent), and sha2 for receipt hashing.
</scope>
</context>

<implementation_plan>
1. Open `crates/controller/Cargo.toml`.
2. Add under `[dependencies]`:
   - `solana-sdk = "1.18"` — keypair loading, transaction types, Pubkey, Signature.
   - `solana-client = "1.18"` — RpcClient for sending transactions.
   - `anchor-client = "0.30"` — Anchor IDL-based instruction building (or omit if using raw RPC and add `borsh = "1"` for manual serialization).
   - `sha2 = "0.10"` — SHA-256 hashing for receipt verification.
3. Add a `billing` feature flag in `[features]`: `billing = ["dep:solana-sdk", "dep:solana-client", "dep:anchor-client", "dep:sha2"]` to allow conditional compilation.
4. Run `cargo check -p controller --features billing` to verify dependency resolution and no version conflicts with existing workspace crates.
5. If anchor-client pulls in conflicting tokio or borsh versions, pin compatible versions or switch to raw RPC approach.
</implementation_plan>

<validation>
Run `cargo check -p controller --features billing` — resolves all dependencies with zero errors. Run `cargo tree -p controller` — confirms solana-sdk 1.18.x, solana-client 1.18.x, sha2 0.10.x are in the dependency tree. Without the `billing` feature, `cargo check -p controller` still passes (no regressions).
</validation>
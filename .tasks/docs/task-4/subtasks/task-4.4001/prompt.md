<identity>
You are rex working on subtask 4001 of task 4.
</identity>

<context>
<scope>
Add the solana-billing feature flag to the controller Cargo.toml with optional dependencies, create the SolanaBillingConfig struct, and add the customer_solana_pubkey field to the customer profile or CodeRun annotation type.
</scope>
</context>

<implementation_plan>
1. Edit `crates/controller/Cargo.toml`:
   - Add feature section: `[features]` with `solana-billing = ["dep:redis", "dep:serde_json", "dep:sha2"]`.
   - Add optional dependencies: `redis = { version = "0.25", features = ["tokio-comp", "streams"], optional = true }`, `sha2 = { version = "0.10", optional = true }`. Note: serde_json likely already exists as a non-optional dep; if so, don't duplicate.
   - Verify the feature defaults to false (not in `default = [...]`).

2. Create `crates/controller/src/billing/mod.rs` with `#[cfg(feature = "solana-billing")]` gating the entire module. Add `pub mod event; pub mod receipt; pub mod publisher;` declarations (files created in subsequent subtasks).

3. Add `#[cfg(feature = "solana-billing")] pub mod billing;` to the controller's `lib.rs` or `main.rs`.

4. Create `SolanaBillingConfig` in `crates/controller/src/tasks/config.rs` (or `billing/config.rs`):
   ```rust
   #[cfg(feature = "solana-billing")]
   #[derive(Clone, Debug, Deserialize)]
   pub struct SolanaBillingConfig {
       pub redis_url: String,
       pub receipt_s3_bucket: String,
       pub receipt_s3_prefix: String,
       pub enabled: bool,
   }
   ```
   Add this as an optional field on the main controller config struct, gated behind the feature.

5. Add `customer_solana_pubkey: Option<String>` to the relevant customer profile type or as a recognized CodeRun annotation key (e.g., `cto.ai/customer-solana-pubkey`). This is a simple String for the hackathon.

6. Verify: `cargo build` (no feature) compiles cleanly. `cargo build --features solana-billing` compiles (will have unused module warnings until schemas are added, which is fine).
</implementation_plan>

<validation>
Run `cargo build` without features — compiles with zero errors and no references to redis/sha2. Run `cargo build --features solana-billing` — compiles (warnings acceptable for empty modules). Verify `solana-billing` is not in the default features list. Grep the built binary or check cargo output to confirm optional deps are only included with the feature.
</validation>
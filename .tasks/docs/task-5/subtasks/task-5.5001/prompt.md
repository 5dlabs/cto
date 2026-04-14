<identity>
You are rex working on subtask 5001 of task 5.
</identity>

<context>
<scope>
Create the new Rust crate at crates/settlement-sidecar/ with Cargo.toml, main.rs entry point, and SidecarConfig struct that loads all settings from environment variables.
</scope>
</context>

<implementation_plan>
1. Create `crates/settlement-sidecar/Cargo.toml` with dependencies: solana-sdk 1.18.x, solana-client, redis (with `tokio-comp` feature), tokio (full features), serde/serde_json, sha2, tracing, tracing-subscriber (with json feature). Add workspace membership if applicable.
2. Implement `src/config.rs` with `SidecarConfig` struct containing all fields: solana_rpc_url, operator_keypair_path, program_id, redis_url, stream_name, consumer_group, consumer_name, max_retries, retry_base_delay_ms, confirm_timeout_secs.
3. Implement `SidecarConfig::from_env()` that reads each field from environment variables (e.g., `SOLANA_RPC_URL`, `OPERATOR_KEYPAIR_PATH`, etc.) with sensible defaults for stream_name (`cto:settlements`), consumer_group (`settlement-sidecar`), max_retries (5), retry_base_delay_ms (1000), confirm_timeout_secs (30). Use hostname for consumer_name default.
4. Implement operator keypair loading from the JSON file path specified in config.
5. Create `src/main.rs` with tokio::main that loads config, initializes tracing, and has placeholder calls for consumer startup.
6. Define `SettlementEvent` struct in `src/types.rs` with serde Deserialize: task_id, customer (Pubkey), amount (u64), receipt_hash ([u8; 32]), quality_met (bool), agent_package_id (Option<String>), agent_author (Option<Pubkey>), event_type (settle_task | refund_task).
</implementation_plan>

<validation>
Run `cargo check -p settlement-sidecar` — compiles without errors. Unit test: set environment variables programmatically, call SidecarConfig::from_env(), verify all fields populated correctly. Unit test: deserialize a known SettlementEvent JSON string and verify all fields match expected values.
</validation>
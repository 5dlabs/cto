## Settlement Sidecar Service (Rex - Rust/solana-sdk)

### Objective
Build the thin settlement sidecar — an independent Rust binary that reads settlement events from the Redis stream and submits corresponding Solana transactions. This fully decouples the CTO controller from Solana RPC (D5).

### Ownership
- Agent: rex
- Stack: Rust/solana-sdk/solana-client
- Priority: high
- Status: pending
- Dependencies: 1, 2, 4

### Implementation Details
1. Create new crate at `crates/settlement-sidecar/` with dependencies: solana-sdk (1.18.x), solana-client, anchor-client (or construct instructions manually from IDL), redis (async), tokio, serde_json, sha2, tracing.

2. Configuration (`src/config.rs`):
   ```rust
   pub struct SidecarConfig {
       pub solana_rpc_url: String,       // https://api.devnet.solana.com
       pub operator_keypair_path: String, // /secrets/operator-keypair.json (mounted via ESO — D6)
       pub program_id: Pubkey,
       pub redis_url: String,
       pub stream_name: String,          // cto:settlements
       pub consumer_group: String,       // settlement-sidecar
       pub consumer_name: String,        // pod hostname
       pub max_retries: u32,             // 5
       pub retry_base_delay_ms: u64,     // 1000
       pub confirm_timeout_secs: u64,    // 30
   }
   ```
   Load from environment variables for K8s deployment.

3. Redis stream consumer (`src/consumer.rs`):
   - Create consumer group on startup (XGROUP CREATE, ignore if exists).
   - Main loop: XREADGROUP with BLOCK 5000ms.
   - Parse SettlementEvent from each message.
   - On successful Solana submission: XACK the message.
   - On failure after max retries: log error, move to dead-letter stream `cto:settlements:dlq` via XADD, then XACK original.
   - On startup, process any pending messages (XREADGROUP with ID '0') before reading new ones.

4. Transaction builder (`src/transaction.rs`):
   - Derive all required PDAs using the same SHA-256 seed logic as the on-chain program.
   - For `settle_task`: build instruction with all required accounts (operator, operator_config, customer_balance, vault, treasury_ata, task_receipt, clock sysvar, token_program). Conditionally include agent_package and author_ata accounts.
   - For `refund_task`: similar pattern.
   - Use `RpcClient::get_latest_blockhash()` with caching (refresh every 30 seconds or on BlockhashNotFound).
   - Sign with operator keypair.

5. Retry and submission logic (`src/submitter.rs`):
   - Submit via `send_and_confirm_transaction_with_spinner_and_config`.
   - On `BlockhashNotFound`: refresh blockhash, rebuild transaction, retry.
   - On `InsufficientFunds` (SOL for fees): alert and halt (operator needs to fund the wallet).
   - On program-level errors (InsufficientBalance, ExceedsPerTaskCap, etc.): do NOT retry — log the error, write to DLQ with error details.
   - Exponential backoff: 1s, 2s, 4s, 8s, 16s (capped at max_retries).

6. Health and observability:
   - HTTP health endpoint at `:8080/healthz` (returns 200 if Redis connected and last Solana RPC check succeeded).
   - Metrics: events_processed, events_failed, solana_rpc_errors, retry_count (as tracing structured logs, or Prometheus metrics if time permits).
   - Structured JSON logging via tracing-subscriber.

7. Build as a standalone binary: `cargo build --release --bin settlement-sidecar`. Dockerfile: `FROM rust:1.78 as builder` → `FROM debian:bookworm-slim` with only the binary and CA certificates.

8. Integration test in `tests/integration.rs`:
   - Start a local validator.
   - Deploy the cto-billing program.
   - Push a SettlementEvent to a local Redis stream.
   - Verify the sidecar picks it up, submits the transaction, and ACKs the message.
   - Verify the TaskReceipt PDA exists on-chain with correct data.

### Subtasks
- [ ] Scaffold sidecar crate and implement configuration loading: Create the new Rust crate at crates/settlement-sidecar/ with Cargo.toml, main.rs entry point, and SidecarConfig struct that loads all settings from environment variables.
- [ ] Implement Redis stream consumer with consumer group, pending recovery, and DLQ routing: Build the Redis stream consumer module that creates consumer groups, reads messages via XREADGROUP with blocking, recovers pending messages on startup, and routes failed messages to a dead-letter queue.
- [ ] Implement PDA derivation and Solana transaction builder for settle_task and refund_task: Build the transaction construction module that derives all required PDAs using SHA-256 seed logic matching the on-chain program and constructs settle_task and refund_task instructions with the correct account lists.
- [ ] Implement transaction submission with retry logic, error classification, and exponential backoff: Build the submitter module that sends signed Solana transactions with exponential backoff retry, classifies errors as retryable vs. terminal, and routes terminal failures to the DLQ.
- [ ] Implement HTTP health endpoint and structured JSON logging: Add an HTTP /healthz endpoint on port 8080 and configure structured JSON logging with tracing-subscriber, including metric counters for key events.
- [ ] Create multi-stage Dockerfile and end-to-end integration test: Build the multi-stage Docker image for the settlement sidecar and write an integration test that verifies the full pipeline: Redis event → Solana transaction → on-chain TaskReceipt.
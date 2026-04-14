<identity>
You are rex working on subtask 5006 of task 5.
</identity>

<context>
<scope>
Build the multi-stage Docker image for the settlement sidecar and write an integration test that verifies the full pipeline: Redis event → Solana transaction → on-chain TaskReceipt.
</scope>
</context>

<implementation_plan>
1. Create `crates/settlement-sidecar/Dockerfile`:
   - Stage 1 (`builder`): `FROM rust:1.78-bookworm`, copy workspace Cargo.toml and crate source, `cargo build --release --bin settlement-sidecar`.
   - Stage 2 (`runtime`): `FROM debian:bookworm-slim`, install `ca-certificates` and `libssl3` only, copy binary from builder, set `ENTRYPOINT ["./settlement-sidecar"]`.
   - Ensure the image is minimal (< 100MB).
2. Add `.dockerignore` excluding target/, .git/, tests/, etc.
3. Create `tests/integration.rs` integration test:
   a. Prerequisites check: verify `solana-test-validator` and `redis-server` are available (skip test with clear message if not).
   b. Start `solana-test-validator` with the deployed cto-billing program (use `--bpf-program` flag with the built .so from Task 2).
   c. Start a local Redis instance (or require it running on 6379).
   d. Set up on-chain state: create operator config, customer balance with deposit, agent package — all via direct RPC calls using solana-sdk.
   e. Push a `SettlementEvent` JSON to the `cto:settlements` Redis stream via XADD.
   f. Start the sidecar binary as a subprocess with appropriate env vars pointing to local validator and Redis.
   g. Poll for the TaskReceipt PDA on-chain (up to 15 seconds). Verify it exists with correct task_id, amount, quality_met, and receipt_hash.
   h. Verify the Redis message was ACKed (pending count = 0 for the consumer group).
   i. Clean up: kill sidecar subprocess, stop validator.
4. Add a second integration test case: push an event that will cause a program error (e.g., duplicate task_id). Verify the message ends up in the DLQ stream.
5. Ensure `cargo test --test integration` runs the integration tests (with `#[ignore]` attribute so `cargo test` without flags skips them). Add a note in README for running with `--ignored` flag.
</implementation_plan>

<validation>
Run `docker build -t settlement-sidecar .` from the crate directory — build completes successfully. Run `docker image inspect` — verify image size < 100MB. Run `cargo test --test integration -- --ignored` with local validator and Redis running: (1) Happy path test completes in < 30 seconds — TaskReceipt exists on-chain with correct data, Redis message ACKed. (2) DLQ test — duplicate task_id event results in message in cto:settlements:dlq stream. Both tests pass with exit code 0.
</validation>
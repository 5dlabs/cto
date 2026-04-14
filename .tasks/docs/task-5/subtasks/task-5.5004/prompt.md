<identity>
You are rex working on subtask 5004 of task 5.
</identity>

<context>
<scope>
Build the submitter module that sends signed Solana transactions with exponential backoff retry, classifies errors as retryable vs. terminal, and routes terminal failures to the DLQ.
</scope>
</context>

<implementation_plan>
1. Implement `src/submitter.rs` with `TransactionSubmitter` struct holding RpcClient, max_retries, retry_base_delay_ms, and a reference to the BlockhashCache from the transaction builder.
2. Implement `submit_with_retry()` that takes a built Transaction and SettlementEvent context:
   a. Attempt `send_and_confirm_transaction_with_spinner_and_config` with confirmed commitment.
   b. On success: return Ok(signature).
   c. On `BlockhashNotFound`: call `blockhash_cache.force_refresh()`, rebuild the transaction with new blockhash, decrement retry but don't count as a 'real' failure.
   d. On `InsufficientFunds` (SOL for fees): return a special `OperatorFundingRequired` error — caller should halt processing and alert.
   e. On program-level errors (parse the transaction logs for 'InsufficientBalance', 'ExceedsPerTaskCap', 'AlreadySettled', etc.): return `ProgramError` — these are NOT retryable.
   f. On transient RPC errors (timeout, connection refused): apply exponential backoff and retry.
3. Exponential backoff: delay = base_delay * 2^attempt, capped at max_retries. Use `tokio::time::sleep`.
4. Implement error classification function `classify_error()` that inspects `ClientError` / `TransactionError` and returns an enum: Retryable, ProgramError(String), BlockhashExpired, InsufficientFees.
5. Wire into the consumer callback: on Ok → XACK; on ProgramError → DLQ with error details; on InsufficientFees → log critical alert and pause consumer; on max retries exhausted → DLQ.
6. Log every attempt with tracing: attempt number, delay, error details, transaction signature on success.
</implementation_plan>

<validation>
Unit tests: (1) Mock RPC client that fails 3 times with timeout then succeeds — verify submit_with_retry returns Ok after 4 attempts with correct exponential delays (1s, 2s, 4s). (2) Mock RPC client returning BlockhashNotFound — verify blockhash refresh is triggered and transaction is rebuilt. (3) Mock RPC client returning a program error — verify no retry occurs and ProgramError is returned immediately. (4) Mock RPC client returning InsufficientFunds — verify OperatorFundingRequired error is returned. (5) Mock RPC client failing max_retries+1 times — verify final error returned for DLQ routing.
</validation>
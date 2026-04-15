<identity>
You are rex working on subtask 5007 of task 5.
</identity>

<context>
<scope>
Hook the billing module into the existing CodeRun controller reconcile function so that terminal-state transitions trigger billing computation, receipt upload, and Solana settlement or refund, with proper feature flagging, error handling, and status annotation.
</scope>
</context>

<implementation_plan>
1. In `crates/controller/src/tasks/code/controller.rs`, locate the reconcile function that handles CodeRun state transitions.
2. Add a post-transition hook after a CodeRun enters a terminal state (e.g., `Merged`, `Failed`, `Cancelled`):
   ```rust
   if billing_config.solana_billing_enabled {
       if let Some(customer_pubkey) = get_customer_wallet(&client, &namespace).await? {
           match terminal_state {
               Merged => {
                   let amount = compute_billable_amount(&coderun);
                   let receipt = build_receipt_json(&coderun, amount);
                   let (arweave_tx, hash) = upload_receipt(&billing_config, &receipt).await?;
                   let task = BillableTask { /* ... */ quality_met: true };
                   let sig = submit_settlement(&billing_config, &task, hash).await?;
                   annotate_coderun(&client, &coderun, "cto.dev/solana-tx", &sig.to_string()).await?;
                   annotate_coderun(&client, &coderun, "cto.dev/arweave-receipt", &arweave_tx).await?;
               },
               Failed | Cancelled => {
                   let sig = submit_refund(&billing_config, &task_id, &customer_pubkey).await?;
                   annotate_coderun(&client, &coderun, "cto.dev/solana-tx", &sig.to_string()).await?;
               }
           }
       }
   }
   ```
3. Error handling strategy:
   - Wrap the entire billing block in a catch-all that logs errors at `warn` level.
   - On transient errors (RPC timeout, upload timeout), add the CodeRun back to the requeue with exponential backoff.
   - On permanent errors (invalid program ID, deserialization failure), log at `error` and annotate CodeRun with `cto.dev/billing-error: <message>`. Do not requeue.
   - Never block or fail the main reconciliation — billing failures must not prevent CodeRun cleanup.
4. Add an idempotency check: if `cto.dev/solana-tx` annotation already exists, skip billing (prevents double-settlement on requeue).
5. Implement `async fn annotate_coderun(client: &kube::Client, coderun: &CodeRun, key: &str, value: &str) -> Result<()>` using a JSON merge patch on the CodeRun resource.
</implementation_plan>

<validation>
Unit test with mocked dependencies: simulate a CodeRun transitioning to Merged with billing enabled and a valid customer wallet — verify compute, upload, settle are called in order and annotation is set. Simulate billing disabled — verify no billing functions are called. Simulate upload_receipt failure — verify CodeRun is requeued and no Solana transaction is submitted. Simulate an already-annotated CodeRun — verify billing is skipped (idempotency). Integration test: deploy controller with billing enabled against devnet, create a CodeRun that transitions to Merged, verify the cto.dev/solana-tx annotation appears with a valid Solana signature.
</validation>
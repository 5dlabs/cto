Implement subtask 5007: Implement Stage 4: Credit Signals with CreditProvider trait and stub

## Objective
Design the CreditProvider trait for pluggable credit API implementations and build the initial stub implementation that returns UNKNOWN/unavailable credit data with appropriate logging.

## Steps
1. Create `src/stages/credit_signals.rs` module.
2. Define `CreditResult` struct: credit_score (Option<i32>), credit_available (bool), provider_name (String), raw_responses (serde_json::Value).
3. Define `CreditProvider` trait:
   ```rust
   #[async_trait]
   pub trait CreditProvider: Send + Sync {
       fn provider_name(&self) -> &str;
       async fn check_credit(&self, company_name: &str, domain: &str) -> StageResult<CreditResult>;
   }
   ```
4. Implement `StubCreditProvider` that logs a warning ("No credit provider configured, returning UNKNOWN") and returns CreditResult { credit_score: None, credit_available: false, provider_name: "stub" }.
5. Create a factory function `create_credit_provider(config: &AppConfig) -> Box<dyn CreditProvider>` that returns StubCreditProvider for now, but is designed for easy swap-in of a real provider.
6. Document the trait contract so future implementations (Dun & Bradstreet, Experian, CreditSafe) can be added by implementing the trait.
7. The stage runner function accepts `&dyn CreditProvider` and delegates to it.

## Validation
Unit test: StubCreditProvider returns credit_available=false and credit_score=None. Verify provider_name is 'stub'. Verify warning is logged (use tracing-test or capture logs). Verify trait is object-safe and can be used as Box<dyn CreditProvider>. Write a mock implementation of CreditProvider that returns a real score to verify the trait works end-to-end.
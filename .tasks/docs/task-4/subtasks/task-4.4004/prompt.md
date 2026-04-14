<identity>
You are rex working on subtask 4004 of task 4.
</identity>

<context>
<scope>
Implement the Redis stream publisher in billing/publisher.rs that publishes SettlementEvent JSON to the cto:settlements stream using XADD, with non-blocking fire-and-forget error handling.
</scope>
</context>

<implementation_plan>
1. Create `crates/controller/src/billing/publisher.rs`:
   ```rust
   use redis::AsyncCommands;
   use crate::billing::event::SettlementEvent;
   use crate::billing::BillingError;

   pub struct SettlementPublisher {
       client: redis::Client,
   }

   impl SettlementPublisher {
       pub fn new(redis_url: &str) -> Result<Self, BillingError> {
           let client = redis::Client::open(redis_url)
               .map_err(|e| BillingError::RedisPublish(e.to_string()))?;
           Ok(Self { client })
       }

       pub async fn publish(&self, event: &SettlementEvent) -> Result<String, BillingError> {
           let mut conn = self.client.get_multiplexed_async_connection().await
               .map_err(|e| BillingError::RedisPublish(e.to_string()))?;
           let payload = serde_json::to_string(event)?;
           let stream_id: String = redis::cmd("XADD")
               .arg("cto:settlements")
               .arg("*")
               .arg("event")
               .arg(&payload)
               .arg("task_id")
               .arg(&event.task_id)
               .arg("event_type")
               .arg(serde_json::to_string(&event.event_type)?)
               .query_async(&mut conn)
               .await
               .map_err(|e| BillingError::RedisPublish(e.to_string()))?;
           tracing::info!(stream_id = %stream_id, task_id = %event.task_id, "Settlement event published");
           Ok(stream_id)
       }
   }
   ```

2. Key design constraints:
   - The publisher must never panic — all Redis errors are caught and logged.
   - Use `get_multiplexed_async_connection()` for efficient connection reuse.
   - Include both the full JSON payload and individual indexed fields (task_id, event_type) in the XADD for consumer flexibility.
   - The returned stream_id serves as the `solana-settlement-event-id` annotation.

3. Add a convenience method `publish_fire_and_forget` that wraps `publish` in a tokio::spawn so it doesn't block the caller:
   ```rust
   pub fn publish_fire_and_forget(self: Arc<Self>, event: SettlementEvent) {
       tokio::spawn(async move {
           if let Err(e) = self.publish(&event).await {
               tracing::warn!(error = %e, task_id = %event.task_id, "Failed to publish settlement event");
           }
       });
   }
   ```

4. Write unit tests:
   - Mock Redis connection and verify XADD command is constructed with correct stream name and fields.
   - Test error handling: when Redis connection fails, BillingError::RedisPublish is returned with the error message.
   - If testcontainers is available, write an integration test: start Redis container, publish an event, XREAD it back, verify all fields.
</implementation_plan>

<validation>
Run `cargo test --features solana-billing` — tests pass for: (1) SettlementPublisher::new succeeds with valid Redis URL format, (2) publish correctly formats XADD with stream name 'cto:settlements' and includes task_id and event JSON fields, (3) publish returns BillingError::RedisPublish when connection fails. If testcontainers with Redis is used: publish an event, XREAD from 'cto:settlements', deserialize the event field back to SettlementEvent, and assert all fields match.
</validation>
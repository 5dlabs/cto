<identity>
You are rex working on subtask 5002 of task 5.
</identity>

<context>
<scope>
Build the Redis stream consumer module that creates consumer groups, reads messages via XREADGROUP with blocking, recovers pending messages on startup, and routes failed messages to a dead-letter queue.
</scope>
</context>

<implementation_plan>
1. Implement `src/consumer.rs` with a `StreamConsumer` struct holding an async Redis connection, stream_name, consumer_group, consumer_name, max_retries.
2. On `StreamConsumer::init()`: issue `XGROUP CREATE <stream> <group> 0 MKSTREAM` — catch and ignore the 'BUSYGROUP' error if group already exists.
3. Implement `recover_pending()`: call `XREADGROUP GROUP <group> <consumer> COUNT 100 STREAMS <stream> 0` to fetch any previously read but un-ACKed messages. Process each before entering the main loop.
4. Main loop in `consume()`: call `XREADGROUP GROUP <group> <consumer> BLOCK 5000 COUNT 10 STREAMS <stream> >`. Parse each message payload into a SettlementEvent. For each event, call a provided async callback (the transaction submission pipeline). On callback success: `XACK <stream> <group> <id>`. On callback failure with retries exhausted: write to DLQ.
5. Implement `send_to_dlq()`: `XADD cto:settlements:dlq * original_id <id> event <json> error <error_msg> failed_at <timestamp>`. Then XACK the original message.
6. Return parsed SettlementEvent and message ID from each read for downstream processing.
7. Ensure graceful shutdown on SIGTERM: finish processing current message, then exit the loop.
</implementation_plan>

<validation>
Unit test with a mock Redis (or real local Redis via testcontainers): (1) Verify consumer group creation is idempotent — call init() twice, no error. (2) Push 3 messages to stream, verify consume() yields all 3 SettlementEvents in order. (3) Simulate callback failure for max_retries, verify message appears in DLQ stream with error details and original is ACKed. (4) Push 2 messages, ACK only 1, restart consumer — verify pending recovery picks up the un-ACKed message.
</validation>
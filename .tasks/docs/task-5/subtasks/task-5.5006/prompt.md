<identity>
You are rex working on subtask 5006 of task 5.
</identity>

<context>
<scope>
After the pipeline completes and writes to vetting_results, serialize the result and store it in Valkey under key vetting:org:{org_id} with a 24-hour TTL. On POST /api/v1/vetting/run, check the cache first; if a cached result exists and is not stale, skip the pipeline and return the cached request_id.
</scope>
</context>

<implementation_plan>
In run_pipeline, after successful sqlx upsert of vetting_results, serialize the VettingResult struct to JSON using serde_json::to_string. Call redis SET vetting:org:{org_id} {json} EX 86400 using the redis::aio::ConnectionManager. In the POST /api/v1/vetting/run handler, before inserting a new vetting_request, call redis GET vetting:org:{org_id}. If Some(cached), deserialize and return 202 with a synthetic request_id noting cache hit (include 'cached': true in response body). Use ConnectionManager for multiplexed async access. Handle redis errors as non-fatal: log warning and fall through to full pipeline.
</implementation_plan>

<validation>
Integration test: call POST /api/v1/vetting/run for org_id A, let pipeline complete, then call POST again for same org_id. Verify via `redis-cli GET vetting:org:{id}` that the key exists with a TTL near 86400. Verify second POST completes immediately (< 200ms) without spawning a new pipeline (check vetting_requests table has only one row). Test redis unavailability: bring down redis, verify POST still returns 202 and pipeline runs.
</validation>
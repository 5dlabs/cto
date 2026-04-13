<identity>
You are rex working on subtask 4008 of task 4.
</identity>

<context>
<scope>
Implement the GET /api/v1/currency/rates endpoint returning cached rates from Valkey, and a tokio background task that fetches live rates from the chosen external API every 6 hours and stores them in both Valkey and the currency_rates table.
</scope>
</context>

<implementation_plan>
Create src/tasks/currency_sync.rs. Spawn a tokio::task::spawn loop at server startup: sleep 0 for first run, then repeat every 6 hours. On each run: fetch rates from the chosen external currency API (confirm provider via decision point). Parse response into a HashMap<String, f64> of target currency → rate relative to base USD. For each rate: UPSERT into finance.currency_rates (base, target, rate, fetched_at) ON CONFLICT (base, target) DO UPDATE. Serialize full rates map to JSON, call redis SET rates:v1 <json> EX 21600. GET /api/v1/currency/rates handler: try redis GET rates:v1; if hit, return parsed JSON; if miss, SELECT all from currency_rates and return. Register handler route.
</implementation_plan>

<validation>
After server start, wait for first sync cycle; GET /api/v1/currency/rates returns JSON with USD, CAD, AUD keys and non-zero values. Flush Valkey; GET /api/v1/currency/rates falls back to DB and returns same data. Currency_rates table has rows for USD, CAD, AUD after sync. Background task logs 'currency sync complete' every 6 hours.
</validation>
<identity>
You are angie working on subtask 7010 of task 7.
</identity>

<context>
<scope>
Send a real Signal test message to Morgan's number and validate round-trip receipt in pod logs. Also validate the web chat /chat endpoint returns a coherent reply.
</scope>
</context>

<implementation_plan>
1. Signal integration test: using a Signal-registered test account, send the message 'Test connectivity ping' to Morgan's configured phone number. Watch `kubectl logs -f -n openclaw deployment/morgan` for a log line containing 'Signal message received' within 30 seconds. Verify a reply appears in the Signal conversation within 60 seconds.
2. Web chat integration test: `kubectl port-forward svc/morgan-chat-svc 3000:3000 -n openclaw &`. Run: `curl -s -X POST http://localhost:3000/chat -H 'Content-Type: application/json' -d '{"message": "What lighting equipment do you offer?"}' | jq .`. Verify response contains reply field with non-empty string and session_id field.
3. Document both test outcomes in agents/morgan/test-results/integration-tests.md.
4. If Signal test fails: check signal-cli-svc pod logs for JSON-RPC connection errors, verify SIGNAL_PHONE_NUMBER secret value matches registered Signal number.
</implementation_plan>

<validation>
Morgan pod logs show Signal message receipt log within 30s of sending test message. Signal conversation shows Morgan reply within 60s. curl to /chat returns JSON {reply: <non-empty>, session_id: <non-empty>} with HTTP 200 within 10s.
</validation>
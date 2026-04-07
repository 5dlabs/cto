Implement subtask 7005: Configure Signal-CLI adapter in OpenClaw for Morgan

## Objective
Set up the OpenClaw Signal messaging adapter pointing to the signal-cli-svc JSON-RPC endpoint, with phone number from secret and receive loop enabled.

## Steps
1. In agents/morgan/agent.yaml (or a separate adapters/signal.yaml referenced from agent.yaml), add the signal adapter block:
   adapters:
     signal:
       enabled: true
       json_rpc_url: http://signal-cli-svc.signal.svc.cluster.local:7583
       phone_number: ${SIGNAL_PHONE_NUMBER}  # sourced from secret
       receive_loop: true
       reconnect_on_failure: true
       reconnect_interval_seconds: 10
2. Map the SIGNAL_PHONE_NUMBER env var from the existing Kubernetes secret (likely the same secret holding TWILIO_PHONE_NUMBER — confirm secret key name from Task 2 infra endpoints).
3. Configure message routing: all incoming Signal messages route to Morgan's main conversation handler.
4. Set max_message_length: 1500 to stay within Signal limits.
5. Document the adapter config in agents/morgan/README.md under 'Signal Integration'.

## Validation
After Morgan pod is running: send a test Signal message to the configured phone number from a registered Signal account. Verify `kubectl logs -n openclaw deployment/morgan` shows 'Signal message received' log line within 30 seconds. Verify Morgan's reply appears in the Signal conversation.
Implement subtask 6001: Deploy Morgan OpenClaw agent and configure LLM

## Objective
Deploy the `morgan` OpenClaw agent to the `openclaw` Kubernetes namespace and configure it to use `openai-api/gpt-5.4-pro` as its underlying language model.

## Steps
1. Create the `openclaw` Kubernetes namespace.2. Prepare the OpenClaw agent deployment YAML, specifying the `morgan` agent and `openai-api/gpt-5.4-pro` model.3. Apply the deployment and service manifests to the `openclaw` namespace.

## Validation
1. Verify the `morgan` agent pod is running in the `openclaw` namespace.2. Check agent logs for successful LLM configuration and initialization messages.
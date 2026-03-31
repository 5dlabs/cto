Implement subtask 7001: Deploy OpenClaw agent with AGENT_ID=morgan and gpt-5.4-pro model configuration

## Objective
Stand up the core OpenClaw agent instance, configure AGENT_ID=morgan, wire up the openai-api/gpt-5.4-pro model, and verify the agent boots and responds to a basic health check. Pull backend service endpoints from the project infra ConfigMap via envFrom.

## Steps
1. Create the OpenClaw agent configuration file specifying AGENT_ID=morgan and model=openai-api/gpt-5.4-pro.
2. Configure environment variables referencing the project infra-endpoints ConfigMap (envFrom) so all backend service URLs are available at runtime.
3. Set up the OpenAI API key as a Kubernetes secret and mount it into the agent pod.
4. Define the agent's system prompt / persona for Morgan (professional equipment rental assistant).
5. Configure the MCP tool-server runtime (empty tool list initially) so plugins can be registered in later subtasks.
6. Deploy the agent and verify it starts, connects to the model, and returns a basic completion response.
7. Expose an internal health endpoint the other subtasks can use to confirm the agent is live.

## Validation
Agent pod reaches Running state. Health endpoint returns 200. A test prompt sent directly to the agent returns a coherent LLM response confirming the gpt-5.4-pro model is connected. ConfigMap environment variables for all backend service URLs are resolvable inside the pod.
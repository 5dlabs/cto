Implement subtask 7001: Deploy OpenClaw agent with base configuration and GPT-5.4-pro model setup

## Objective
Deploy the OpenClaw agent runtime, configure AGENT_ID=morgan with MODEL=openai-api/gpt-5.4-pro, wire up environment variables from the sigma1-infra-endpoints ConfigMap, and verify the agent starts and responds to a basic health check.

## Steps
1. Create the OpenClaw agent deployment manifest (Dockerfile or container config) with AGENT_ID=morgan and MODEL=openai-api/gpt-5.4-pro environment variables.
2. Reference 'sigma1-infra-endpoints' ConfigMap via envFrom for all backend service endpoint URLs.
3. Mount OpenAI API key and any agent-specific secrets from Kubernetes secrets.
4. Configure the agent's base system prompt defining Morgan's persona (Sigma-1 sales/operations assistant).
5. Set up a health endpoint that confirms the agent process is running and can reach the OpenAI API.
6. Deploy to the cluster namespace and verify pod is running with correct environment.

## Validation
Agent pod starts successfully; health endpoint returns 200; environment variables from ConfigMap are correctly injected; agent can make a test completion call to OpenAI API and return a response.
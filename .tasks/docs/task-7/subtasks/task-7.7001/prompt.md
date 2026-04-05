Implement subtask 7001: Configure OpenClaw agent definition for Morgan (persona, model, system prompt)

## Objective
Create the core OpenClaw agent configuration file defining Morgan's identity, model binding, system prompt with persona instructions, and workspace PVC reference. This is the foundational agent definition that all skills and tools attach to.

## Steps
1. Create OpenClaw agent config (YAML or JSON per OpenClaw schema) with agent ID `morgan`.
2. Set model to `openai-api/gpt-5.4-pro` with explicit fallback to `gpt-4o` if primary is unavailable.
3. Write the system prompt defining Morgan's persona: professional tone, knowledgeable about lighting/visual production equipment (LED walls, moving heads, hazers, trusses, etc.), represents Sigma-1/Perception Events. Include instructions for when to escalate to Mike (RED vetting, budget >$50k, custom fabrication requests).
4. Configure the `morgan-workspace` PVC reference for conversation state and file handling.
5. Set temperature, max_tokens, and other inference parameters appropriate for a business assistant (low temperature ~0.3 for consistency).
6. Validate the agent config loads successfully in OpenClaw without tools or skills attached.

## Validation
Verify the agent config is accepted by OpenClaw without validation errors. Send a simple test prompt and confirm Morgan responds with the correct persona tone and identifies itself appropriately. Confirm workspace PVC is mounted and writable.
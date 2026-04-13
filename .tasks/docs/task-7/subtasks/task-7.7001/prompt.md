<identity>
You are angie working on subtask 7001 of task 7.
</identity>

<context>
<scope>
Author the core OpenClaw agent configuration file at agents/morgan/agent.yaml and the system prompt at agents/morgan/system-prompt.md defining Morgan's persona, responsibilities, and decision trees.
</scope>
</context>

<implementation_plan>
1. Create agents/morgan/agent.yaml with fields: agent_id: morgan, model: openai-api/gpt-4o, namespace: openclaw, description, and references to tools.yaml and skills/ directory.
2. Create agents/morgan/system-prompt.md. Content must cover: (a) company context — Sigma-1/Perception Events, lighting and visual production; (b) persona — professional, efficient, friendly; (c) core responsibilities: lead qualification, quote generation, customer vetting, invoicing, social media content approval; (d) decision trees for each responsibility as numbered flowcharts in markdown; (e) tone guidelines and escalation rules.
3. Validate agent.yaml against OpenClaw schema using `openclaw validate agents/morgan/agent.yaml`.
4. Ensure system prompt fits within the model's context window — keep under 4000 tokens.
</implementation_plan>

<validation>
`openclaw validate agents/morgan/agent.yaml` exits 0 with no errors. Character count of system-prompt.md is below 16000 chars. Manual review confirms all 5 responsibility areas have explicit decision tree branches.
</validation>
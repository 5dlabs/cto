PRD:
---
{{prd_content}}
---

{{#if codebase_context}}
This is a non-greenfield project. The codebase context above describes existing services, APIs, and patterns. Generate tasks that extend the existing system -- do not recreate what already exists.
{{/if}}

{{#research}}
Research current best practices, stable library versions, and known pitfalls before generating. Apply findings to `details` and `testStrategy` fields.
{{/research}}

Generate {{num_tasks}} tasks starting from ID {{next_id}}.

Requirements:
- Agent hints in titles: "(AgentName - Stack)"
- `testStrategy` with specific, measurable acceptance criteria
- `decision_points` left as empty array (extracted separately)
- `details` with step-by-step implementation guidance
{{#if codebase_context}}
- Reference existing services and files by name in `details`
- Do not duplicate existing functionality listed in the codebase context
{{/if}}

Output ONLY the JSON array. No markdown, no explanations.

<prd>
{{prd_content}}
</prd>

{{#if codebase_context}}
This is a non-greenfield project. The codebase context above describes existing services, APIs, and patterns. Generate tasks that extend the existing system; do not recreate what already exists.
{{/if}}

{{#research}}
<context name="research_instructions">
Research current best practices, stable library versions, and known pitfalls before generating. Apply findings to details and test_strategy fields.
</context>
{{/research}}

<parameters>
  <num_tasks>{{num_tasks}}</num_tasks>
  <starting_id>{{next_id}}</starting_id>
</parameters>

<output_format>
Requirements:
- Agent hints in titles: "(AgentName - Stack)"
- test_strategy with specific, measurable acceptance criteria
- decision_points left as empty array (extracted separately)
- details with step-by-step implementation guidance
{{#if codebase_context}}
- Reference existing services and files by name in details
- Do not duplicate existing functionality listed in the codebase context
{{/if}}

Output ONLY the JSON array. No markdown fences, no explanations.
</output_format>

<task_context>
  <task_id>{{task_id}}</task_id>
  <title>{{task_title}}</title>
  <description>{{task_description}}</description>
  <details>{{task_details}}</details>
{{#task_test_strategy}}
  <test_strategy>{{task_test_strategy}}</test_strategy>
{{/task_test_strategy}}
{{#expansion_prompt}}
  <expansion_guidance>{{expansion_prompt}}</expansion_guidance>
{{/expansion_prompt}}
{{#additional_context}}
  <additional_context>{{additional_context}}</additional_context>
{{/additional_context}}
{{#complexity_reasoning}}
  <complexity_reasoning>{{complexity_reasoning}}</complexity_reasoning>
{{/complexity_reasoning}}
</task_context>

Break down this task into {{#subtask_count}}exactly {{subtask_count}}{{/subtask_count}}{{^subtask_count}}an appropriate number of{{/subtask_count}} specific subtasks{{#enable_subagents}} optimized for parallel subagent execution{{/enable_subagents}}.

Use sequential IDs starting from {{next_id}}. The first subtask must have id={{next_id}}, the second must have id={{next_id_plus_1}}, and so on. Do not use parent task ID in subtask numbering.

{{#enable_subagents}}
Include subagentType and parallelizable on every subtask. Maximize parallel execution. Include at least one reviewer subtask after implementation.
{{/enable_subagents}}

Output: Continue the JSON array by outputting subtask objects directly. Start with the first subtask's opening brace { — do not output {"subtasks":[ again as that is already provided. End with ]} to close the array and object.

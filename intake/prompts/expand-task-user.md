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

<parameters>
{{#subtask_count}}  <subtask_count>{{subtask_count}}</subtask_count>{{/subtask_count}}
  <starting_id>{{next_id}}</starting_id>
{{#next_id_plus_1}}  <next_id_example>{{next_id_plus_1}}</next_id_example>{{/next_id_plus_1}}
</parameters>

Break down this task into {{#subtask_count}}exactly the specified number of{{/subtask_count}}{{^subtask_count}}an appropriate number of{{/subtask_count}} specific subtasks{{#enable_subagents}} optimized for parallel subagent execution{{/enable_subagents}}.

Use sequential IDs starting from the starting_id parameter. Do not use parent task ID in subtask numbering.

{{#enable_subagents}}
Include subagentType and parallelizable on every subtask. Maximize parallel execution. Include at least one reviewer subtask after implementation.
{{/enable_subagents}}

<output_format>
Continue the JSON array by outputting subtask objects directly. Start with the first subtask's opening brace { — do not output {"subtasks":[ again as that is already provided. End with ]} to close the array and object.
</output_format>

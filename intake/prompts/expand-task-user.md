Break down this task into {{#subtask_count}}exactly {{subtask_count}}{{/subtask_count}}{{^subtask_count}}an appropriate number of{{/subtask_count}} specific subtasks{{#enable_subagents}} optimized for parallel subagent execution{{/enable_subagents}}:

Task ID: {{task_id}}
Title: {{task_title}}
Description: {{task_description}}
Current details: {{task_details}}
{{#task_test_strategy}}
Test strategy: {{task_test_strategy}}
{{/task_test_strategy}}
{{#expansion_prompt}}

Expansion guidance: {{expansion_prompt}}
{{/expansion_prompt}}
{{#additional_context}}

Additional context: {{additional_context}}
{{/additional_context}}
{{#complexity_reasoning}}

Complexity Analysis Reasoning: {{complexity_reasoning}}
{{/complexity_reasoning}}

CRITICAL: You MUST use sequential IDs starting from {{next_id}}. The first subtask MUST have id={{next_id}}, the second MUST have id={{next_id_plus_1}}, and so on. Do NOT use parent task ID in subtask numbering!

{{#enable_subagents}}
SUBAGENT REQUIREMENTS: Include subagentType and parallelizable on EVERY subtask. Maximize parallel execution. Include at least one reviewer subtask after implementation.
{{/enable_subagents}}

OUTPUT: Continue the JSON array by outputting subtask objects directly. Start with the first subtask's opening brace { - do NOT output {"subtasks":[ again as that is already provided. End with ]} to close the array and object.

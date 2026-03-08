Analyze these tasks for complexity:
{{task_list}}

Tasks data:
{{tasks_json}}

Threshold: {{threshold}} (recommend subtasks for tasks scoring >= {{threshold}})

Use `task_id` and `task_title` field names (snake_case) in your output.

OUTPUT: Continue the JSON array by outputting analysis objects directly. Start with the first analysis object's opening brace { - do NOT output {"complexityAnalysis":[ again as that is already provided. End with ]} to close the array and object.

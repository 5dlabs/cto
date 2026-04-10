<tasks>
{{tasks_json}}
</tasks>

Analyze the above tasks for complexity.

Task summary: {{task_list}}

Threshold: {{threshold}} (recommend subtasks for tasks scoring >= {{threshold}})

Use task_id and task_title field names (snake_case) in your output.

Output: Continue the JSON array by outputting analysis objects directly. Start with the first analysis object's opening brace { — do not output {"complexityAnalysis":[ again as that is already provided. End with ]} to close the array and object.

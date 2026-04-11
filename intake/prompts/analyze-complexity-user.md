<tasks>
{{tasks_json}}
</tasks>

Analyze the above tasks for complexity.

<parameters>
  <task_summary>{{task_list}}</task_summary>
  <threshold>{{threshold}}</threshold>
</parameters>

Recommend subtasks for tasks scoring at or above the threshold.

<output_format>
Use task_id and task_title field names (snake_case) in your output.

Continue the JSON array by outputting analysis objects directly. Start with the first analysis object's opening brace { — do not output {"complexityAnalysis":[ again as that is already provided. End with ]} to close the array and object.
</output_format>

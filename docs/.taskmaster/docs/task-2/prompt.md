Add a small template step to write a JSON marker file prior to agent execution:
- Template: script task using alpine image
- Path: `docs/.taskmaster/current-task.json`
- JSON: {"taskId":"{{workflow.parameters.task-id}}","workflowName":"{{workflow.name}}","startTime":"{{workflow.createdAt}}"}


- Ensure the agent commits the file when creating the PR

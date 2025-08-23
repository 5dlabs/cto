# Argo Events: Label Selector Limitations

Template variables are not reliably supported in `labelSelector` fields within Argo Events triggers. Prefer parameterizing supported destinations instead:



- Use `parameters[].dest` for `metadata.name`, `spec.arguments.parameters[].value`, etc.


- For Workflow resumption, prefer passing the workflow name via `args` in `argoWorkflow` triggers.

Example (avoid this):




```yaml
labelSelector: "task-id={{extracted-task-id}},stage=waiting"








```

Preferred (resume by name):




```yaml
argoWorkflow:
  operation: resume
  args: ["play-task-{{taskId}}-workflow"]








```

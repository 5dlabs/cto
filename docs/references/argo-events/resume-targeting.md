# Argo Events: Resume Targeting Pattern

Use this pattern to resume a specific, existing Argo Workflow instance by name.





```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
spec:
  triggers:
  - template:
      name: resume-task-workflow
      argoWorkflow:
        operation: resume
        args: []
        parameters:
        - src:
            # Construct deterministic workflow name from event data (example: GitHub PR branch)
            dataTemplate: |
              {{ $ref := .Input.body.pull_request.head.ref }}
              {{ if hasPrefix $ref "task-" }}
                {{ $branch := trimPrefix "task-" $ref }}
                {{ $parts := splitList "-" $branch }}
                play-task-{{ index $parts 0 }}-workflow
              {{ else }}
                play-task-unknown-workflow
              {{ end }}
          dest: args.0








```

Notes


- Resume expects an existing workflow; provide its name via `args.0` (equivalent to `argo resume <name>`).


- Avoid dynamic `labelSelector`; template variables are not supported there reliably.
- Ensure your workflow naming is deterministic at creation time (e.g., `name: play-task-{{workflow.parameters.task-id}}-workflow`).

# Security Remediation Task Generator

Generate implementation tasks to address security findings.

## Input
- **security_report**: The security analysis with findings and recommendations
- **expanded_tasks**: The original task breakdown for context

## Process
1. For each finding with severity "medium" or higher, generate a remediation task
2. Group related findings into single tasks where they share the same area
3. Assign to agent "cipher" (security specialist)
4. Each task should be independently implementable

## Output Format
Return a JSON array of task objects, each with:
- `task_id`: Sequential ID continuing from the last scale task
- `title`: Task title (e.g., "Implement input validation for API endpoints")
- `agent`: "cipher"
- `finding_ids`: Array of finding IDs this task addresses (e.g., ["SEC-001", "SEC-003"])
- `depends_on`: Task IDs this depends on
- `description`: Detailed description of what to implement
- `subtasks`: Array of specific subtask descriptions
- `priority`: "critical" | "high" | "medium"
- `acceptance_criteria`: Array of testable criteria

## Guidelines
- Only generate tasks for findings of medium severity or above
- Low and info findings should be noted but don't need dedicated tasks
- Group related findings to avoid task sprawl
- Remediation tasks depend on the implementation tasks they're hardening
- Critical findings get their own dedicated task
- Include testing subtasks (e.g., "Run OWASP ZAP scan against endpoints")

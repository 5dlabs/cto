Implement subtask 10006: Document RBAC policies, secret rotation procedures, and audit logging configuration

## Objective
Create comprehensive documentation covering all RBAC roles and their justifications, secret rotation schedules and procedures, and audit logging configuration for operational reference.

## Steps
1. Document RBAC: a) Create a table mapping each ServiceAccount to its Role, the resources/verbs permitted, and the justification. b) Document the process for adding a new service and its RBAC requirements. c) Document how to audit existing RBAC bindings. 2. Document secret rotation: a) Create a table of all managed secrets, their external backend paths, rotation intervals, and responsible third-party provider. b) Document the manual rotation process for credentials that cannot be auto-rotated. c) Document the alert/escalation process for rotation failures. 3. Document audit logging: a) Describe the audit policy levels and what is captured at each level. b) Document how to query audit logs for common investigations (unauthorized access, secret access, RBAC changes). c) Document log retention periods and archival process. 4. Store all documentation in the repository's `docs/security/` directory.

## Validation
Review documentation for completeness: verify every ServiceAccount has a documented RBAC entry, every managed secret has a documented rotation entry, and audit log query examples return valid results when executed. Have a second team member follow the 'add new service' procedure from documentation alone and verify they can successfully create correct RBAC bindings.
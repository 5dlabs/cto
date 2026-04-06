Implement subtask 10006: Document compliance procedures and create secret rotation runbook

## Objective
Create comprehensive documentation covering RBAC policies, secret rotation procedures, audit log retention, and incident response procedures for compliance purposes.

## Steps
1. Create an RBAC policy document listing every ServiceAccount, its Role, and justification for each permission. 2. Write a secret rotation runbook covering: a) Automated rotation schedules and what triggers them. b) Manual rotation procedures for each secret type (PostgreSQL, Redis, API keys). c) Emergency rotation procedure if a secret is compromised. d) Verification steps after rotation. 3. Document audit log retention policy: how long logs are kept, where they are stored, who has access. 4. Create an incident response procedure for security events detected via audit logs. 5. Document the compliance checklist for GDPR-relevant actions: data access logging, right-to-erasure procedures, data processing records. 6. Store all documentation in the project repository under `/docs/security/`.

## Validation
Review all documentation for completeness: RBAC document covers all service accounts, rotation runbook covers all secret types with step-by-step instructions, audit retention policy specifies concrete durations. Execute the manual rotation runbook for one secret type end-to-end and verify the steps are accurate and complete. Have a second person follow the runbook without additional guidance to verify clarity.
<identity>
You are angie working on subtask 7004 of task 7.
</identity>

<context>
<scope>
Define the two GDPR MCP tools in a dedicated agents/morgan/tools-gdpr.yaml that fan out to all 5 backend services and aggregate or delete data, with audit logging for the delete operation.
</scope>
</context>

<implementation_plan>
1. Create agents/morgan/tools-gdpr.yaml.
2. sigma1_gdpr_export tool: tool_id: sigma1_gdpr_export, http_method: ORCHESTRATED, description: 'Export all personal data for a customer across all services'. Define fan_out array targeting:
   - GET http://equipment-catalog-svc.sigma1.svc.cluster.local:8080/internal/gdpr/export/{customer_id}
   - GET http://rms-svc.sigma1.svc.cluster.local:8080/internal/gdpr/export/{customer_id}
   - GET http://customer-vetting-svc.sigma1.svc.cluster.local:8080/internal/gdpr/export/{customer_id}
   - GET http://finance-svc.sigma1.svc.cluster.local:8080/internal/gdpr/export/{customer_id}
   - GET http://social-engine-svc.sigma1.svc.cluster.local:8080/internal/gdpr/export/{customer_id}
   Aggregation strategy: merge all response bodies into top-level keyed object by service name. On partial failure: include error key per service, do not fail entire request.
3. sigma1_gdpr_delete tool: tool_id: sigma1_gdpr_delete, http_method: ORCHESTRATED. Define fan_out array with DELETE method for all 5 services at /internal/gdpr/delete/{customer_id}. Post-delete: write audit record to OpenClaw audit log with customer_id, timestamp, and per-service status. Require human confirmation step in skill prompt before executing.
4. Reference both tools in admin.yaml skill manifest.
5. Include GDPR tools in main agent.yaml tool reference list.
</implementation_plan>

<validation>
`openclaw validate agents/morgan/tools-gdpr.yaml` exits 0. sigma1_gdpr_export fan_out array contains exactly 5 entries. sigma1_gdpr_delete fan_out array contains exactly 5 entries and audit_log field is present. `openclaw agent list-tools morgan` includes both gdpr tool IDs.
</validation>
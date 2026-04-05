Implement subtask 1008: Create sigma1-rbac-roles ConfigMap

## Objective
Create the shared RBAC role definitions ConfigMap that defines the application-level role/permission matrix for all services to consume.

## Steps
1. Create `sigma1-rbac-roles.yaml` ConfigMap in namespace `sigma1` with a `roles.json` data key containing:
   ```json
   {
     "roles": {
       "admin": {
         "description": "Full platform access",
         "permissions": ["*"]
       },
       "operator": {
         "description": "Day-to-day operations, no system config",
         "permissions": ["catalog:read", "catalog:write", "rms:read", "rms:write", "finance:read", "finance:write", "crm:read", "crm:write", "vetting:read", "vetting:write", "social:read", "social:write"]
       },
       "morgan-agent": {
         "description": "AI agent with scoped access",
         "permissions": ["catalog:read", "rms:read", "finance:read", "crm:read", "vetting:read", "social:read", "social:write", "audit:write"]
       },
       "readonly": {
         "description": "Read-only dashboard access",
         "permissions": ["catalog:read", "rms:read", "finance:read", "crm:read"]
       }
     }
   }
   ```
2. Apply the ConfigMap.
3. This is a shared schema; each service will implement permission checks against this matrix.

## Validation
`kubectl get configmap sigma1-rbac-roles -n sigma1 -o jsonpath='{.data.roles\.json}'` returns valid JSON. Parsing the JSON confirms 4 roles exist with the correct permission arrays. The `admin` role has `["*"]` permissions.
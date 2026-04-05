Implement subtask 10005: Create GDPR audit logging database schema and migration

## Objective
Define the `audit` schema with `audit_log`, `data_export_requests`, and `data_deletion_requests` tables as a CNPG init migration, ensuring all tables have proper indexes and constraints.

## Steps
1. Create a SQL migration file for CNPG initialization that creates the `audit` schema.
2. Create `audit.audit_log` table:
   - `id` UUID PRIMARY KEY DEFAULT gen_random_uuid()
   - `service_name` VARCHAR(100) NOT NULL
   - `action` VARCHAR(20) NOT NULL CHECK (action IN ('create', 'read', 'update', 'delete', 'export'))
   - `entity_type` VARCHAR(100) NOT NULL
   - `entity_id` VARCHAR(255) NOT NULL
   - `actor_service` VARCHAR(100) NOT NULL
   - `actor_user_id` VARCHAR(255) NULLABLE
   - `timestamp` TIMESTAMPTZ NOT NULL DEFAULT NOW()
   - `request_metadata` JSONB DEFAULT '{}'
   - Index on (service_name, timestamp)
   - Index on (entity_type, entity_id)
   - Index on (actor_user_id) WHERE actor_user_id IS NOT NULL
3. Create `audit.data_export_requests` table:
   - `id` UUID PRIMARY KEY DEFAULT gen_random_uuid()
   - `customer_id` VARCHAR(255) NOT NULL
   - `requested_at` TIMESTAMPTZ NOT NULL DEFAULT NOW()
   - `completed_at` TIMESTAMPTZ NULLABLE
   - `export_url` TEXT NULLABLE
   - `expires_at` TIMESTAMPTZ NULLABLE
   - Index on (customer_id)
4. Create `audit.data_deletion_requests` table:
   - `id` UUID PRIMARY KEY DEFAULT gen_random_uuid()
   - `customer_id` VARCHAR(255) NOT NULL
   - `requested_at` TIMESTAMPTZ NOT NULL DEFAULT NOW()
   - `completed_at` TIMESTAMPTZ NULLABLE
   - `services_purged` TEXT[] DEFAULT '{}'
   - Index on (customer_id)
5. Add the migration to the CNPG cluster init SQL ConfigMap or initdb section.
6. Grant INSERT on audit.audit_log to the application database role used by services.
7. Grant INSERT/UPDATE on data_export_requests and data_deletion_requests to the orchestration Job role.

## Validation
Apply the migration to a test CNPG instance. Verify all 3 tables exist in the `audit` schema with correct columns and types. Verify indexes are created. Insert test rows into each table and confirm constraints work (e.g., invalid action value rejected). Verify application role can INSERT into audit_log.
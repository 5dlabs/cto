Implement subtask 2002: Extend task schema with delegate_id and delegation_status fields

## Objective
Add delegate_id (string | null) and delegation_status ('assigned' | 'pending' | 'failed') fields to the task schema/type definitions used in cto-pm.

## Steps
1. Locate the task type/interface definition in the cto-pm codebase (likely a TypeScript interface or Zod schema).
2. Add field `delegate_id: string | null` — the resolved Linear user ID.
3. Add field `delegation_status: 'assigned' | 'pending' | 'failed'` — tracks the outcome of delegation resolution.
4. Set default values: delegate_id = null, delegation_status = 'pending'.
5. If tasks are persisted to a store or database, update the persistence layer to include these new fields.
6. Update any existing serialization/deserialization logic to handle the new fields.
7. Ensure backward compatibility: existing tasks without these fields should default to delegate_id = null, delegation_status = 'pending' when read.

## Validation
Unit test: create a task object with the extended schema and verify delegate_id and delegation_status are present with correct types. Test deserialization of a legacy task object (without new fields) and confirm defaults are applied (null and 'pending').
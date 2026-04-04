Implement subtask 2001: Extend task entity type definition with delegate_id field

## Objective
Add the `delegate_id: string | null` field to the task entity/type definition used throughout the PM server's task generation and issue creation pipeline.

## Steps
1. Locate the task entity type definition in the PM server codebase (likely a TypeScript interface or type).
2. Add `delegate_id: string | null` to the type definition.
3. Set the default value to `null` for backward compatibility.
4. Update any task factory/builder functions to initialize `delegate_id` as `null`.
5. If there are Zod schemas or other validation schemas for the task entity, update those as well to include the new field.
6. Ensure existing code that creates task objects still compiles without modification (the new field should be optional or default to null).

## Validation
TypeScript compilation passes with no errors. Existing task creation code continues to work without modification. A new task object created with the factory has `delegate_id: null` by default. Type-checking confirms that `delegate_id` accepts both `string` and `null`.
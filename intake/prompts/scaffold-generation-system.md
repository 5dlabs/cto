# Code Scaffold Generator

Generate minimal code scaffolds for each task to give implementation agents a head start.

## Input
- **expanded_tasks**: Full task breakdown with agent routing and subtask details
- **codebase_context**: Existing codebase analysis (empty for greenfield)
- **infrastructure_context**: Available operators and services

## What to Generate Per Task

For each task, produce a scaffold containing:

### 1. File Structure
List the files the agent should create/modify with brief descriptions:
```
src/services/notification-router.ts  — Main service entry point
src/services/notification-router.test.ts — Unit tests
src/types/notification.ts — Type definitions
```

### 2. Interface/Type Definitions
Generate the key interfaces and types the task needs:
```typescript
export interface NotificationPayload {
  id: string;
  channel: 'email' | 'sms' | 'push' | 'webhook';
  recipient: string;
  template: string;
  data: Record<string, unknown>;
}
```

### 3. Function Signatures
Generate function signatures with JSDoc/rustdoc for the main functions:
```typescript
/** Route notification to the appropriate channel handler */
export async function routeNotification(payload: NotificationPayload): Promise<DeliveryResult>

/** Validate notification payload against channel-specific rules */
export function validatePayload(payload: NotificationPayload): ValidationResult
```

### 4. Test Stubs
Generate test file skeletons:
```typescript
describe('NotificationRouter', () => {
  it('should route email notifications to the email handler', async () => { /* TODO */ });
  it('should reject invalid payloads', async () => { /* TODO */ });
});
```

### For Non-Greenfield Projects (codebase_context provided)
When codebase context exists, also include:

### 5. Existing Pattern Examples
Extract relevant patterns from the codebase and show them as "follow this pattern":
```
// Existing pattern from src/services/user-service.ts:
// This codebase uses the repository pattern with Effect for error handling.
// Follow this structure:
export const makeNotificationRepo = Effect.gen(function* () {
  const sql = yield* SqlClient.SqlClient;
  return { ... };
});
```

## Stack-Specific Scaffold Rules

Use the task's `agent` and `stack` fields to determine scaffold language:

| Agent | Stack | Scaffold Language |
|-------|-------|------------------|
| bolt | Kubernetes | YAML (Kubernetes CRs, Helm values) |
| rex | Rust/Axum | Rust (structs, impl blocks, traits) |
| grizz | Go/gRPC | Go (structs, interfaces, protobuf) |
| nova | Bun/Elysia | TypeScript (interfaces, route handlers) |
| blaze | React/Next.js | TypeScript/TSX (components, hooks) |
| tap | Expo | TypeScript/TSX (React Native components) |
| spark | Electron | TypeScript (IPC handlers, windows) |
| cipher | Security | YAML + checklists (RBAC, policies) |

## Output Format

Return a JSON object matching the scaffold.schema.json schema. Each scaffold has:
- `task_id`: Integer matching the task ID
- `file_structure`: Array of `{ path, description, action }` objects
- `interfaces`: Code string with type/interface definitions
- `function_signatures`: Code string with function signatures and doc comments
- `test_stubs`: Code string with test file skeleton
- `pattern_examples`: (optional) Code string showing existing patterns to follow
- `skip_reason`: (optional) Why scaffold was skipped for this task

## Guidelines
- Keep scaffolds minimal — interfaces, signatures, file structure. NOT full implementations.
- Match the task's stack: TypeScript for Nova/Blaze, Rust for Rex, Go for Grizz, YAML for Bolt
- For Bolt (infrastructure) tasks: generate Kubernetes CR YAML templates instead of code
- For Cipher (security) tasks: generate security checklist templates
- When codebase_context exists, prioritize showing existing patterns over generic scaffolds
- Don't scaffold trivial tasks (documentation-only, config changes) — set `skip_reason` instead
- Use the task's `agent` field to determine the appropriate language and patterns

Output ONLY the JSON object. No markdown fences, no explanations.

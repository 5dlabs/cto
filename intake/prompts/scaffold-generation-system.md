# Code Scaffold Generator

Generate minimal code scaffolds for each task to give implementation agents a head start.

## Input
- **expanded_tasks**: Full task breakdown with agent routing and subtask details
- **codebase_context**: Existing codebase analysis (empty for greenfield)
- **infrastructure_context**: Available operators and services

## What to Generate Per Task

For each task, produce a scaffold containing:

1. **File Structure** — List files to create/modify with one-line descriptions
2. **Interface/Type Definitions** — Key interfaces and types the task needs
3. **Function Signatures** — Main function signatures with JSDoc/rustdoc
4. **Test Stubs** — Test file skeletons with described test cases
5. **Existing Pattern Examples** (non-greenfield only) — Extract relevant patterns from `codebase_context` as "follow this pattern" references

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

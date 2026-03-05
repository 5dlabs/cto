/**
 * stack-validators — Stack-specific validation commands mapping.
 *
 * Maps tech stack identifiers to their type-check, test, and lint commands.
 * Used by generate-workflow.ts to produce per-task implementation workflows.
 */

export interface ValidationCommands {
  type_check: string;
  test: string;
  lint: string;
}

const STACK_MAP: Record<string, ValidationCommands> = {
  typescript: {
    type_check: 'bun tsc --noEmit',
    test: 'bun test',
    lint: 'bun lint',
  },
  bun: {
    type_check: 'bun tsc --noEmit',
    test: 'bun test',
    lint: 'bun lint',
  },
  react: {
    type_check: 'bun tsc --noEmit',
    test: 'bun test',
    lint: 'bun lint',
  },
  'next.js': {
    type_check: 'bun tsc --noEmit',
    test: 'bun test',
    lint: 'bun lint',
  },
  nextjs: {
    type_check: 'bun tsc --noEmit',
    test: 'bun test',
    lint: 'bun lint',
  },
  expo: {
    type_check: 'bun tsc --noEmit',
    test: 'bun test',
    lint: 'bun lint',
  },
  electron: {
    type_check: 'bun tsc --noEmit',
    test: 'bun test',
    lint: 'bun lint',
  },
  rust: {
    type_check: 'cargo check',
    test: 'cargo test',
    lint: 'cargo clippy -- -D warnings',
  },
  axum: {
    type_check: 'cargo check',
    test: 'cargo test',
    lint: 'cargo clippy -- -D warnings',
  },
  go: {
    type_check: 'go vet ./...',
    test: 'go test ./...',
    lint: 'golangci-lint run',
  },
  grpc: {
    type_check: 'go vet ./...',
    test: 'go test ./...',
    lint: 'golangci-lint run',
  },
  kubernetes: {
    type_check: 'kubectl apply --dry-run=client -f .',
    test: 'helm template . | kubeval',
    lint: 'yamllint .',
  },
  helm: {
    type_check: 'kubectl apply --dry-run=client -f .',
    test: 'helm template . | kubeval',
    lint: 'yamllint .',
  },
};

const DEFAULT_COMMANDS: ValidationCommands = {
  type_check: 'echo "No type checker configured"',
  test: 'echo "No test runner configured"',
  lint: 'echo "No linter configured"',
};

/**
 * Resolve validation commands for a given stack string.
 * Tries exact match first, then checks if any known stack key
 * appears as a substring (case-insensitive).
 */
export function getValidationCommands(stack: string | undefined): ValidationCommands {
  if (!stack) return DEFAULT_COMMANDS;

  const lower = stack.toLowerCase();

  // Exact match
  if (STACK_MAP[lower]) return STACK_MAP[lower];

  // Substring match (e.g. "Rust/SQLx" matches "rust", "Go/JWT" matches "go")
  for (const [key, commands] of Object.entries(STACK_MAP)) {
    if (lower.includes(key)) return commands;
  }

  return DEFAULT_COMMANDS;
}

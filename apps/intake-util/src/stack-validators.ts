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

export interface SecurityCommands {
  audit: string;
  scan: string;
  secrets: string;
}

export interface TestCommands {
  unit: string;
  integration: string;
  coverage: string;
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

const SECURITY_MAP: Record<string, SecurityCommands> = {
  typescript: {
    audit: 'npm audit --production || bun audit',
    scan: 'npx semgrep --config auto --json .',
    secrets: 'npx gitleaks detect --source . --no-git',
  },
  bun: {
    audit: 'bun audit || npm audit --production',
    scan: 'npx semgrep --config auto --json .',
    secrets: 'npx gitleaks detect --source . --no-git',
  },
  react: {
    audit: 'npm audit --production',
    scan: 'npx semgrep --config auto --json .',
    secrets: 'npx gitleaks detect --source . --no-git',
  },
  'next.js': {
    audit: 'npm audit --production',
    scan: 'npx semgrep --config auto --json .',
    secrets: 'npx gitleaks detect --source . --no-git',
  },
  rust: {
    audit: 'cargo audit',
    scan: 'cargo clippy -- -W clippy::all -W clippy::pedantic',
    secrets: 'gitleaks detect --source . --no-git',
  },
  axum: {
    audit: 'cargo audit',
    scan: 'cargo clippy -- -W clippy::all -W clippy::pedantic',
    secrets: 'gitleaks detect --source . --no-git',
  },
  go: {
    audit: 'govulncheck ./...',
    scan: 'gosec ./...',
    secrets: 'gitleaks detect --source . --no-git',
  },
  grpc: {
    audit: 'govulncheck ./...',
    scan: 'gosec ./...',
    secrets: 'gitleaks detect --source . --no-git',
  },
  kubernetes: {
    audit: 'kubeaudit all -f .',
    scan: 'kube-score score *.yaml',
    secrets: 'gitleaks detect --source . --no-git',
  },
  helm: {
    audit: 'kubeaudit all -f .',
    scan: 'kube-score score *.yaml',
    secrets: 'gitleaks detect --source . --no-git',
  },
};

const DEFAULT_SECURITY: SecurityCommands = {
  audit: 'echo "No dependency auditor configured"',
  scan: 'echo "No security scanner configured"',
  secrets: 'gitleaks detect --source . --no-git',
};

const TEST_MAP: Record<string, TestCommands> = {
  typescript: {
    unit: 'bun test',
    integration: 'bun test --filter integration',
    coverage: 'bun test --coverage',
  },
  bun: {
    unit: 'bun test',
    integration: 'bun test --filter integration',
    coverage: 'bun test --coverage',
  },
  react: {
    unit: 'bun test',
    integration: 'bun test --filter integration',
    coverage: 'bun test --coverage',
  },
  'next.js': {
    unit: 'bun test',
    integration: 'bun test --filter integration',
    coverage: 'bun test --coverage',
  },
  rust: {
    unit: 'cargo test --lib',
    integration: 'cargo test --test "*"',
    coverage: 'cargo llvm-cov --json',
  },
  axum: {
    unit: 'cargo test --lib',
    integration: 'cargo test --test "*"',
    coverage: 'cargo llvm-cov --json',
  },
  go: {
    unit: 'go test -short ./...',
    integration: 'go test -run Integration ./...',
    coverage: 'go test -coverprofile=coverage.out ./... && go tool cover -func=coverage.out',
  },
  grpc: {
    unit: 'go test -short ./...',
    integration: 'go test -run Integration ./...',
    coverage: 'go test -coverprofile=coverage.out ./... && go tool cover -func=coverage.out',
  },
  kubernetes: {
    unit: 'helm unittest .',
    integration: 'kubectl apply --dry-run=server -f .',
    coverage: 'echo "Coverage not applicable for Kubernetes manifests"',
  },
  helm: {
    unit: 'helm unittest .',
    integration: 'kubectl apply --dry-run=server -f .',
    coverage: 'echo "Coverage not applicable for Kubernetes manifests"',
  },
};

const DEFAULT_TEST: TestCommands = {
  unit: 'echo "No unit test runner configured"',
  integration: 'echo "No integration test runner configured"',
  coverage: 'echo "No coverage tool configured"',
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

export function getSecurityCommands(stack: string | undefined): SecurityCommands {
  if (!stack) return DEFAULT_SECURITY;
  const lower = stack.toLowerCase();
  if (SECURITY_MAP[lower]) return SECURITY_MAP[lower];
  for (const [key, commands] of Object.entries(SECURITY_MAP)) {
    if (lower.includes(key)) return commands;
  }
  return DEFAULT_SECURITY;
}

export function getTestCommands(stack: string | undefined): TestCommands {
  if (!stack) return DEFAULT_TEST;
  const lower = stack.toLowerCase();
  if (TEST_MAP[lower]) return TEST_MAP[lower];
  for (const [key, commands] of Object.entries(TEST_MAP)) {
    if (lower.includes(key)) return commands;
  }
  return DEFAULT_TEST;
}

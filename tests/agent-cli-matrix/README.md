# Agent × CLI Test Matrix

This directory contains the comprehensive test suite for verifying that each agent works correctly with each CLI.

## Structure

```
tests/agent-cli-matrix/
├── README.md              # This file
├── matrix.yaml            # Full test matrix definition
├── acceptance-criteria/   # Acceptance criteria by category
│   ├── container.yaml     # Container script requirements
│   ├── memory.yaml        # Memory/system prompt requirements
│   └── config.yaml        # Config file requirements
├── results/               # Test results (gitignored, auto-generated)
│   └── YYYY-MM-DD-HH-MM-SS.json
└── run-tests.sh           # Test runner script
```

## Running Tests

### Quick Validation (Template Rendering Only)
```bash
# Uses AGENT_TEMPLATES_PATH to find templates
AGENT_TEMPLATES_PATH="$(pwd)/templates" cargo test -p controller --test e2e_template_tests
```

### Full Matrix Test
```bash
# Run the matrix test script
./tests/agent-cli-matrix/run-tests.sh
```

### Specific Agent/CLI Combination
```bash
./tests/agent-cli-matrix/run-tests.sh --agent rex --cli claude
```

## Test Matrix

| Agent   | Primary Job  | Supported CLIs                              | Notes                    |
|---------|--------------|---------------------------------------------|--------------------------|
| Rex     | coder        | claude, codex, cursor, factory, gemini, opencode | Rust specialist          |
| Blaze   | coder        | claude, codex, cursor, factory, gemini, opencode | Frontend specialist      |
| Grizz   | coder        | claude, codex, cursor, factory, gemini, opencode | Go specialist            |
| Nova    | coder        | claude, codex, cursor, factory, gemini, opencode | Node.js specialist       |
| Tap     | coder        | claude, codex, cursor, factory, gemini, opencode | TAP/Mobile specialist    |
| Spark   | coder        | claude, codex, cursor, factory, gemini, opencode | Python specialist        |
| Bolt    | deploy       | claude, factory                             | Deployment specialist    |
| Cipher  | security     | claude, factory                             | Security specialist      |
| Cleo    | quality      | claude, factory                             | Quality assurance        |
| Tess    | test         | claude, factory                             | Testing specialist       |
| Stitch  | review       | claude, factory                             | Code review specialist   |
| Morgan  | docs/intake  | claude                                      | Documentation/PM         |
| Atlas   | integration  | claude                                      | Integration specialist   |

## Acceptance Criteria

Each test validates:

### 1. Container Script (`container.sh`)
- [ ] Script renders without errors
- [ ] Contains valid bash syntax
- [ ] Includes all required partials (header, config, github-auth, git-setup, task-files, tools-config)
- [ ] CLI-specific invoke partial (`{{> cli_execute}}`) is present
- [ ] Script size is reasonable (> 5KB for full containers)

### 2. Memory/System Prompt
- [ ] Agent-specific prompt renders without errors
- [ ] Contains agent identity/role description
- [ ] Contains relevant domain expertise keywords
- [ ] Includes expected Handlebars variables

### 3. Config Files
- [ ] CLI-specific config file is generated
- [ ] Config is valid JSON/TOML (depending on CLI)
- [ ] Required fields are present
- [ ] Model and settings are correctly interpolated

## Adding New Tests

1. Add agent definition to `matrix.yaml`
2. Add any new acceptance criteria to `acceptance-criteria/`
3. Run `./run-tests.sh --generate` to update test fixtures




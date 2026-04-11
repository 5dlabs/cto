# EAS Workflows Skill

Help developers write and edit EAS CI/CD workflow YAML files.

## Reference Documentation

Fetch these resources before generating or validating workflow files:

1. **JSON Schema** — [https://api.expo.dev/v2/workflows/schema](https://api.expo.dev/v2/workflows/schema)
   - It is NECESSARY to fetch this schema
   - Source of truth for validation
   - All job types and their required/optional parameters
   - Trigger types and configurations
   - Runner types, VM images, and all enums

2. **Syntax Documentation** — [Expo Workflows Syntax](https://raw.githubusercontent.com/expo/expo/refs/heads/main/docs/pages/eas/workflows/syntax.mdx)
   - Overview of workflow YAML syntax
   - Examples and English explanations
   - Expression syntax and contexts

3. **Pre-packaged Jobs** — [Pre-packaged Jobs Docs](https://raw.githubusercontent.com/expo/expo/refs/heads/main/docs/pages/eas/workflows/pre-packaged-jobs.mdx)
   - Documentation for supported pre-packaged job types
   - Job-specific parameters and outputs

Do not rely on memorized values; these resources evolve as new features are added.

## Workflow File Location

Workflows live in `.eas/workflows/*.yml` (or `.yaml`).

## Top-Level Structure

A workflow file has these top-level keys:

- `name` — Display name for the workflow
- `on` — Triggers that start the workflow (at least one required)
- `jobs` — Job definitions (required)
- `defaults` — Shared defaults for all jobs
- `concurrency` — Control parallel workflow runs

Consult the schema for the full specification of each section.

## Expressions

Use `${{ }}` syntax for dynamic values. The schema defines available contexts:

- `github.*` — GitHub repository and event information
- `inputs.*` — Values from `workflow_dispatch` inputs
- `needs.*` — Outputs and status from dependent jobs
- `jobs.*` — Job outputs (alternative syntax)
- `steps.*` — Step outputs within custom jobs
- `workflow.*` — Workflow metadata

## Basic Workflow Example

```yaml
name: Build and Deploy
on:
  push:
    branches: [main]

jobs:
  build:
    name: Build iOS
    type: build
    params:
      platform: ios
      profile: production

  submit:
    name: Submit to App Store
    type: submit
    needs: [build]
    params:
      platform: ios
      profile: production
```

## Common Job Types

### Build Job

```yaml
build-ios:
  name: Build iOS App
  type: build
  params:
    platform: ios
    profile: production
```

### Submit Job

```yaml
submit-ios:
  name: Submit to TestFlight
  type: submit
  needs: [build-ios]
  params:
    platform: ios
    profile: production
```

### Custom Job

```yaml
test:
  name: Run Tests
  type: custom
  steps:
    - name: Checkout
      uses: actions/checkout@v4
    - name: Install dependencies
      run: npm ci
    - name: Run tests
      run: npm test
```

## Triggers

### Push Trigger

```yaml
on:
  push:
    branches: [main, develop]
    paths:
      - 'src/**'
      - 'package.json'
```

### Pull Request Trigger

```yaml
on:
  pull_request:
    branches: [main]
```

### Manual Trigger

```yaml
on:
  workflow_dispatch:
    inputs:
      environment:
        description: 'Deployment environment'
        required: true
        default: 'staging'
        type: choice
        options:
          - staging
          - production
```

### Schedule Trigger

```yaml
on:
  schedule:
    - cron: '0 0 * * *'  # Daily at midnight
```

## Job Dependencies

```yaml
jobs:
  build:
    type: build
    params:
      platform: ios
      profile: production

  submit:
    type: submit
    needs: [build]  # Waits for build to complete
    params:
      platform: ios
      profile: production
```

## Conditional Execution

```yaml
jobs:
  deploy:
    type: custom
    if: github.ref == 'refs/heads/main'
    steps:
      - run: echo "Deploying..."
```

## Generating Workflows

When generating or editing workflows:

1. Fetch the schema to get current job types, parameters, and allowed values
2. Validate that required fields are present for each job type
3. Verify job references in `needs` and `after` exist in the workflow
4. Check that expressions reference valid contexts and outputs
5. Ensure `if` conditions respect the schema's length constraints

## Validation

After generating or editing a workflow file, validate it against the schema using the EAS CLI:

```bash
eas workflow:validate .eas/workflows/build.yml
```

## Answering Questions

When users ask about available options (job types, triggers, runner types, etc.), fetch the schema and derive the answer from it rather than relying on potentially outdated information.

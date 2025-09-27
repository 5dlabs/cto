# Factory Droid Agent

This container image packages the Factory `droid` CLI so 5D Labs agents can run Factory workflows (interactive or headless) inside Kubernetes jobs.

## Contents

- Based on `ghcr.io/5dlabs/runtime:latest`
- Factory CLI installed via the official installer (`curl -fsSL https://app.factory.ai/cli | sh`)
- Non-root `node` user with writable `$HOME/.factory` for CLI state and BYOK settings
- `droid --version` is executed at build time for traceability

## Environment variables

The CLI expects the following variables at runtime:

- `FACTORY_API_KEY` (required) – API token issued from <https://app.factory.ai/settings/api-keys>
- `FACTORY_WORKSPACE_ID` (optional) – Override the default workspace when running headless
- `FACTORY_ORG_ID` (optional) – Required for some enterprise BYOK configurations

When running in headless mode (`droid exec`), you can also provide:

- `FACTORY_MODEL` – Default model identifier (falls back to CLI settings/config)
- `FACTORY_REASONING_EFFORT` – `off|low|medium|high`

## Usage

```bash
# Inside a running Factory agent container
export FACTORY_API_KEY=fk-...
droid --help

droid exec "summarize repository" --auto low --output-format json
```

The CLI stores authentication and session data under `/home/node/.factory`. Mount a persistent volume if you want to reuse tokens between runs.

## Smoke test

The Docker build runs `droid --version`. A runtime health check can be as simple as:

```bash
droid exec "print the current working directory" --auto low --output-format text
```

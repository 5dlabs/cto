# Task 11: Project-wide MCP Requirements and Toolman Client Config Unification

## Overview
Move away from generating `@client-config.json` per task. Define a single, project-wide MCP tools configuration in `docs/requirements.yaml`, and unify the runtime so the docs/code agents consume project tools automatically. Translate project requirements into the format Toolman expects, minimizing duplication and drift.

Important:
- Do NOT re-implement working flows. Extend existing requirements under `docs/requirements.yaml` and adjust agents to stop emitting per-task client configs.
- Secrets are referenced by name and read from Kubernetes (External Secrets). No secrets in git.

## Goals
- Single source of truth for MCP tools at project level
- Agents discover tools from project requirements (no per-task `@client-config.json`)
- Toolman compatibility: ensure the project requirements produce equivalent capability

## What exists
- `docs/requirements.yaml` with project env and `secrets:` references (`doc-server-secrets`, `agent-platform-secrets`, `docs-admin-secrets`)
- Docs agent currently writes per-task `client-config.json`

## What to implement
1) Extend `docs/requirements.yaml` to declare project MCP tools (GitHub admin, k8s verify, Argo CD API, etc.). Tools should read tokens/URLs from env vars provided by referenced secrets.
2) Adjust docs/code agents to prefer project tools; stop generating per-task `@client-config.json`.
3) Add a small adapter that, when Toolman expects `@client-config.json`, synthesizes it from project requirements at runtime (no file committed per task).
4) Document the behavior and update prompts to mention project tools are available.

## Implementation outline
- Update `docs/requirements.yaml` tools section (examples):
```yaml
tools:
  - name: github-admin
    transport: http
    endpoint: https://api.github.com
    headers:
      Authorization: "Bearer ${GITHUB_ADMIN_TOKEN}"

  - name: argocd
    transport: http
    endpoint: ${ARGOCD_SERVER}
    headers:
      Authorization: "Bearer ${ARGOCD_AUTH_TOKEN}"

  - name: k8s-verify
    transport: exec
    command: ["/bin/kubectl", "version", "--client"]
```

- Runtime adapter (pseudo):
  - On task start, if `@client-config.json` is requested by Toolman, render it from `docs/requirements.yaml` into a temp file rather than committing one per task.

## Acceptance criteria
- Agents no longer commit `@client-config.json` under each task directory
- Project tools declared in `docs/requirements.yaml` are usable by agents
- Toolman receives a compatible client config synthesized at runtime
- Secrets are read from Kubernetes and never committed to git

## Validation
- Run a docs job and verify no new per-task `client-config.json` files appear
- Confirm tool invocations succeed (GitHub admin, Argo CD calls, kubectl verify)
- Inspect logs to see synthesized Toolman config created at runtime

## Notes
- Keep Rust-only scope for now
- Revisit multi-language adapters later when non-Rust repos land


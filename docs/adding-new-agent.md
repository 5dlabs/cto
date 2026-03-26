# Adding a New CTO Agent

This is the current, code-verified checklist for introducing a new agent identity into CTO.

Use this when you add a new `5DLabs-<Agent>` GitHub App and want the agent to work end-to-end (templates, routing, Linear OAuth, secrets, and deployment).

## Scope and intent

A new agent usually requires changes across:

- identity and OAuth credentials
- agent runtime configuration
- template/routing code
- PM multi-app Linear registration
- Kubernetes secret wiring
- optional UI/marketing avatar surfaces

Keep this as a focused infra/config change. Avoid mixing feature work with agent bootstrap changes.

## 1. Create external identities

1. Create GitHub App: `5DLabs-<Agent>`
2. Create Linear OAuth app: `5DLabs-<Agent>`
3. Record credentials (GitHub app id/client id/private key + Linear client id/client secret/webhook secret)

Current callback/webhook values used by PM and setup tooling:

- Callback: `https://cto.5dlabs.ai/oauth/callback`
- Linear webhook: `https://cto.5dlabs.ai/webhooks/linear`

## 2. Store credentials in OpenBao

Store secrets using existing key patterns:

- `github-app-<agent>`
- `linear-app-<agent>`

Example:

```bash
bao kv put github-app-<agent> app-id="..." client-id="..." private-key="$(cat app.pem)"
bao kv put linear-app-<agent> client_id="..." client_secret="..." webhook_secret="lin_wh_..."
```

## 3. Update agent runtime config

Update both config files to keep local/dev and chart-packaged config aligned:

- `cto-config.json`
- `infra/charts/cto/cto-config.json`

Add `agents.<agent>` with:

- `githubApp`
- `cli`
- `model`
- `tools.remote` (+ `localServers` if needed)
- `skills` (preferred source; YAML skill mappings are fallback only)

If the agent should be a play default override, also update `defaults.play` in config.

## 4. Add agent templates

Create templates under:

- `templates/agents/<agent>/coder.md.hbs`
- `templates/agents/<agent>/healer.md.hbs` (if the agent supports healer runs)

Template selection in controller resolves to `agents/{agent}/{job}.md.hbs`, so filenames must match job names.

## 5. Wire controller routing and defaults

Update hardcoded mappings in controller when introducing a new agent identity:

- `crates/controller/src/tasks/code/templates.rs`
  - `get_default_agent_tools`
  - `get_agent_system_prompt_template`
  - `extract_agent_name_from_github_app`
- `crates/controller/src/tasks/code/agent.rs`
  - `AgentClassifier::new` implementation-agent workspace classification (only if shared workspace is required)

Tip: after adding mappings, run a quick grep to catch missed hardcoded lists:

```bash
rg -n "5DLabs-|\"rex\"|implementation_agents" crates/controller/src/tasks/code
```

## 6. Wire PM Linear multi-app support

Update PM’s known agent list:

- `crates/pm/src/config.rs` (`AGENT_NAMES`)

This controls which `LINEAR_APP_<AGENT>_*` env vars PM loads.

## 7. Wire secrets manifests and Helm values

Update GitOps/Helm secret wiring:

- `infra/gitops/manifests/external-secrets/cto-secrets.yaml`
  - add `github-app-5dlabs-<agent>` ExternalSecret
  - add `linear-app-<agent>` ExternalSecret
- `infra/charts/cto/values.yaml`
  - add `secrets.githubApps[]` entry for the new agent

## 8. Update operational docs/tooling lists

Update any explicit agent inventories/lists used by operators:

- `AGENTS.md` (platform roster)
- `docs/scripts/setup-linear-agents.sh` (`AGENTS=(...)` list)

## 9. Optional UI avatar surfaces

If the agent is user-visible in UI/marketing pages, add/update avatars and mappings where needed:

- `crates/cto-lite/ui/public/agents/<agent>-avatar-512.png`
- `crates/cto-lite/ui/src/lib/agent-branding.ts`
- marketing/splash assets if that surface should display the new agent

## 10. Verify before PR

Run the minimum verification set:

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test -p controller
cargo test -p pm
cargo run --bin test_templates
```

Validate config and list updates:

```bash
jq '.agents.<agent>' cto-config.json
jq '.agents.<agent>' infra/charts/cto/cto-config.json
rg -n "<agent>|5DLabs-<Agent>" crates/pm crates/controller docs/scripts/setup-linear-agents.sh infra/gitops/manifests/external-secrets/cto-secrets.yaml infra/charts/cto/values.yaml
```

## 11. Deploy and post-deploy checks

1. Sync `external-secrets-config`
2. Sync `cto`
3. Verify secret materialization and PM app readiness

Example checks:

```bash
kubectl get externalsecrets -n cto | rg "<agent>"
kubectl get secrets -n cto | rg "<agent>"
kubectl logs -n cto deploy/pm-server --since=10m | rg "LINEAR_APP_<AGENT>|<agent>"
```

## Common pitfalls

- Added agent in config, but missed controller mapping in `templates.rs`
- Added GitHub app secret but not Linear app secret (or vice versa)
- Updated root `cto-config.json` but not `infra/charts/cto/cto-config.json`
- Forgot to extend `AGENT_NAMES` in PM, so Linear env vars are ignored
- Forgot to update `docs/scripts/setup-linear-agents.sh` list, so onboarding tooling skips the agent

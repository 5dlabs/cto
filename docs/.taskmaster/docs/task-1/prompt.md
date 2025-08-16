# Task 1: Implement Helm Values and Agents ConfigMap for Multi-Agent Orchestration

## Objective

Implement comprehensive Helm configuration management for a multi-agent orchestration system. Create Helm values, templates, and ConfigMaps to manage agent personas' system prompts.

Important:
- First, perform a full evaluation of existing functionality and patterns (e.g., current system prompt rendering in `infra/charts/controller`, existing agent entries in `values.yaml`, and existing GitHub App secrets/ExternalSecrets in `infra/secret-store`). Strictly follow these patterns and extend only; do not reinvent.
- Do NOT re-implement functionality that already exists. Extend the existing assets instead.
- Charts live under `infra/charts/controller`. Do not create a new chart.
- Testing/verification is via Argo CD sync and kubectl inspection, not local `helm upgrade`.
- Scope: Rust-only. Multi-language support is explicitly out of scope for now.

## Context

You are implementing the configuration foundation for a multi-agent system where different AI agents (Rex, Clippy, QA, Triage, Security) each have unique personas and responsibilities. These agents need their system prompts properly managed through Kubernetes ConfigMaps.
Friendly names mapping (use consistently across docs and values):
- Clippy → Cleo
- QA → Tess
- Triage → Stitch
- Security → Onyx


Reality alignment with our platform:
- Helm is used only to render agent prompts and agent metadata into the `controller-agents` ConfigMap (see `infra/charts/controller/templates/agents-configmap.yaml`).
- Orchestration is via Argo Workflows + Events and the existing CodeRun/DocsRun CRDs. We do not introduce new orchestration charts.
- Charts are installed and reconciled by Argo CD. Testing is done by pushing a PR and observing Argo CD sync/health, not by local `helm upgrade`.

What to do in this task:
- Create new GitHub Apps for Clippy, QA, Triage, and Security using admin env vars (KUBECONFIG_B64, ARGOCD_SERVER/USERNAME/PASSWORD, GITHUB_ADMIN_TOKEN). Store their credentials in the secret store and materialize them via ExternalSecrets in `agent-platform` with names `github-app-5dlabs-{clippy,qa,triage,security}`.
- Wire the new Apps into `infra/charts/controller/values.yaml` under `.Values.agents`.
- Improve each agent's system prompt to be more technical/specific (see Guidance below).
- Define friendly agent names to match Morgan/Rex style and document the mapping (Clippy → "Cleo", QA → "Tess", Triage → "Stitch", Security → "Onyx").
- Note: Token generation is already fully implemented in the container template (`infra/charts/controller/claude-templates/code/container.sh.hbs`) - no changes needed there.

Guidance: Draft system prompts (paste into `infra/charts/controller/values.yaml` under `.Values.agents[*].systemPrompt`)

- Clippy (Cleo):
  - Purpose: formatting, lint fixes, and pedantic conformance ONLY. Never change runtime behavior.
  - Rust focus:
    - Enforce cargo fmt; rustfmt defaults; no custom style deviations
    - Run cargo clippy with `-W clippy::all -W clippy::pedantic` and achieve ZERO warnings
    - Prefer explicit types; avoid unnecessary clones; leverage borrowing idioms
    - Forbid unsafe unless pre-existing and justified; never introduce new unsafe
    - Do not refactor or reorder logic; produce minimal, mechanical diffs
  - If any change would alter semantics, STOP and propose a PR comment instead

- QA (Tess):
  - You ONLY add tests and test scaffolding; you never change implementation code.
  - Rust testing practice:
    - Prefer unit/integration tests in Rust; clear arrange-act-assert
    - Avoid flakiness (no sleeps unless necessary; use retries with bounds)
  - Kubernetes verification:
    - Prove behavior with concrete logs/requests/responses and expected outputs
    - Store artifacts predictably; link evidence in PR comments
  - CI/CD execution (required):
    - Use GitHub Actions to build and push an image for the changes (e.g., GHCR)
    - Deploy the image to the cluster (apply manifests/Helm) in a test namespace
    - Run an extensive regression suite against the deployed service based on the task’s acceptance criteria
    - Publish logs/evidence/artifacts; mark pass/fail clearly, and approve PR only if all criteria pass
  - Approve PRs when acceptance criteria are proven; do not merge.

- Triage (Stitch):
  - Focus on reproducing CI failures and making the SMALLEST viable fix to turn red → green.
  - Scope control:
    - Update tests if they are wrong; otherwise touch the fewest lines possible
    - Avoid broad refactors and stylistic changes; keep diffs surgical

- Security (Onyx):
  - Read security reports (CodeQL, Dependabot). Apply least-privilege remediations.
  - Avoid introducing new secrets; remove accidental secret exposure
  - Document CVE references, affected packages, version ranges, and remediation rationale in PR body

## Requirements

### 1. Helm Chart Structure
- Use existing chart under `infra/charts/controller`
- Do not create a new chart; extend values and prompts
- Managed by Argo CD; avoid local Helm installs

### 2. Values Configuration
Update `infra/charts/controller/values.yaml` (map/object keys with friendly names in `name` field):
```yaml
agents:
  rex:
    name: "Rex"
    githubApp: "5DLabs-Rex"
    systemPrompt: |
      ...
  clippy:
    name: "Cleo"
    githubApp: "5DLabs-Clippy"
    systemPrompt: |
      ...
  qa:
    name: "Tess"
    githubApp: "5DLabs-QA"
    systemPrompt: |
      ...
  triage:
    name: "Stitch"
    githubApp: "5DLabs-Triage"
    systemPrompt: |
      ...
  security:
    name: "Onyx"
    githubApp: "5DLabs-Security"
    systemPrompt: |
      ...
```

Testing via Argo CD:
- Open a PR with the updated prompts/values.
- Allow Argo CD to sync the controller chart and render the new `controller-agents` ConfigMap.
- Verify with:
  - `kubectl -n agent-platform get cm controller-agents -o yaml`
  - `argocd app get controller` (Health: Healthy, Sync: Synced)

### 3. JSON Schema Validation
Create `values.schema.json` to validate the values structure with proper type checking and required fields.

### 4. Helper Templates
Implement in `templates/_helpers.tpl`:
- `platform.renderPrompt`: Renders agent prompt files from chart files
- `platform.agentVolumes`: Defines volumes for ConfigMaps
- `platform.agentVolumeMounts`: Defines volume mounts for containers

### 5. ConfigMap Templates

#### Controller Agents ConfigMap
- Template: `templates/controller-agents-configmap.yaml`
- Renders all agent system prompts from `files/agents/*.md`
- Each prompt accessible by its filename as ConfigMap key

 

### 6. Values Updates and Secrets

#### Agent Prompts
Update prompts inline under `.Values.agents[*].systemPrompt` in `infra/charts/controller/values.yaml` (no chart files created):
- `rex_system-prompt.md` content → `.Values.agents[name==rex].systemPrompt`
- `clippy_system-prompt.md` content → `.Values.agents[name==clippy].systemPrompt`
- `qa_system-prompt.md` content → `.Values.agents[name==qa].systemPrompt`
- `triage_system-prompt.md` content → `.Values.agents[name==triage].systemPrompt`
- `security_system-prompt.md` content → `.Values.agents[name==security].systemPrompt`

#### ExternalSecrets (update existing manifests)
- Extend `infra/secret-store/agent-secrets-external-secrets.yaml` to include:
  - `github-app-5dlabs-clippy`
  - `github-app-5dlabs-qa`
  - `github-app-5dlabs-triage`
  - `github-app-5dlabs-security`
- Ensure each target Secret exposes `appId` and `privateKey` keys.
- Confirm `ClusterSecretStore` is `secret-store` and namespace is `agent-platform`.
- The existing container template will automatically handle token generation using these secrets - no additional implementation needed.

### 7. Smoke Test WorkflowTemplate (optional)
Create `templates/workflowtemplates/agent-mount-smoke.yaml` to validate mount points:
- Must verify `/etc/agents` directory exists and contains prompt files
- Should output "OK" on success

### 8. Documentation
Update or create `docs/.taskmaster/architecture.md` with:
- Overview of agents ConfigMap configuration
- How to override values per environment
- Mount points and file locations
- Testing and validation procedures

## Implementation Steps

1. **Create chart structure**:
   ```bash
   # N/A: chart already exists under infra/charts/controller
   ```

2. **Implement core updates**:
   - Update `infra/charts/controller/values.yaml` agents
   - (Optional) templates/workflowtemplates/agent-mount-smoke.yaml

3. **Validate implementation**:
   ```bash
   # Use Argo CD instead of local Helm
   argocd app sync controller && argocd app get controller
   ```

4. **Test deployment**:
   ```bash
   kubectl create ns dev || true
   # Use Argo Workflows/DocsRun/CodeRun to validate mounts; avoid local Helm
   kubectl -n dev get cm controller-agents
   kubectl -n dev create wf --from=wftmpl/agent-mount-smoke
   ```

## Success Criteria

- [ ] All Helm templates render without errors
- [ ] `helm lint` passes with no warnings
- [ ] `controller-agents` ConfigMap updated with new agent prompts
- [ ] Smoke test WorkflowTemplate executes and outputs "OK"
- [ ] All agent prompts accessible at `/etc/agents/*.md`
- [ ] Values can be overridden with `-f` or `--set`
- [ ] Total ConfigMap size < 900KB
- [ ] All files UTF-8 encoded

## Important Constraints

1. **ConfigMap Size**: Keep total size under 900KB (Kubernetes limit ~1MiB)
2. **Naming**: Use chart-scoped names (platform.*) for helpers
3. **Indentation**: Careful with nindent values in templates
4. **Fast-fail**: Missing files should fail at helm template time
5. **Compatibility**: Ensure compatibility with Argo Workflows pod specs

## Error Handling

- If prompt files are missing: Helm template should fail immediately
- If values validation fails: Helm lint should catch schema violations
- If ConfigMap too large: Consider splitting or compressing prompts
- If mounts fail: Smoke test will catch and report the issue

## Testing Commands Reference

```bash
# Linting
argocd app get controller

# Size check
echo "Prompts are inline in values; ensure aggregated size stays under ConfigMap limits"

# Encoding check (if needed for any local prompt assets)
echo "Prompts are inline in values"

# Deployment test
echo "Use Argo workflows to validate mounts and behavior"

# Smoke test
kubectl -n dev create wf --from=wftmpl/agent-mount-smoke
argo -n dev logs @latest
```

## Notes

- This task sets up the configuration foundation for all subsequent tasks
- Agent prompts will be refined as the system evolves

- Environment-specific overrides use standard Helm values patterns

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

You are implementing the configuration foundation for a multi-agent system where different AI agents each have unique personas and responsibilities. These agents need their system prompts properly managed through Kubernetes ConfigMaps.

The new agents to add (using friendly names like our existing Rex, Morgan, Blaze, Cipher):
- **Cleo**: Formatting & code quality specialist
- **Tess**: QA & testing specialist  
- **Stitch**: CI/CD triage specialist
- **Onyx**: Security specialist


Reality alignment with our platform:
- Helm is used only to render agent prompts and agent metadata into the `controller-agents` ConfigMap (see `infra/charts/controller/templates/agents-configmap.yaml`).
- Orchestration is via Argo Workflows + Events and the existing CodeRun/DocsRun CRDs. We do not introduce new orchestration charts.
- Charts are installed and reconciled by Argo CD. Testing is done by pushing a PR and observing Argo CD sync/health, not by local `helm upgrade`.

What to do in this task:
- Create new GitHub Apps (5DLabs-Clippy, 5DLabs-QA, 5DLabs-Triage, 5DLabs-Security) using admin env vars (KUBECONFIG_B64, ARGOCD_SERVER/USERNAME/PASSWORD, GITHUB_ADMIN_TOKEN). Store their credentials in the secret store and materialize them via ExternalSecrets in `agent-platform` with names `github-app-5dlabs-{clippy,qa,triage,security}`.
- Add the new agents to `infra/charts/controller/values.yaml` under `.Values.agents` with keys `clippy`, `qa`, `triage`, `security`.
- Set their friendly names in the `name` field: Cleo, Tess, Stitch, Onyx.
- Add a `role` field describing their specialty (following the existing pattern).
- Write robust, technical system prompts for each agent (see Guidance below).
- Note: Token generation is already fully implemented in the container template (`infra/charts/controller/claude-templates/code/container.sh.hbs`) - no changes needed there.

Guidance: Draft system prompts (paste into `infra/charts/controller/values.yaml` under `.Values.agents.<key>.systemPrompt`)

Follow Anthropic's documentation format with YAML frontmatter. Omit the `tools` field to inherit all available tools.

- Cleo (clippy key):
```yaml
---
name: Cleo
description: Rust formatting and code quality specialist. Ensures zero Clippy warnings and perfect rustfmt compliance. Use for all formatting and lint fixes.
# tools: omitted to inherit all available tools
---

You are Cleo, a meticulous Rust code quality specialist with a maniacal focus on achieving ZERO Clippy warnings.

When invoked:
1. Run `cargo fmt --all -- --check` to identify formatting issues
2. Run `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic`
3. Fix ALL issues found - no exceptions

Your strict rules:
- Enforce cargo fmt with default rustfmt settings - no custom deviations
- Achieve ZERO Clippy warnings with pedantic lints enabled
- Prefer explicit types over inference where it improves clarity
- Eliminate unnecessary clones - leverage borrowing and references
- Forbid unsafe code unless pre-existing and justified
- Never refactor logic - only formatting and lint fixes
- Produce minimal, mechanical diffs

If any change would alter program semantics or behavior, STOP immediately and create a PR comment explaining why the fix cannot be applied safely.
```

- Tess (qa key):
```yaml
---
name: Tess
description: Quality assurance and testing specialist. Writes comprehensive tests and validates acceptance criteria. Never modifies implementation code.
# tools: omitted to inherit all available tools
---

You are Tess, a rigorous QA specialist who ONLY adds tests and test scaffolding. You never modify implementation code.

When invoked:
1. Review the task's acceptance criteria thoroughly
2. Identify all untested code paths and scenarios
3. Write comprehensive test coverage immediately

Testing requirements:
- Write unit and integration tests following arrange-act-assert pattern
- Achieve high coverage (≥95% target, ~100% on critical paths)
- Avoid flaky tests - no arbitrary sleeps, use proper synchronization
- Test both happy paths and edge cases exhaustively

Kubernetes validation process:
1. Build and push test image to GHCR
2. Deploy to test namespace in cluster
3. Run full regression suite against deployed service
4. Capture concrete evidence: logs, requests, responses
5. Document results in PR with links to artifacts

Approval criteria:
- All acceptance criteria validated through actual tests
- Test evidence clearly documented
- No regressions detected
- Approve PR when all tests pass (but never merge)
```

- Stitch (triage key):
```yaml
---
name: Stitch
description: CI/CD triage and remediation specialist. Fixes failing builds with minimal, surgical changes. Focus on turning red tests green.
# tools: omitted to inherit all available tools
---

You are Stitch, a CI/CD triage specialist focused on fixing failures with surgical precision.

When invoked:
1. Examine CI failure logs immediately
2. Reproduce the failure locally
3. Apply the SMALLEST possible fix

Triage principles:
- Make minimal changes - touch the fewest lines possible
- Fix the immediate problem only
- Update tests if they're wrong, fix code if it's broken
- No refactoring or style changes
- Keep diffs surgical and focused
- Document the root cause in your commit message

Your goal: Turn red → green with minimal disruption.
```

- Onyx (security key):
```yaml
---
name: Onyx
description: Security and vulnerability specialist. Remediates security issues, removes exposed secrets, and applies least-privilege fixes.
# tools: omitted to inherit all available tools
---

You are Onyx, a security specialist focused on identifying and remediating vulnerabilities.

When invoked:
1. Review security scan reports (CodeQL, Dependabot, cargo-audit)
2. Prioritize by severity: Critical → High → Medium → Low
3. Apply fixes immediately

Security requirements:
- Apply least-privilege principle to all remediations
- Never introduce new secrets or credentials
- Remove any accidentally exposed secrets immediately
- Update vulnerable dependencies to secure versions
- Add input validation where missing
- Implement proper error handling that doesn't leak information

Documentation requirements:
- List all CVE numbers addressed
- Specify affected packages and version ranges
- Explain remediation approach
- Note any breaking changes or compatibility impacts

Your fixes must be secure, minimal, and well-documented.
```

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

### 0. **CRITICAL: Verify Access Before Starting**

Before writing any code, verify you have access to both kubectl and Argo CD:

```bash
# Verify kubectl access
kubectl cluster-info
kubectl get nodes
kubectl -n agent-platform get pods

# If kubectl fails, check your KUBECONFIG environment variable
echo $KUBECONFIG_B64 | base64 -d > /tmp/kubeconfig
export KUBECONFIG=/tmp/kubeconfig
kubectl cluster-info

# Verify Argo CD access
argocd version --client
argocd login $ARGOCD_SERVER --username $ARGOCD_USERNAME --password $ARGOCD_PASSWORD
argocd app list
argocd app get controller

# If these commands fail, STOP and report the issue - you cannot proceed without cluster access
```

### 1. **Create GitHub Apps**:
   Use the GitHub web UI or API to create the new GitHub Apps (5DLabs-Clippy, 5DLabs-QA, 5DLabs-Triage, 5DLabs-Security)

### 2. **Implement core updates**:
   - Update `infra/charts/controller/values.yaml` agents
   - Add ExternalSecrets to `infra/secret-store/agent-secrets-external-secrets.yaml`

### 3. **Validate implementation**:
   ```bash
   # Use Argo CD to sync and verify
   argocd app sync controller
   argocd app get controller
   kubectl -n agent-platform get cm controller-agents -o yaml
   ```

### 4. **Test deployment**:
   ```bash
   # Verify the ConfigMap contains new agents
   kubectl -n agent-platform get cm controller-agents -o yaml | grep -E "Cleo|Tess|Stitch|Onyx"
   
   # Check ExternalSecrets are synced
   kubectl -n agent-platform get externalsecrets | grep github-app-5dlabs
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

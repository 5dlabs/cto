# Task 1: Helm Values and Agents ConfigMap for Personas

## Overview

This task implements the foundational configuration management for the multi-agent orchestration system. It establishes Helm values and templates to render agent personas' system prompts into ConfigMaps.

Platform alignment:
- Do NOT re-implement functionality that already exists. Extend existing assets instead.
- We already install and manage charts with Argo CD. Helm is used for prompts/config only; orchestration is in Argo Workflows/Events with CodeRun/DocsRun CRDs.
- We already have `agents-configmap.yaml` that renders `agents.yaml` and per-agent `*_system-prompt.md` from `.Values.agents.<key>.systemPrompt`. Do not create a new chart.
- This task focuses on improving prompts, adding the Cleo/Tess/Stitch agents, and wiring names consistently.
- Scope: Rust-only. Multi-language support is out of scope for this phase.
 - Administrative operations (creating apps/resources in GitHub/Kubernetes/Argo CD) must use the `agent-admin-secrets` secret; role-specific agent operations use their own GitHub App secrets via ExternalSecrets.

## Technical Context

The system uses multiple specialized AI agents, each with distinct personas and responsibilities. We standardize GitHub App names as "Persona (5DLabs)" to match current org conventions (e.g., "Rex (5DLabs)", "Morgan (5DLabs)"):
- **Rex**: Primary implementation agent → GitHub App: "Rex (5DLabs)"
- **Cleo**: Formatting and pedantic warnings agent (Clippy role) → GitHub App: "Cleo (5DLabs)"
- **Tess**: Testing-only agent with strict Kubernetes verification requirements (QA role) → GitHub App: "Tess (5DLabs)"
- **Stitch**: CI failure remediation agent (Triage role) → GitHub App: "Stitch (5DLabs)"
- **Security**: Security vulnerability remediation role → GitHub App: reuse existing "Cipher (5DLabs)" (no new Security app)

Each agent requires:
1. A unique system prompt defining its persona and constraints
2. Proper mounting of these prompts into workflow pods

## Secret Management and GitHub App Authentication

Do NOT re-implement functionality that already exists. Extend the existing assets instead.

### Admin Secret (for administrative operations)
- Use a single admin secret named `agent-admin-secrets` for administrative operations (cluster and Argo CD administration, and a repo/org-level GitHub admin PAT when absolutely necessary).
- Expected keys (already documented in `docs/requirements.yaml`): `KUBECONFIG_B64`, `ARGOCD_SERVER`, `ARGOCD_USERNAME`/`ARGOCD_PASSWORD` (or `ARGOCD_AUTH_TOKEN`), and `GITHUB_ADMIN_TOKEN`.
- This secret is NOT used by role-specific agents; those use their own GitHub App secrets via ExternalSecrets.

### External Secrets for GitHub Apps (per-agent credentials)
- Extend existing ExternalSecrets under `infra/secret-store/agent-secrets-external-secrets.yaml` (do not create a new file) to add the three new agents introduced in this task.
- Existing pattern already covers several apps (Rex/Blaze/Morgan/Cipher). Add entries for the new ones:
  - Cleo (GitHub App: "Cleo (5DLabs)")
  - Tess (GitHub App: "Tess (5DLabs)")
  - Stitch (GitHub App: "Stitch (5DLabs)")
- For the Security role, reuse the existing GitHub App: "Cipher (5DLabs)" (no new app to create)
- Follow the established naming and `ClusterSecretStore` reference (our cluster-wide store is `secret-store`). Target Kubernetes `Secret`s should contain at least:
  - `appId`: GitHub App ID
  - `privateKey`: GitHub App private key (PEM)

### Token Generation (already implemented)
The container template (`infra/charts/controller/claude-templates/code/container.sh.hbs`) already handles all GitHub App authentication and token generation. No changes needed - the existing pattern will automatically work with the new agents once their ExternalSecrets are in place.

## Prerequisites

Before starting implementation, verify access to required tools:
- **kubectl**: Must have access to the Kubernetes cluster (via `KUBECONFIG_B64` environment variable)
- **Argo CD CLI**: Must be able to login and sync applications (via `ARGOCD_SERVER`, `ARGOCD_USERNAME`, `ARGOCD_PASSWORD`)
- **Argo Workflows CLI**: Available at `/usr/local/bin/argo` - test with `argo version --short` (should show v3.7.1+)
- **GitHub CLI or API access**: For creating GitHub Apps (via `GITHUB_ADMIN_TOKEN`)

If any of these tools are not accessible, STOP and report the issue before proceeding.

**IMPORTANT**: The Argo Workflows CLI (`argo`) is different from Argo CD CLI (`argocd`). Both are available in the container.

## PREREQUISITE: GitHub Apps Must Exist

**ASSUMPTION**: GitHub Apps follow the naming convention "Persona (5DLabs)" and are configured at the organization level.

### Required GitHub Apps (Pre-created or to be created manually):
1. **Cleo (5DLabs)** - Code quality and formatting specialist
2. **Tess (5DLabs)** - Quality assurance and testing specialist  
3. **Stitch (5DLabs)** - CI/CD failure remediation specialist
4. **Cipher (5DLabs)** - Security vulnerability remediation specialist (already exists; reused for Security role)

### GitHub App Validation Requirements:
- **Verify Existence**: Apps must be visible at `https://github.com/organizations/5dlabs/settings/apps`
- **Verify Installation**: Apps must be installed on the organization with repository access
- **Verify Permissions**: Each app must have proper repository permissions (Contents: Read/Write, Pull Requests: Read/Write, Issues: Read/Write)
- **Verify Credentials**: App ID and private key must be available in the external secret store

**TASK SCOPE**: This task focuses on Helm configuration, ExternalSecrets setup, and system integration. GitHub App creation is handled separately as a manual prerequisite.

## Implementation Guide

### Phase 1: Chart Structure Setup

#### 1.1 Define Helm Values Schema

**Location**: `infra/charts/controller/values.yaml` (existing)

Use the existing map/object structure under `.Values.agents` (keys are agent identifiers; the `name` field is the friendly display name). The `githubApp` must be the exact GitHub App name using the convention "Persona (5DLabs)":

```yaml
agents:
  rex:
    name: "Rex"
    githubApp: "Rex (5DLabs)"
    role: "Senior Backend Architect & Systems Engineer"
    systemPrompt: |
      # Rex system prompt (truncated)
      ...

  clippy:
    name: "Cleo"
    githubApp: "Cleo (5DLabs)"
    role: "Formatting & Code Quality Specialist"
    systemPrompt: |
      # Cleo system prompt (truncated)
      ...

  qa:
    name: "Tess"
    githubApp: "Tess (5DLabs)"
    role: "Quality Assurance & Testing Specialist"
    systemPrompt: |
      # Tess system prompt (truncated)
      ...

  triage:
    name: "Stitch"
    githubApp: "Stitch (5DLabs)"
    role: "CI/CD Triage & Remediation Specialist"
    systemPrompt: |
      # Stitch system prompt (truncated)
      ...

  security:
    name: "Onyx"
    githubApp: "Cipher (5DLabs)"  # Reuse existing app for security role
    role: "Security & Vulnerability Specialist"
    systemPrompt: |
      # Onyx system prompt (truncated)
      ...
```

#### 1.2 Add JSON Schema Validation

If needed, update schema validation to reflect added fields (optional for now).

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "agents": {
      "type": "object",
      "additionalProperties": {
        "type": "object",
        "required": ["name", "githubApp", "systemPrompt"],
        "properties": {
          "name": {"type": "string", "minLength": 1},
          "githubApp": {"type": "string", "minLength": 1},
          "systemPrompt": {"type": "string", "minLength": 1}
        }
      }
    }
  },
  "required": ["agents"]
}
```

### Phase 2: Helper Templates

#### 2.1 Create Helm Helper Functions

No new helpers required; we’re extending existing values/prompts.

```yaml
{{- define "platform.renderPrompt" -}}
{{- $f := printf "agents/%s" .systemPromptFile -}}
{{- .Files.Get $f | nindent 6 -}}
{{- end -}}

{{- define "platform.agentVolumes" -}}
- name: agents-prompts
  configMap:
    name: controller-agents
{{- end -}}

{{- define "platform.agentVolumeMounts" -}}
- name: agents-prompts
  mountPath: /etc/agents
  readOnly: true
{{- end -}}
```

### Phase 3: ConfigMap Templates

#### 3.1 Controller Agents ConfigMap

Already implemented as `infra/charts/controller/templates/agents-configmap.yaml`.

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: controller-agents
  labels:
    app.kubernetes.io/name: controller-agents
    app.kubernetes.io/part-of: platform
data:
{{- range .Values.agents }}
  {{ .systemPromptFile }}: |-
{{ include "platform.renderPrompt" . }}
{{- end }}
```

 

### Phase 4: Package Files

#### 4.1 Agent Prompt Files

Prompts are supplied inline via `.Values.agents.<key>.systemPrompt` in `infra/charts/controller/values.yaml`. 

Each system prompt should follow the Anthropic documentation format with YAML frontmatter:
```yaml
---
name: AgentName
description: Brief description of the agent's role and when to use it
# tools: omitted to inherit all available tools
---

[Agent's detailed system prompt content here]
```

The prompts will be rendered using the GitHub App name for each agent, for example:
- `Rex (5DLabs)_system-prompt.md`
- `Cleo (5DLabs)_system-prompt.md`
- `Tess (5DLabs)_system-prompt.md`
- `Stitch (5DLabs)_system-prompt.md`
- `Cipher (5DLabs)_system-prompt.md` (Security)

 

### Phase 5: WorkflowTemplate Integration

#### 5.1 Smoke Test Template

Optional. Validation should primarily be via Argo CD sync + checking rendered ConfigMap content.

```yaml
apiVersion: argoproj.io/v1alpha1
kind: WorkflowTemplate
metadata:
  name: agent-mount-smoke
spec:
  entrypoint: main
  templates:
    - name: main
      container:
        image: alpine:3.20
        command: ['sh', '-c']
        args: ['ls -l /etc/agents && echo OK']
        volumeMounts:
{{ include "platform.agentVolumeMounts" . | nindent 10 }}
  volumes:
{{ include "platform.agentVolumes" . | nindent 4 }}
```

## Testing Strategy

### Unit Testing
```bash
Using Argo CD (source of truth):
```bash
argocd app sync controller
argocd app get controller
kubectl -n agent-platform get cm controller-agents -o yaml | head -n 80
```
```

### Integration Testing
```bash
Use Workflows that mount the `controller-agents` ConfigMap and verify the path `/etc/agents/${GITHUB_APP}_system-prompt.md` resolves. `${GITHUB_APP}` must equal the exact GitHub App name (e.g., `Rex (5DLabs)`).
```

### Validation Checks
```bash
# Prompts are inline in values; ensure combined prompt content stays well under the ConfigMap size limit (~1MiB)
echo "Verify prompt sizes in values.yaml if adding large blocks"
```

## Dependencies

- Kubernetes cluster with Argo Workflows installed
- Helm 3.x
- External Secrets Operator (for GitHub App secrets)
- Proper RBAC permissions for ConfigMap creation

Additionally for secrets:
- ClusterSecretStore set up (`secret-store`)
- Access to external secret backend with GitHub App credentials

## Risk Mitigation

### ConfigMap Size Limits
- **Risk**: Kubernetes ConfigMaps have a ~1MiB size limit
- **Mitigation**: Monitor combined prompt sizes, implement compression if needed

### Helper Name Collisions
- **Risk**: Template helper names might conflict with other charts
- **Mitigation**: Use chart-scoped naming (platform.*)

### Missing Files
- **Risk**: Missing prompt files break helm template/install
- **Mitigation**: Helm's .Files.Get fails fast at template time

### Indentation Errors
- **Risk**: Incorrect nindent values break YAML rendering
- **Mitigation**: Comprehensive helm template testing, use consistent nindent values

## Architecture Integration

This task establishes the configuration foundation for the multi-agent orchestration system:

1. **Agent Personas**: Each agent's system prompt defines its role, constraints, and behavior
2. **Per-agent Authentication**: GitHub App credentials delivered via ExternalSecrets; tokens minted in-container
3. **Workflow Integration**: Volume mounts make prompts and tokens available to workflow pods
4. **Environment Flexibility**: Helm values allow per-environment customization

## Next Steps

After completing this task:
1. GitHub App ExternalSecrets and in-container token minting are included here (Task 2 merged into Task 1). Any monitoring/rotation hardening can be a follow-up.
2. Task 3 extends existing workflow templates (we already have them) rather than creating new ones; focus on parameter simplification and prompt mounts.
3. Task 4 will implement event sensors that trigger workflows with appropriate agent selections

## References

- [Product Requirements Document](../prd.txt)
- [Architecture Document](../architecture.md)
- [Helm Best Practices](https://helm.sh/docs/chart_best_practices/)
- [Kubernetes ConfigMap Documentation](https://kubernetes.io/docs/concepts/configuration/configmap/)
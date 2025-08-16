# Task 1: Helm Values and Agents ConfigMap for Personas

## Overview

This task implements the foundational configuration management for the multi-agent orchestration system. It establishes Helm values and templates to render agent personas' system prompts into ConfigMaps.

Platform alignment:
- Do NOT re-implement functionality that already exists. Extend existing assets instead.
- We already install and manage charts with Argo CD. Helm is used for prompts/config only; orchestration is in Argo Workflows/Events with CodeRun/DocsRun CRDs.
- We already have `agents-configmap.yaml` that renders `agents.yaml` and per-agent `*_system-prompt.md` from `.Values.agents.<key>.systemPrompt`. Do not create a new chart.
- This task focuses on improving prompts, adding the Clippy GitHub App, and wiring names consistently.
- Scope: Rust-only. Multi-language support is out of scope for this phase.
 - Administrative operations (creating apps/resources in GitHub/Kubernetes/Argo CD) must use the `agent-admin-secrets` secret; role-specific agent operations use their own GitHub App secrets via ExternalSecrets.

## Technical Context

The system uses multiple specialized AI agents, each with distinct personas and responsibilities:
- **Rex**: Primary implementation agent
- **Clippy**: Formatting and pedantic warnings agent  
- **QA**: Testing-only agent with strict Kubernetes verification requirements
- **Triage**: CI failure remediation agent
- **Security**: Vulnerability remediation agent

Friendly names mapping (use consistently across docs and values):
- Clippy → Cleo
- QA → Tess
- Triage → Stitch
- Security → Onyx

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
- Extend existing ExternalSecrets under `infra/secret-store/agent-secrets-external-secrets.yaml` (do not create a new file) to add the four new agents introduced in this task.
- Existing pattern already covers several apps (e.g., Rex/Blaze/Morgan/Cipher). Add entries for the new ones:
  - Clippy
  - QA
  - Triage
  - Security
- Follow the established naming and `ClusterSecretStore` reference (our cluster-wide store is `secret-store`). Target Kubernetes `Secret`s should contain at least:
  - `appId`: GitHub App ID
  - `privateKey`: GitHub App private key (PEM)

### Token Generation Flow (existing pattern)
Containers mint GitHub App installation tokens inside the container. No separate token microservice.
1. InitContainer (or entrypoint) reads `appId`/`privateKey` from the mounted Secret
2. Creates RS256 JWT for the GitHub App
3. Exchanges JWT for an installation access token
4. Writes token to a shared volume at `/var/run/github/token` with 0600 permissions
5. Main container reads token from `/var/run/github/token`

### Workflow Integration Pattern (reference)
Ensure workflows mount the per-agent Secret and expose a shared volume for the token file.
```yaml
initContainers:
- name: gh-token
  image: ghcr.io/5dlabs/cto/runtime:latest
  env:
    - name: APP_ID
      valueFrom:
        secretKeyRef:
          name: github-app-<agent>
          key: appId
    - name: PRIVATE_KEY
      valueFrom:
        secretKeyRef:
          name: github-app-<agent>
          key: privateKey
    - name: OUTPUT_PATH
      value: /var/run/github/token
  volumeMounts:
    - name: github-tmp
      mountPath: /var/run/github

containers:
- name: runner
  env:
    - name: GITHUB_TOKEN_FILE
      value: /var/run/github/token
  volumeMounts:
    - name: github-tmp
      mountPath: /var/run/github
```

## Implementation Guide

### Phase 1: Chart Structure Setup

#### 1.1 Define Helm Values Schema

**Location**: `infra/charts/controller/values.yaml` (existing)

Use the existing map/object structure under `.Values.agents` (keys are agent identifiers; the `name` field is the friendly display name):

```yaml
agents:
  rex:
    name: "Rex"
    githubApp: "5DLabs-Rex"
    systemPrompt: |
      # Rex system prompt (truncated)
      ...

  clippy:
    name: "Cleo"            # friendly name
    githubApp: "5DLabs-Clippy"
    systemPrompt: |
      # Cleo system prompt (truncated)
      ...

  qa:
    name: "Tess"            # friendly name
    githubApp: "5DLabs-QA"
    systemPrompt: |
      # Tess system prompt (truncated)
      ...

  triage:
    name: "Stitch"          # friendly name
    githubApp: "5DLabs-Triage"
    systemPrompt: |
      # Stitch system prompt (truncated)
      ...

  security:
    name: "Onyx"            # friendly name
    githubApp: "5DLabs-Security"
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

Prompts are supplied inline via `.Values.agents[*].systemPrompt` in `infra/charts/controller/values.yaml`. Update those strings to the improved versions below.
- `rex_system-prompt.md`
- `clippy_system-prompt.md`
- `qa_system-prompt.md`
- `triage_system-prompt.md`
- `security_system-prompt.md`

 

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
Use Workflows that mount the `controller-agents` ConfigMap and verify the path `/etc/agents/${GITHUB_APP}_system-prompt.md` resolves.
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
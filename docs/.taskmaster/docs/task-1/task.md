# Task 1: Helm Values and Agents ConfigMap for Personas and Project-wide Tools

## Overview

This task implements the foundational configuration management for the multi-agent orchestration system. It establishes Helm values and templates to render agent personas' system prompts into ConfigMaps and defines project-wide MCP (Model Context Protocol) tools configuration.

Platform alignment:
- Do NOT re-implement functionality that already exists. Extend existing assets instead.
- We already install and manage charts with Argo CD. Helm is used for prompts/config only; orchestration is in Argo Workflows/Events with CodeRun/DocsRun CRDs.
- We already have `agents-configmap.yaml` that renders `agents.yaml` and per-agent `*_system-prompt.md` from `.Values.agents[*].systemPrompt`. Do not create a new chart.
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
2. Access to project-wide MCP tools configuration
3. Proper mounting of these configurations into workflow pods

## Implementation Guide

### Phase 1: Chart Structure Setup

#### 1.1 Define Helm Values Schema

**Location**: `infra/charts/controller/values.yaml` (existing)

```yaml
agents:
  - name: rex
    githubApp: rex-agent
    systemPromptFile: rex_system-prompt.md
  - name: clippy
    githubApp: clippy-agent
    systemPromptFile: clippy_system-prompt.md
  - name: qa
    githubApp: qa-agent
    systemPromptFile: qa_system-prompt.md
  - name: triage
    githubApp: triage-agent
    systemPromptFile: triage_system-prompt.md
  - name: security
    githubApp: security-agent
    systemPromptFile: security_system-prompt.md

mcp:
  requirementsFile: requirements.yaml
```

#### 1.2 Add JSON Schema Validation

If needed, update schema validation to reflect added fields (optional for now).

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "agents": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["name", "githubApp", "systemPromptFile"],
        "properties": {
          "name": {"type": "string", "minLength": 1},
          "githubApp": {"type": "string", "minLength": 1},
          "systemPromptFile": {"type": "string", "pattern": "^.+\\.md$"}
        }
      },
      "minItems": 1
    },
    "mcp": {
      "type": "object",
      "properties": {
        "requirementsFile": {"type": "string", "minLength": 1}
      },
      "required": ["requirementsFile"]
    }
  },
  "required": ["agents", "mcp"]
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
- name: mcp-requirements
  configMap:
    name: mcp-requirements
{{- end -}}

{{- define "platform.agentVolumeMounts" -}}
- name: agents-prompts
  mountPath: /etc/agents
  readOnly: true
- name: mcp-requirements
  mountPath: /work/requirements.yaml
  subPath: requirements.yaml
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

#### 3.2 MCP Requirements (out of scope)

Project-level MCP tools are out of scope for Task 1 and handled in a later task.

### Phase 4: Package Files

#### 4.1 Agent Prompt Files

Prompts are supplied inline via `.Values.agents[*].systemPrompt` in `infra/charts/controller/values.yaml`. Update those strings to the improved versions below.
- `rex_system-prompt.md`
- `clippy_system-prompt.md`
- `qa_system-prompt.md`
- `triage_system-prompt.md`
- `security_system-prompt.md`

#### 4.2 MCP Requirements

Project-level MCP requirements live at `docs/requirements.yaml`. Extend it to define a safe admin tool that reads `${GITHUB_ADMIN_TOKEN}` from env (sourced via External Secrets) rather than embedding tokens.

```yaml
tools:
  - name: github-comments
    transport: http
    endpoint: http://mcp-github-comments:8080
  - name: k8s-verify
    transport: exec
    command: [/bin/kubectl, version, --client]
```

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
2. **MCP Tools**: Project-wide tool configuration enables consistent agent capabilities
3. **Workflow Integration**: Volume mounts make configurations available to all workflow pods
4. **Environment Flexibility**: Helm values allow per-environment customization

## Next Steps

After completing this task:
1. Task 2 will define/extend GitHub App secrets via External Secrets and (optionally) an installation-token helper, and add the new Clippy App.
2. Task 3 extends existing workflow templates (we already have them) rather than creating new ones; focus on parameter simplification and prompt/requirements mounts.
3. Task 4 will implement event sensors that trigger workflows with appropriate agent selections

## References

- [Product Requirements Document](../prd.txt)
- [Architecture Document](../architecture.md)
- [Helm Best Practices](https://helm.sh/docs/chart_best_practices/)
- [Kubernetes ConfigMap Documentation](https://kubernetes.io/docs/concepts/configuration/configmap/)
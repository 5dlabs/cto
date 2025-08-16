# Task 1: Helm Values and Agents ConfigMap for Personas and Project-wide Tools

## Overview

This task implements the foundational configuration management for the multi-agent orchestration system. It establishes Helm values and templates to render agent personas' system prompts into ConfigMaps and defines project-wide MCP (Model Context Protocol) tools configuration.

## Technical Context

The system uses multiple specialized AI agents, each with distinct personas and responsibilities:
- **Rex**: Primary implementation agent
- **Clippy**: Formatting and pedantic warnings agent  
- **QA**: Testing-only agent with strict Kubernetes verification requirements
- **Triage**: CI failure remediation agent
- **Security**: Vulnerability remediation agent

Each agent requires:
1. A unique system prompt defining its persona and constraints
2. Access to project-wide MCP tools configuration
3. Proper mounting of these configurations into workflow pods

## Implementation Guide

### Phase 1: Chart Structure Setup

#### 1.1 Define Helm Values Schema

**Location**: `charts/platform/values.yaml`

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

**Location**: `charts/platform/values.schema.json`

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

**Location**: `charts/platform/templates/_helpers.tpl`

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

**Location**: `charts/platform/templates/controller-agents-configmap.yaml`

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

#### 3.2 MCP Requirements ConfigMap

**Location**: `charts/platform/templates/mcp-requirements-configmap.yaml`

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: mcp-requirements
  labels:
    app.kubernetes.io/name: mcp-requirements
    app.kubernetes.io/part-of: platform
data:
  requirements.yaml: |-
{{ .Files.Get (printf "%s" .Values.mcp.requirementsFile) | nindent 6 }}
```

### Phase 4: Package Files

#### 4.1 Agent Prompt Files

Create system prompt files under `charts/platform/files/agents/`:
- `rex_system-prompt.md`
- `clippy_system-prompt.md`
- `qa_system-prompt.md`
- `triage_system-prompt.md`
- `security_system-prompt.md`

#### 4.2 MCP Requirements

**Location**: `charts/platform/files/requirements.yaml`

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

**Location**: `charts/platform/templates/workflowtemplates/agent-mount-smoke.yaml`

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
        args: ['ls -l /etc/agents && test -f /work/requirements.yaml && echo OK']
        volumeMounts:
{{ include "platform.agentVolumeMounts" . | nindent 10 }}
  volumes:
{{ include "platform.agentVolumes" . | nindent 4 }}
```

## Testing Strategy

### Unit Testing
```bash
# Validate schema
helm lint charts/platform

# Render templates
helm template charts/platform | head -n 50

# Check ConfigMap generation
helm template charts/platform | yq '. | select(.kind=="ConfigMap")' -o yaml
```

### Integration Testing
```bash
# Deploy to dev namespace
kubectl create ns dev || true
helm upgrade --install platform charts/platform -n dev

# Verify ConfigMaps
kubectl -n dev get cm controller-agents -o yaml
kubectl -n dev get cm mcp-requirements -o yaml

# Run smoke test
kubectl -n dev create wf --from=wftmpl/agent-mount-smoke
argo -n dev logs @latest
```

### Validation Checks
```bash
# Size constraints (< 900KB total)
find charts/platform/files/agents -type f -printf '%s\n' | awk '{s+=$1} END {print s}'

# UTF-8 encoding
file -I charts/platform/files/agents/*.md | grep -v 'charset=utf-8'
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
1. Task 2 will create the Orchestrator DAG WorkflowTemplate using these configurations
2. Task 3 will simplify the CodeRun API to leverage these mounted configurations
3. Task 4 will implement event sensors that trigger workflows with appropriate agent selections

## References

- [Product Requirements Document](../prd.txt)
- [Architecture Document](../architecture.md)
- [Helm Best Practices](https://helm.sh/docs/chart_best_practices/)
- [Kubernetes ConfigMap Documentation](https://kubernetes.io/docs/concepts/configuration/configmap/)
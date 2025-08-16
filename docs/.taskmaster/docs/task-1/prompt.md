# Task 1: Implement Helm Values and Agents ConfigMap for Multi-Agent Orchestration

## Objective

Implement comprehensive Helm configuration management for a multi-agent orchestration system. Create Helm values, templates, and ConfigMaps to manage agent personas' system prompts and project-wide MCP tools configuration.

## Context

You are implementing the configuration foundation for a multi-agent system where different AI agents (Rex, Clippy, QA, Triage, Security) each have unique personas and responsibilities. These agents need their system prompts and shared MCP tools configuration properly managed through Kubernetes ConfigMaps.

## Requirements

### 1. Helm Chart Structure
- Work in the `charts/platform` directory (create if not exists)
- Implement proper Helm 3.x patterns and best practices
- Ensure all templates are idempotent and upgradeable

### 2. Values Configuration
Create `values.yaml` with:
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

#### MCP Requirements ConfigMap  
- Template: `templates/mcp-requirements-configmap.yaml`
- Renders project-wide MCP tools configuration
- Source: `files/requirements.yaml`

### 6. File Packaging

#### Agent Prompts
Create placeholder prompt files in `files/agents/`:
- `rex_system-prompt.md`: "You are Rex, the primary implementation agent..."
- `clippy_system-prompt.md`: "You are Clippy, responsible for formatting and pedantic warnings..."
- `qa_system-prompt.md`: "You are QA agent. You can ONLY add tests, never modify implementation..."
- `triage_system-prompt.md`: "You are Triage agent, specialized in CI failure remediation..."
- `security_system-prompt.md`: "You are Security agent, focused on vulnerability remediation..."

#### MCP Requirements
Create `files/requirements.yaml`:
```yaml
tools:
  - name: github-comments
    transport: http
    endpoint: http://mcp-github-comments:8080
  - name: k8s-verify
    transport: exec
    command: [/bin/kubectl, version, --client]
```

### 7. Smoke Test WorkflowTemplate
Create `templates/workflowtemplates/agent-mount-smoke.yaml` to validate mount points:
- Must verify `/etc/agents` directory exists and contains prompt files
- Must verify `/work/requirements.yaml` exists and is readable
- Should output "OK" on success

### 8. Documentation
Update or create `docs/.taskmaster/architecture.md` with:
- Overview of agents ConfigMap and MCP tools configuration
- How to override values per environment
- Mount points and file locations
- Testing and validation procedures

## Implementation Steps

1. **Create chart structure**:
   ```bash
   mkdir -p charts/platform/{templates,files/agents}
   cd charts/platform
   ```

2. **Implement core files in order**:
   - values.yaml
   - values.schema.json
   - templates/_helpers.tpl
   - templates/controller-agents-configmap.yaml
   - templates/mcp-requirements-configmap.yaml
   - files/agents/*.md (all 5 prompts)
   - files/requirements.yaml
   - templates/workflowtemplates/agent-mount-smoke.yaml

3. **Validate implementation**:
   ```bash
   helm lint charts/platform
   helm template charts/platform
   ```

4. **Test deployment**:
   ```bash
   kubectl create ns dev || true
   helm upgrade --install platform charts/platform -n dev
   kubectl -n dev get cm controller-agents mcp-requirements
   kubectl -n dev create wf --from=wftmpl/agent-mount-smoke
   ```

## Success Criteria

- [ ] All Helm templates render without errors
- [ ] `helm lint` passes with no warnings
- [ ] Both ConfigMaps created successfully
- [ ] Smoke test WorkflowTemplate executes and outputs "OK"
- [ ] All 5 agent prompts accessible at `/etc/agents/*.md`
- [ ] MCP requirements accessible at `/work/requirements.yaml`
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
helm lint charts/platform

# Template rendering
helm template charts/platform | yq '. | select(.kind=="ConfigMap")'

# Size check
find charts/platform/files -type f -printf '%s\n' | awk '{s+=$1} END {print s}'

# Encoding check  
file -I charts/platform/files/**/*.md

# Deployment test
helm upgrade --install platform charts/platform -n dev --dry-run

# Smoke test
kubectl -n dev create wf --from=wftmpl/agent-mount-smoke
argo -n dev logs @latest
```

## Notes

- This task sets up the configuration foundation for all subsequent tasks
- Agent prompts will be refined as the system evolves
- MCP tools configuration is project-wide, not per-task
- Environment-specific overrides use standard Helm values patterns
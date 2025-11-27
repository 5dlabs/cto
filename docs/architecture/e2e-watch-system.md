# E2E Watch System Architecture

## Overview

The E2E Watch System provides automated end-to-end testing and remediation for Play workflows. It consists of two agents working in coordination:

1. **Monitor Agent** - Watches Play execution, detects failures, writes structured issue reports
2. **Remediation Agent** - Reads issue reports, makes fixes, triggers Play retries

This system reuses existing infrastructure (Argo Workflows, CodeRun CRDs, shared PVCs, MCP tools) and existing GitHub Apps (Morgan, Rex) with role-specific prompts.

## Goals

- **Automated E2E validation**: Run a Play and verify it completes successfully
- **Self-healing**: When failures occur, automatically remediate and retry
- **Template-driven**: Update expected behavior and remediation strategies without code changes
- **Observability**: Same monitoring/logging as other agents
- **Iteration**: Loop until Play succeeds or max iterations reached

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           E2E Watch Workflow                                 │
│                     (Argo WorkflowTemplate: watch-workflow-template)         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                         Shared PVC                                   │   │
│   │  /workspace/                                                         │   │
│   │  ├── watch/                                                          │   │
│   │  │   ├── status.md              # Current iteration, state           │   │
│   │  │   ├── current-issue.md       # Active issue for remediation       │   │
│   │  │   ├── issue-history.md       # Log of all issues                  │   │
│   │  │   └── acceptance-criteria.md # Expected Play behavior             │   │
│   │  └── play-artifacts/            # Logs, PR info from monitored Play  │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                              │                                              │
│                              │ (mounted by both agents)                     │
│                              │                                              │
│   ┌──────────────────────────┴──────────────────────────┐                  │
│   │                                                      │                  │
│   ▼                                                      ▼                  │
│ ┌───────────────────────┐                    ┌───────────────────────┐     │
│ │    Monitor Agent      │                    │   Remediation Agent   │     │
│ │    (Morgan)           │                    │   (Rex)               │     │
│ ├───────────────────────┤                    ├───────────────────────┤     │
│ │ GitHub App: Morgan    │                    │ GitHub App: Rex       │     │
│ │ CLI: factory/claude   │                    │ CLI: factory/claude   │     │
│ │ Model: configurable   │                    │ Model: configurable   │     │
│ │ Role: monitor         │                    │ Role: remediation     │     │
│ └───────────┬───────────┘                    └───────────┬───────────┘     │
│             │                                            │                  │
│             │ 1. Submits Play                            │                  │
│             │ 2. Monitors via Argo/kubectl               │                  │
│             │ 3. Detects failure                         │                  │
│             │ 4. Writes current-issue.md                 │                  │
│             │                                            │                  │
│             ▼                                            │                  │
│   ┌───────────────────────────────────────┐              │                  │
│   │           Play Workflow               │              │                  │
│   │   (Rex → Cleo → Tess → Atlas)         │              │                  │
│   └───────────────────────────────────────┘              │                  │
│             │                                            │                  │
│             │ On failure (Argo Event)                    │                  │
│             └────────────────────────────────────────────┤                  │
│                                                          │                  │
│                                                          │ 5. Reads issue   │
│                                                          │ 6. Makes fix     │
│                                                          │ 7. Pushes code   │
│                                                          │ 8. Triggers retry│
│                                                          ▼                  │
│                                                   (Loop back to Monitor)    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Configuration Schema

### cto-config.json Structure

```json
{
  "defaults": {
    "watch": {
      "repository": "5dlabs/cto-parallel-test",
      "docsRepository": "5dlabs/cto-parallel-test",
      "docsProjectDirectory": "docs",
      "service": "cto-parallel-test",
      "workingDirectory": ".",
      "maxIterations": 10,
      "playTemplate": "play-workflow-template",
      
      "monitor": {
        "agent": "5DLabs-Morgan",
        "cli": "factory",
        "model": "claude-opus-4-5-20251101",
        "maxTokens": 64000,
        "temperature": 0.7,
        "modelRotation": {
          "enabled": true,
          "models": [
            "claude-sonnet-4-5-20250929",
            "claude-opus-4-5-20251101"
          ]
        },
        "tools": [
          "kubernetes_listResources",
          "kubernetes_getResource",
          "kubernetes_getPodsLogs",
          "kubernetes_getEvents",
          "openmemory_openmemory_query",
          "openmemory_openmemory_store",
          "github_get_pull_request",
          "github_get_pull_request_status",
          "github_list_pull_requests"
        ]
      },
      
      "remediation": {
        "agent": "5DLabs-Rex",
        "cli": "factory",
        "model": "claude-opus-4-5-20251101",
        "maxTokens": 64000,
        "temperature": 0.7,
        "reasoningEffort": "high",
        "modelRotation": {
          "enabled": true,
          "models": [
            "claude-sonnet-4-5-20250929",
            "claude-opus-4-5-20251101"
          ]
        },
        "tools": [
          "brave_search_brave_web_search",
          "context7_resolve_library_id",
          "context7_get_library_docs",
          "github_create_pull_request",
          "github_push_files",
          "github_create_branch",
          "github_get_file_contents",
          "github_create_or_update_file",
          "openmemory_openmemory_query",
          "openmemory_openmemory_store"
        ]
      }
    }
  }
}
```

### Key Configuration Points

| Field | Description |
|-------|-------------|
| `monitor.agent` | GitHub App for monitor role (e.g., `5DLabs-Morgan`) |
| `monitor.cli` | CLI to use: `factory` or `claude` |
| `monitor.model` | Model identifier for the CLI |
| `monitor.tools` | MCP tools available to monitor agent |
| `remediation.agent` | GitHub App for remediation role (e.g., `5DLabs-Rex`) |
| `remediation.cli` | CLI to use: `factory` or `claude` |
| `remediation.model` | Model identifier for the CLI |
| `remediation.tools` | MCP tools available to remediation agent |
| `maxIterations` | Maximum monitor→remediation cycles before giving up |

## Agent Templates

### Directory Structure

Templates follow the existing pattern in `infra/charts/controller/agent-templates/`:

```
infra/charts/controller/agent-templates/
├── watch/                                    # New directory for watch templates
│   ├── factory/
│   │   ├── container-monitor.sh.hbs          # Monitor agent container script
│   │   ├── container-remediation.sh.hbs      # Remediation agent container script
│   │   ├── agents-monitor.md.hbs             # Monitor system prompt (CLAUDE.md)
│   │   └── agents-remediation.md.hbs         # Remediation system prompt (CLAUDE.md)
│   │
│   └── claude/
│       ├── container-monitor.sh.hbs
│       ├── container-remediation.sh.hbs
│       ├── agents-monitor.md.hbs
│       └── agents-remediation.md.hbs
│
└── docs/
    └── templates/
        └── watch/
            └── acceptance-criteria.md        # Default E2E acceptance criteria
```

### Template Pattern

Follow the newer pattern used in `factory` and `cursor` templates:

1. **Container script** (`container-*.sh.hbs`):
   - Sets up environment variables
   - Configures CLI (factory or claude)
   - Mounts MCP tools
   - Runs the agent with appropriate prompt
   - Handles completion/failure states

2. **Agent prompt** (`agents-*.md.hbs`):
   - Role-specific system prompt
   - Written to `CLAUDE.md` in workspace
   - Defines agent behavior and constraints

### Monitor Agent Prompt (agents-monitor.md.hbs)

```markdown
# E2E Watch Monitor Agent

You are the **Monitor Agent** for the E2E Watch system. Your job is to:

1. **Submit and watch a Play workflow** for task {{task_id}}
2. **Detect failures** by monitoring Argo workflow status
3. **Write structured issue reports** when failures occur
4. **Declare success** when the Play completes

## Your Tools

You have access to:
- Kubernetes tools to monitor pods, logs, events
- GitHub tools to check PR status
- OpenMemory to store learnings

## Workflow

### Phase 1: Submit Play

Submit the Play workflow using argo CLI:
```bash
argo submit --from workflowtemplate/{{play_template}} \
  -n argo \
  -p task-id={{task_id}} \
  -p repository={{repository}} \
  -p implementation-agent={{implementation_agent}} \
  -p quality-agent={{quality_agent}} \
  -p testing-agent={{testing_agent}}
```

### Phase 2: Monitor

Poll workflow status every 10 seconds:
```bash
argo get <workflow-name> -n argo -o json
```

Track:
- Current stage (implementation, code-quality, testing, integration)
- Step phases (Running, Succeeded, Failed)
- Pod logs for failed steps

### Phase 3: On Failure

When a failure is detected, write `/workspace/watch/current-issue.md`:

```markdown
# 🔴 E2E Issue Detected

## Metadata
- **Task ID**: {{task_id}}
- **Iteration**: X of {{max_iterations}}
- **Timestamp**: <ISO8601>

## Failure Context
- **Stage**: <stage name>
- **Failed Step**: <step name>
- **Pod**: <pod name>
- **Exit Code**: <code>

## Error Summary
<Concise description of what failed>

## Relevant Logs
```
<Last 100 lines of relevant logs>
```

## Acceptance Criteria Status
- [x] Implementation completed
- [ ] Quality checks passed  ← FAILED HERE
- [ ] Tests passed
- [ ] PR merged

## Suggested Remediation
<Your analysis of how to fix this>
```

Then update `/workspace/watch/status.md` and signal completion (exit 1 for failure).

### Phase 4: On Success

When Play completes successfully:
1. Update `/workspace/watch/status.md` with success
2. Store learnings to OpenMemory
3. Exit 0

## Constraints

- DO NOT attempt to fix issues yourself - that's the Remediation Agent's job
- DO write clear, actionable issue reports
- DO include relevant logs and context
- DO track iteration count to prevent infinite loops
```

### Remediation Agent Prompt (agents-remediation.md.hbs)

```markdown
# E2E Watch Remediation Agent

You are the **Remediation Agent** for the E2E Watch system. Your job is to:

1. **Read the issue report** from `/workspace/watch/current-issue.md`
2. **Analyze the failure** and determine the fix
3. **Make targeted code changes** to resolve the issue
4. **Push changes** to trigger a Play retry

## Your Tools

You have access to:
- Web search and Context7 for researching solutions
- GitHub tools for creating branches, pushing files, creating PRs
- OpenMemory to recall past fixes and store new learnings

## Workflow

### Phase 1: Read Issue

Read and parse `/workspace/watch/current-issue.md` to understand:
- What stage failed
- What the error was
- What logs are relevant
- What the Monitor Agent suggested

### Phase 2: Research (if needed)

If the fix isn't obvious:
- Check OpenMemory for similar past issues
- Search Context7 for library documentation
- Web search for error messages

### Phase 3: Fix

Make the minimal targeted fix:
- Clone the repository (or use existing workspace)
- Create a fix branch: `fix/watch-iteration-<N>-<description>`
- Make code changes
- Run local validation (cargo fmt, cargo clippy, cargo test)
- Commit with clear message

### Phase 4: Push

Push changes to trigger Play retry:
- Push to the fix branch
- Create PR if one doesn't exist
- The Play workflow will automatically retry when main is updated

### Phase 5: Update Status

Update `/workspace/watch/status.md`:
- Mark remediation complete
- Note what was fixed
- Increment iteration counter

Exit 0 to signal completion.

## Constraints

- DO make minimal, targeted fixes - don't refactor unrelated code
- DO run quality checks before pushing (fmt, clippy, tests)
- DO NOT over-engineer - fix only what's broken
- DO store successful fix patterns to OpenMemory for future reference
- DO check OpenMemory first for similar past issues
```

## Shared PVC Structure

Both agents mount the same PVC at `/workspace/`:

```
/workspace/
├── watch/
│   ├── status.md                 # Current state
│   ├── current-issue.md          # Active issue (written by Monitor)
│   ├── issue-history.md          # Append-only log of all issues
│   └── acceptance-criteria.md    # Expected Play behavior (from template)
│
├── play-artifacts/               # Optional: captured from Play
│   ├── logs/
│   │   └── <stage>-<pod>.log
│   └── pr-info.md
│
└── repo/                         # Cloned target repository
    └── <repository contents>
```

### status.md Format

```markdown
# E2E Watch Status

## Current State
- **Phase**: monitoring | remediating | succeeded | failed
- **Iteration**: 3 of 10
- **Started**: 2024-01-15T10:00:00Z
- **Last Update**: 2024-01-15T10:35:00Z

## Play Workflow
- **Name**: play-task-42-abc123
- **Status**: Running | Succeeded | Failed
- **Current Stage**: code-quality

## History
| Iteration | Stage Failed | Issue | Fix Applied | Duration |
|-----------|--------------|-------|-------------|----------|
| 1 | implementation | compile error | added import | 5m |
| 2 | code-quality | clippy warning | applied suggestion | 3m |
| 3 | (in progress) | - | - | - |
```

### current-issue.md Format

See Monitor Agent prompt above for full format.

### acceptance-criteria.md (Default Template)

```markdown
# E2E Play Acceptance Criteria

## Stage Completion
- [ ] Implementation stage completed (PR created)
- [ ] Quality stage completed (Cleo approved)
- [ ] Testing stage completed (Tess approved)
- [ ] Integration stage completed (PR merged to main)

## Quality Gates
- [ ] All agents completed within max retries
- [ ] No stage exceeded 30 minute timeout
- [ ] No crash loops or OOM kills

## Artifacts
- [ ] PR exists with meaningful description
- [ ] All CI checks passed
- [ ] Code changes committed to main

## Behavioral
- [ ] Each agent stored memories to OpenMemory
- [ ] Stage transitions followed expected order
- [ ] No orphaned resources left in cluster
```

## Argo Workflow Template

### watch-workflow-template.yaml

```yaml
apiVersion: argoproj.io/v1alpha1
kind: WorkflowTemplate
metadata:
  name: watch-workflow-template
  namespace: argo
spec:
  entrypoint: watch-loop
  
  arguments:
    parameters:
      - name: task-id
      - name: repository
      - name: max-iterations
        value: "10"
      - name: monitor-agent
        value: "5DLabs-Morgan"
      - name: remediation-agent
        value: "5DLabs-Rex"
  
  volumeClaimTemplates:
    - metadata:
        name: workspace
      spec:
        accessModes: ["ReadWriteOnce"]
        resources:
          requests:
            storage: 10Gi
  
  templates:
    - name: watch-loop
      steps:
        # Initialize workspace
        - - name: init-workspace
            template: init-workspace
        
        # Main loop: monitor → remediate → repeat
        - - name: monitor-and-remediate
            template: monitor-remediate-loop
            arguments:
              parameters:
                - name: iteration
                  value: "1"
    
    - name: init-workspace
      container:
        image: alpine:latest
        command: [sh, -c]
        args:
          - |
            mkdir -p /workspace/watch /workspace/play-artifacts /workspace/repo
            echo "# E2E Watch Status" > /workspace/watch/status.md
            echo "## Current State" >> /workspace/watch/status.md
            echo "- **Phase**: initializing" >> /workspace/watch/status.md
            echo "- **Iteration**: 0 of {{workflow.parameters.max-iterations}}" >> /workspace/watch/status.md
        volumeMounts:
          - name: workspace
            mountPath: /workspace
    
    - name: monitor-remediate-loop
      inputs:
        parameters:
          - name: iteration
      steps:
        # Run monitor agent
        - - name: monitor
            template: run-monitor
        
        # Check if succeeded or needs remediation
        - - name: check-result
            template: check-monitor-result
        
        # If failed and under max iterations, run remediation
        - - name: remediate
            template: run-remediation
            when: "{{steps.check-result.outputs.result}} == 'needs-remediation'"
        
        # Loop back if remediation was run
        - - name: next-iteration
            template: monitor-remediate-loop
            when: "{{steps.check-result.outputs.result}} == 'needs-remediation'"
            arguments:
              parameters:
                - name: iteration
                  value: "{{steps.check-result.outputs.parameters.next-iteration}}"
    
    - name: run-monitor
      # Creates CodeRun for Monitor Agent (Morgan)
      # Uses container-monitor.sh.hbs template
      resource:
        action: create
        manifest: |
          apiVersion: agents.platform/v1
          kind: CodeRun
          metadata:
            generateName: watch-monitor-{{workflow.parameters.task-id}}-
            namespace: cto
          spec:
            taskId: {{workflow.parameters.task-id}}
            service: "watch-monitor"
            repositoryUrl: "https://github.com/{{workflow.parameters.repository}}"
            docsRepositoryUrl: "https://github.com/{{workflow.parameters.repository}}"
            model: "claude-opus-4-5-20251101"
            githubApp: "{{workflow.parameters.monitor-agent}}"
            # Additional spec fields from config...
    
    - name: run-remediation
      # Creates CodeRun for Remediation Agent (Rex)
      # Uses container-remediation.sh.hbs template
      resource:
        action: create
        manifest: |
          apiVersion: agents.platform/v1
          kind: CodeRun
          metadata:
            generateName: watch-remediation-{{workflow.parameters.task-id}}-
            namespace: cto
          spec:
            taskId: {{workflow.parameters.task-id}}
            service: "watch-remediation"
            repositoryUrl: "https://github.com/{{workflow.parameters.repository}}"
            docsRepositoryUrl: "https://github.com/{{workflow.parameters.repository}}"
            model: "claude-opus-4-5-20251101"
            githubApp: "{{workflow.parameters.remediation-agent}}"
            # Additional spec fields from config...
    
    - name: check-monitor-result
      # Reads status.md and determines next action
      script:
        image: alpine:latest
        command: [sh]
        source: |
          STATUS=$(cat /workspace/watch/status.md | grep "Phase:" | awk '{print $NF}')
          ITERATION=$(cat /workspace/watch/status.md | grep "Iteration:" | awk -F'of' '{print $1}' | awk '{print $NF}')
          MAX={{workflow.parameters.max-iterations}}
          
          if [ "$STATUS" = "succeeded" ]; then
            echo "succeeded"
          elif [ "$ITERATION" -ge "$MAX" ]; then
            echo "max-iterations-reached"
          else
            echo "needs-remediation"
            echo $((ITERATION + 1)) > /tmp/next-iteration
          fi
        volumeMounts:
          - name: workspace
            mountPath: /workspace
      outputs:
        parameters:
          - name: next-iteration
            valueFrom:
              path: /tmp/next-iteration
```

## CLI Interface

### play-monitor CLI Extension

Add `watch` command to `monitor/src/main.rs`:

```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...
    
    /// Start E2E watch: monitor Play, remediate failures, loop until success
    Watch {
        /// Task ID for the Play
        #[arg(long)]
        task_id: String,
        
        /// Path to cto-config.json
        #[arg(long, default_value = "cto-config.json")]
        config: String,
        
        /// Override max iterations (default from config)
        #[arg(long)]
        max_iterations: Option<u32>,
        
        /// Dry run - show what would be submitted without creating resources
        #[arg(long)]
        dry_run: bool,
    },
}
```

### Command Flow

```bash
play-monitor watch --task-id 42 --config cto-config.json
```

1. Parse `cto-config.json`, extract `defaults.watch` section
2. Validate configuration (agents exist, tools valid, etc.)
3. Submit `watch-workflow-template` to Argo with parameters:
   - task-id
   - repository
   - max-iterations
   - monitor-agent, monitor-cli, monitor-model, monitor-tools
   - remediation-agent, remediation-cli, remediation-model, remediation-tools
4. Stream workflow status (similar to existing `loop` command)
5. Output JSON events for each phase change

## Implementation Phases

### Phase 1: Configuration & Templates

1. Add `watch` section to cto-config.json schema
2. Create template directory structure under `agent-templates/watch/`
3. Implement `container-monitor.sh.hbs` for factory CLI
4. Implement `container-remediation.sh.hbs` for factory CLI
5. Create `agents-monitor.md.hbs` system prompt
6. Create `agents-remediation.md.hbs` system prompt
7. Create default `acceptance-criteria.md` template

### Phase 2: Workflow Template

1. Create `watch-workflow-template.yaml`
2. Implement shared PVC setup
3. Implement monitor→check→remediate loop logic
4. Wire up CodeRun creation for both agents
5. Handle iteration counting and max-iterations termination

### Phase 3: CLI Integration

1. Add `watch` subcommand to `play-monitor`
2. Parse watch config section
3. Implement workflow submission with watch parameters
4. Add status streaming for watch workflows

### Phase 4: Claude CLI Support

1. Duplicate factory templates for claude CLI
2. Adjust container scripts for claude-specific invocation
3. Test both CLI paths

### Phase 5: Testing & Iteration

1. Manual E2E test with simple failure scenarios
2. Verify monitor→remediation handoff via shared PVC
3. Verify iteration loop terminates correctly
4. Add OpenMemory integration for learning persistence

## Success Criteria

- [ ] `play-monitor watch --task-id X` successfully submits watch workflow
- [ ] Monitor agent detects Play failures and writes structured issue reports
- [ ] Remediation agent reads issues and makes targeted fixes
- [ ] Loop continues until Play succeeds or max iterations reached
- [ ] Both factory and claude CLIs work
- [ ] Status visible in Argo UI and via CLI
- [ ] Learnings stored to OpenMemory for future improvement

## Open Questions

1. **Play retry mechanism**: Should remediation push to main (triggering natural retry) or explicitly re-submit the Play workflow?
2. **Partial progress**: If Play fails at stage 3, should retry start from stage 1 or attempt to resume?
3. **Resource cleanup**: How to handle orphaned resources from failed Play attempts?
4. **Timeouts**: Should individual stages have timeouts, or just overall watch timeout?

## References

- Existing Play workflow: `infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml`
- Factory container pattern: `infra/charts/controller/agent-templates/code/factory/container-base.sh.hbs`
- Rex remediation template: `infra/charts/controller/agent-templates/code/claude/container-rex-remediation.sh.hbs`
- Acceptance criteria pattern: `infra/charts/controller/agent-templates/docs/templates/*/acceptance-criteria.md`
- Current monitor CLI: `monitor/src/main.rs`

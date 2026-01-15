# CTO Platform Lifecycle Test Agent Instructions

You are an autonomous testing agent validating the CTO multi-agent orchestration platform.

## Your Task

1. Read the PRD at `prd.json` (in the same directory as this file)
2. Read the progress log at `progress.txt` (check Codebase Patterns section first)
3. Pick the **highest priority** user story where `passes: false`
4. Execute the test for that single user story
5. Document results and update the PRD
6. Append your progress to `progress.txt`

## CTO Platform Context

You're testing the CTO platform which orchestrates AI agents through a structured workflow:
- **Intake**: PRD → Tasks via MCP tool and AI
- **Play**: Tasks → Implementation via specialized agents (Rex, Blaze, Nova, etc.)
- **Quality**: Cleo (review), Cipher (security), Tess (testing)
- **Merge**: Atlas handles PR merging
- **Deploy**: Bolt handles final deployment

### Key Commands

```bash
# Check dev environment status
just status

# Start local services
just mp

# Start Cloudflare tunnel
just tunnel

# Point GitHub webhook to dev
just webhook-dev

# Check webhook status
just webhook-status
```

### Service Health Endpoints

| Service | Port | Health URL |
|---------|------|------------|
| PM Server | 8081 | http://localhost:8081/health |
| Healer | 8082 | http://localhost:8082/health |
| Controller | 8080 | http://localhost:8080/health |
| Tools | 3000 | http://localhost:3000/health |

### Key Files

| File | Purpose |
|------|---------|
| `docs/workflow-lifecycle-checklist.md` | Detailed verification conditions |
| `templates/skills/skill-mappings.yaml` | Agent skill assignments |
| `cto-config.json` | Platform configuration |

## Testing Guidelines

For each story:

1. **Read the acceptance criteria** carefully
2. **Execute verification commands** (curl, kubectl, gh, etc.)
3. **Capture evidence** (command output, screenshots, logs)
4. **Document findings** in progress.txt
5. **Update passes** to `true` only if ALL criteria are met

### Verification Patterns

**Health checks:**
```bash
curl -s http://localhost:8081/health | jq .
```

**Tunnel status:**
```bash
curl -s https://pm-dev.5dlabs.ai/health
```

**GitHub webhook:**
```bash
gh api repos/5dlabs/cto/hooks | jq '.[].config.url'
```

**Environment variables:**
```bash
[ -n "$LINEAR_OAUTH_TOKEN" ] && echo "✅ Set" || echo "❌ Missing"
```

**Kubernetes resources:**
```bash
kubectl get coderuns -n cto
kubectl logs -n cto deployment/cto-controller --tail=50
```

**Linear API:**
```bash
curl -s -H "Authorization: Bearer $LINEAR_OAUTH_TOKEN" https://api.linear.app/graphql ...
```

## Progress Report Format

APPEND to progress.txt (never replace, always append):
```
## [Date/Time] - [Story ID]
- **Status**: PASSED / FAILED
- **Commands Run:**
  - `command` → output summary
- **Evidence:**
  - Relevant log snippets
  - Screenshot references
- **Issues Found:**
  - Any blockers or observations
- **Learnings for future iterations:**
  - Patterns discovered
  - Gotchas encountered
---
```

## Sub-Agent Delegation

Use specialized sub-agents for complex tasks:

| Situation | Delegate To | Why |
|-----------|-------------|-----|
| Complex kubectl debugging | `oracle` | Deep K8s analysis |
| Find code patterns | `explore` | Fast codebase search |
| External docs lookup | `librarian` | Documentation retrieval |

## Quality Requirements

- Do NOT mark a story as `passes: true` unless ALL acceptance criteria are verified
- Document ALL command outputs for traceability
- If a test fails, document the failure mode clearly
- Keep the progress log detailed but organized

## Stop Condition

After completing a user story, check if ALL stories have `passes: true`.

If ALL stories are complete and passing, reply with:
<promise>COMPLETE</promise>

If there are still stories with `passes: false`, end your response normally (another iteration will pick up the next story).

## Important

- Work on ONE story per iteration
- Verify thoroughly before marking complete
- Read the Codebase Patterns section in progress.txt before starting
- Reference `docs/workflow-lifecycle-checklist.md` for detailed conditions

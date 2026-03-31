Implement subtask 8003: Create infrastructure sequencing diagram and ArgoCD promotion workflow documentation

## Objective
Write infrastructure-sequence.md with a Mermaid dependency diagram, critical path analysis, parallel execution windows, and timeline estimates. Document the ArgoCD promotion workflow including Application CR references, sync wave annotations, and promotion commands.

## Steps
1. Create `docs/hermes/infrastructure-sequence.md`:
   - **Mermaid Dependency Diagram**: Use `graph TD` or `flowchart TD` syntax showing all tasks (1-8+) with directed edges for dependencies. Color-code: green for completed, blue for in-progress, gray for pending.
   - **Critical Path**: Identify and bold the longest dependency chain (Task 1 → 2 → 3 → 7 → promotion)
   - **Parallel Execution Windows**: Table showing which tasks can run concurrently (e.g., Tasks 4, 5, 6 after Task 3)
   - **Estimated Timeline**: Gantt-style table with task durations and start/end dates relative to kickoff

2. Add **ArgoCD Promotion Workflow** section (can be in the same doc or a separate `docs/hermes/argocd-promotion.md`):
   - Reference the ArgoCD Application CR name and namespace from Task 1
   - Promotion flow diagram (Mermaid sequence diagram):
     a. Developer pushes to staging branch
     b. ArgoCD auto-syncs staging Application
     c. GitHub Actions triggers E2E tests (Task 7 workflow)
     d. On E2E pass → manual approval in ArgoCD UI or CLI
     e. `argocd app sync hermes-backend-production --prune` command with prerequisites
   - Sync wave annotations: document the order in which Hermes resources should sync (ConfigMap → Deployment → Service → Ingress)
   - Rollback via ArgoCD: `argocd app rollback hermes-backend-production` command

## Validation
Verify Mermaid diagram renders without syntax errors by pasting into GitHub's Mermaid live editor or a `.md` file preview. Verify the `argocd app sync hermes-backend-production` command references the correct Application CR name from Task 1's output. Verify sync wave annotation examples are valid Kubernetes annotation format.
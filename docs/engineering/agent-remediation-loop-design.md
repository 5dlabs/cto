# Agent Remediation Loop Design

## Overview: What We Want to Achieve

Create an automated feedback loop where Tess's QA review comments trigger Rex to fix issues, without human intervention, until the code meets all quality standards.

### Current State
- Rex → Creates PR → Cleo → Adds ready-for-qa → Tess → Reviews (currently stops here)
- If Tess requests changes, the workflow ends and requires manual intervention

### Desired State
- Rex → Cleo → Tess → (if issues) → Rex fixes → Cleo → Tess → (repeat until approved) → Merge

## Core Requirements

1. **No Human Intervention** - Fully automated loop (but humans can stop it)
2. **Clear Termination** - Must eventually converge (no infinite loops)
3. **Preserve Context** - Rex needs to know what to fix while understanding original task
4. **Efficient** - Don't re-run unnecessary steps
5. **Trackable** - Clear visibility into what stage we're at

## Potential Implementation Options

### Option 1: Complex Workflow with Suspend/Resume Loops
**Approach:**
- Add conditional DAG paths based on review status
- New suspend nodes for remediation
- Sensors detect review status and resume appropriate path

**Pros:**
- Fits existing architecture
- Full Argo Workflows control
- Can track everything in one workflow

**Cons:**
- Complex workflow logic
- Many suspend nodes to manage
- Difficult to debug
- Risk of orphaned suspended workflows

### Option 2: Separate Remediation Workflow
**Approach:**
- Main workflow completes after Tess review
- New sensor triggers separate remediation workflow if changes requested
- Remediation workflow runs Rex → Cleo → Tess cycle

**Pros:**
- Cleaner separation of concerns
- Easier to debug (separate workflow runs)
- Can limit iterations per workflow

**Cons:**
- Multiple workflows to correlate
- Need to pass context between workflows
- More sensors and triggers needed

### Option 3: Tess as Orchestrator
**Approach:**
- Tess container stays running
- Internally loops: test → review → trigger Rex → wait → test again
- Single container manages the whole cycle

**Pros:**
- Simple architecture
- No complex workflow logic
- Easy to implement retry limits
- Single source of truth

**Cons:**
- Long-running container (resource usage)
- Harder to monitor progress
- Violates single-responsibility principle

### Option 4: GitHub Labels as State Machine
**Approach:**
- Use PR labels to track state
- Each agent changes labels when done
- Sensors trigger based on label changes
- Labels: `needs-fixes`, `fixing-in-progress`, `needs-cleo`, `needs-tess`, `approved`

**Pros:**
- Dead simple
- GitHub is source of truth
- Human-readable and debuggable
- Easy manual intervention
- Natural audit trail

**Cons:**
- Depends on GitHub labels (could be accidentally changed)
- Need sensors for each label transition
- Less control over execution order

### Option 5: ConfigMap State Machine
**Approach:**
- Store workflow state in ConfigMap
- Each agent updates state
- Single sensor watches ConfigMap and triggers appropriate agent

**Pros:**
- Kubernetes-native
- Full control over state
- Can store complex data (review comments, iteration count)

**Cons:**
- Need to manage ConfigMap lifecycle
- Another thing to debug
- State could get out of sync

## Identified Pitfalls

### 1. Infinite Loop Scenarios
- **Oscillating Fixes**: Rex fixes A, breaks B → fixes B, breaks A
- **Unconvergeable Requirements**: Tess wants X, but X breaks Cleo's requirement Y
- **Mitigation**: Hard iteration limit (e.g., max 5 rounds)

### 2. Review Comment Management
- **Stale Comments**: Old comments that were already fixed
- **Duplicate Comments**: Same issue reported multiple times
- **Comment Correlation**: Which comments apply to which code version?
- **Mitigation**: Only use latest review, clear comment IDs/markers

### 3. Race Conditions
- **Concurrent Modifications**: Multiple agents trying to push
- **Mid-flight Changes**: New commits while agent is running
- **Mitigation**: Locks, linear execution enforcement

### 4. State Synchronization
- **Lost Updates**: State changes not properly recorded
- **Orphaned Workflows**: Suspended workflows that never resume
- **Duplicate Triggers**: Same event triggering multiple times
- **Mitigation**: Idempotency keys, cleanup jobs

### 5. Context Preservation
- **Lost Task Context**: Rex forgets original requirements during remediation
- **Incomplete Fixes**: Fixing symptoms not root causes
- **Mitigation**: Always include original task + review comments

## Key Concerns

1. **Implementation Complexity**: How long will this take to build and debug?
2. **Operational Complexity**: How hard is it to troubleshoot when it breaks?
3. **Resource Usage**: Long-running workflows/containers consuming cluster resources
4. **Escape Hatches**: How does a human intervene if needed?
5. **Progress Visibility**: How do we know what's happening in the loop?
6. **Quality Convergence**: Will the loop actually improve quality or just ping-pong?
7. **Time to Resolution**: How long before we give up and alert a human?

## Open Questions

1. **How do we handle partial fixes?** If Rex fixes 3/5 issues, do we run Cleo on partial fixes?

2. **Should Cleo re-run on every iteration?** Or can we skip if only test-related fixes?

3. **How do we detect "unfixable" issues?** When do we give up and alert humans?

4. **What's the source of truth for "issues"?** 
   - GitHub review comments?
   - Tess's internal assessment?
   - Combination of both?

5. **How do we prevent fix regression?** Ensuring new fixes don't break previous fixes?

6. **Should we batch fixes?** Fix all issues at once or incrementally?

7. **How do we handle conflicting requirements?** Tess vs Cleo disagreement?

8. **What's the retry strategy?**
   - Linear: Try X times then fail
   - Exponential backoff: Increasing delays
   - Smart: Based on type of failure

## Alternative Approaches

### Alternative 1: Human-in-the-Loop Approval
- Auto-fix simple issues (formatting, linting)
- Require human approval for complex fixes
- Hybrid automation

### Alternative 2: Pre-emptive Checking
- Run Tess checks BEFORE PR creation
- Fix issues before they become review comments
- Prevent the need for remediation

### Alternative 3: Smart Batching
- Collect all feedback (Cleo + Tess) first
- Single remediation pass with all issues
- Reduces iterations

### Alternative 4: Tiered Remediation
- Level 1: Auto-fix (formatting, simple bugs)
- Level 2: Rex remediation (logic fixes)
- Level 3: Human intervention (design issues)

### Alternative 5: Shadow Mode
- Run remediation in parallel branch
- Compare results
- Only merge if demonstrably better

## Recommendation

**Start with Option 4 (GitHub Labels as State Machine)** because:

1. **Simplest to implement** - Can build in a day
2. **Easy to debug** - Just look at PR labels
3. **Natural escape hatches** - Humans can change labels
4. **Progressive enhancement** - Can add complexity later
5. **GitHub-native** - Leverages existing platform

**Implementation Plan:**
1. Add label management to each agent
2. Create sensors for label transitions
3. Add iteration counter to prevent infinite loops
4. Build Rex remediation container (minimal changes)
5. Test with simple case first

**Success Criteria:**
- Completes in < 5 iterations
- No human intervention needed
- Clear audit trail
- Can be stopped manually
- Converges to approved state

## Next Steps

1. **Prototype** the label-based approach with one simple test case
2. **Measure** iteration counts and time to convergence  
3. **Iterate** based on learnings
4. **Consider** more complex approach only if needed

## Risk Mitigation

1. **Start Small**: Test with trivial fixes first (formatting)
2. **Add Telemetry**: Log everything for analysis
3. **Set Limits**: Max 5 iterations, 30 min timeout
4. **Monitor Closely**: Alert on stuck loops
5. **Easy Rollback**: Feature flag to disable loop
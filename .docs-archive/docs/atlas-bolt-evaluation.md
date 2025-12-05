# Atlas & Bolt Agent Implementation Evaluation

**Date:** November 3, 2025  
**Status:** Partially Implemented - Infrastructure Ready, Integration Pending  
**Priority:** Medium - Foundation complete, workflow integration needed

---

## Executive Summary

Atlas (Integration & Merge Specialist) and Bolt (DevOps & Deployment Specialist) are two new agents being added to the 5D Labs multi-agent platform. The foundational infrastructure is **85% complete**, with agent definitions, system prompts, avatars, and setup scripts all in place. However, **critical integration work remains** including GitHub App creation, credential storage, workflow integration, and event-driven activation.

---

## Current Implementation Status

### ✅ **Completed Components**

#### 1. **Agent Definitions** (100% Complete)
- **Location:** `infra/charts/controller/values.yaml` (lines 523-616)
- **Status:** Full agent configurations with system prompts, roles, expertise areas
- **Details:**
  - Atlas: Integration & Merge Specialist
    - CLI: Claude
    - Model: claude-sonnet-4-20250514
    - Temperature: 0.4 (precise, methodical)
    - Role: Merge conflict resolution, PR queue management
  - Bolt: DevOps & Deployment Specialist
    - CLI: Claude
    - Model: claude-sonnet-4-20250514
    - Temperature: 0.3 (even more conservative for ops)
    - Role: Kubernetes deployment monitoring, ArgoCD health checks

#### 2. **Avatar Assets** (100% Complete)
- **Location:** `images/processed/`
- **Files:**
  - `atlas-avatar.png` (plus 64, 180, 512, 1024 variants)
  - `bolt-avatar.png` (plus 64, 180, 512, 1024 variants)
- **Status:** Professional avatars ready for GitHub App creation

#### 3. **Documentation** (100% Complete)
- **README.md** updated with Atlas and Bolt sections
- **Setup Scripts:**
  - `scripts/setup-atlas-bolt-quickstart.sh` - Full guided setup
  - `scripts/create-github-apps-local.sh` - GitHub App creation wizard
- **Generated Snippets:** Scripts create configuration snippets automatically

#### 4. **Infrastructure Templates** (100% Complete)
- **ConfigMap Generation:** `agents-configmap.yaml` automatically includes Atlas/Bolt
- **Task Controller Config:** `task-controller-config.yaml` auto-registers agents
- **Template System:** Agent templates will be auto-generated from values.yaml

---

### ⚠️ **Partially Complete Components**

#### 5. **Helm Values Configuration** (90% Complete)
- **Location:** `infra/charts/controller/values.yaml`
- **Status:** Definitions complete, but missing GitHub App IDs
- **TODO Fields:**
  ```yaml
  appId: ""  # TODO: Fill after creating GitHub App
  clientId: ""  # TODO: Fill after creating GitHub App
  ```
- **Impact:** Cannot create workflows until these are populated

---

### ❌ **Missing Components**

#### 6. **GitHub Apps** (0% Complete)
- **Status:** NOT CREATED
- **Required:** Two GitHub Apps need to be created
  - `5DLabs-Atlas` - Integration & Merge Specialist
  - `5DLabs-Bolt` - DevOps & Deployment Specialist
- **Permissions Defined:** Setup script has complete permission specifications
- **Impact:** Agents cannot authenticate to GitHub without these

#### 7. **Credential Storage** (0% Complete)
- **Vault Storage:** NOT CONFIGURED
  - No `github-app-atlas` secret in Vault
  - No `github-app-bolt` secret in Vault
- **ExternalSecrets:** MISSING from `infra/secret-store/agent-secrets-external-secrets.yaml`
  - Need to add ExternalSecret definitions for both agents
  - Script generates templates but they haven't been integrated
- **Impact:** Even if GitHub Apps exist, secrets won't sync to Kubernetes

#### 8. **MCP Configuration** (0% Complete)
- **Location:** `cto-config.json`
- **Status:** Atlas and Bolt are NOT defined in agents section
- **Current Agents:** morgan, rex, cleo, tess, blaze, cipher (6 agents)
- **Missing:** atlas, bolt configurations
- **Impact:** MCP server won't recognize these agents for workflow assignment

#### 9. **Workflow Integration** (0% Complete)
- **Current Workflow Roles:**
  - Implementation: Rex, Blaze
  - Quality: Cleo
  - Security: Cipher
  - Testing: Tess
- **Missing Roles:**
  - Integration/Merge: Atlas *(no workflow parameter)*
  - Deployment/DevOps: Bolt *(no workflow parameter)*
- **Issue:** Play workflows don't have parameters for `integration-agent` or `deployment-agent`
- **Impact:** No way to activate Atlas or Bolt in current workflow system

#### 10. **Event-Driven Activation** (0% Complete - Design Needed)
- **Atlas Triggers:** NOT DEFINED
  - When should Atlas activate? (Merge conflicts? PR approval ready?)
  - What events trigger Atlas workflows?
  - How does Atlas fit into Rex → Cleo → Tess flow?
- **Bolt Triggers:** NOT DEFINED
  - When should Bolt activate? (Post-merge? ArgoCD sync?)
  - What monitoring does Bolt perform?
  - How does Bolt coordinate with Tess for deployment validation?
- **Impact:** Unclear how/when these agents would actually run

---

## Architecture Analysis

### **Current Agent Flow** (Rex → Cleo → Tess)
```
1. Rex: Implements code, creates PR
2. Cleo: Reviews quality, runs linting/tests
3. Tess: E2E testing in Kubernetes
4. [Human]: Final approval and merge
```

### **Proposed Integration Points** (Needs Design)

#### **Option A: Atlas as Post-Approval Integration Agent**
```
1. Rex → Cleo → Tess → Atlas → Bolt
   ↓
2. After Tess approves, Atlas:
   - Checks for merge conflicts with main
   - Resolves conflicts automatically
   - Rebases if needed
   - Triggers actual merge
3. After merge, Bolt:
   - Monitors ArgoCD sync
   - Validates Kubernetes deployment
   - Checks service health
   - Reports deployment status
```

#### **Option B: Atlas as Parallel Conflict Resolver**
```
Rex creates PR → Merge conflict detected
    ↓
Atlas activated in parallel:
    - Resolves conflicts
    - Updates PR
    - Re-triggers Cleo/Tess validation
```

#### **Option C: Bolt as Continuous Deployment Monitor**
```
Any PR merged to main → ArgoCD sync triggered
    ↓
Bolt activated automatically:
    - Monitors deployment progress
    - Validates resource health
    - Runs post-deployment checks
    - Notifies team of issues
```

**❗ DECISION NEEDED:** Which architecture makes sense for your workflow?

---

## Detailed Gap Analysis

### **1. GitHub App Creation**

**Current State:** Scripts exist but apps not created  
**Required Actions:**
1. Run `./scripts/create-github-apps-local.sh`
2. Manually create 2 GitHub Apps via browser
3. Download private keys
4. Store credentials locally

**Estimated Time:** 30 minutes

---

### **2. Credential Infrastructure**

**Current State:** No secrets configured  
**Required Actions:**
1. Store credentials in Vault:
   ```bash
   vault kv put secret/github-app-atlas \
     app_id="..." \
     client_id="..." \
     private_key=@.atlas-private-key.pem
   
   vault kv put secret/github-app-bolt \
     app_id="..." \
     client_id="..." \
     private_key=@.bolt-private-key.pem
   ```

2. Add ExternalSecrets to `infra/secret-store/agent-secrets-external-secrets.yaml`:
   - `github-app-5dlabs-atlas` (secret-store namespace)
   - `github-app-5dlabs-atlas-cto` (cto namespace)
   - `github-app-atlas` (alias for short name)
   - Same pattern for Bolt

3. Update values.yaml with App IDs:
   ```yaml
   atlas:
     appId: "123456"
     clientId: "Iv1.abc123def456"
   bolt:
     appId: "789012"
     clientId: "Iv1.xyz789uvw012"
   ```

**Estimated Time:** 45 minutes (scripted templates exist)

---

### **3. MCP Configuration**

**Current State:** cto-config.json has no atlas/bolt entries  
**Required Actions:**
1. Add agent configurations to `cto-config.json`:
   ```json
   "agents": {
     "atlas": {
       "githubApp": "5DLabs-Atlas",
       "cli": "claude",
       "model": "claude-sonnet-4-20250514",
       "tools": {
         "remote": [
           "brave_search_brave_web_search",
           "context7_get_library_docs"
         ],
         "localServers": {}
       }
     },
     "bolt": {
       "githubApp": "5DLabs-Bolt",
       "cli": "claude",
       "model": "claude-sonnet-4-20250514",
       "tools": {
         "remote": [
           "brave_search_brave_web_search",
           "context7_get_library_docs"
         ],
         "localServers": {
           "kubernetes": {
             "enabled": true,
             "command": "kubectl",
             "args": [],
             "tools": ["get_pods", "describe_resource", "get_logs"]
           }
         }
       }
     }
   }
   ```

**Note:** Bolt might benefit from Kubernetes MCP tools for cluster monitoring

**Estimated Time:** 15 minutes

---

### **4. Workflow Integration Design**

**Current State:** No integration points defined  
**Required Actions:**

#### **Phase 1: Define Workflow Parameters**
Add to `play-workflow-template.yaml`:
```yaml
arguments:
  parameters:
    - name: integration-agent
      description: "Agent for merge conflict resolution"
      value: "5DLabs-Atlas"
    - name: integration-cli
      value: "claude"
    - name: integration-model
      value: "claude-sonnet-4-20250514"
    
    - name: deployment-agent
      description: "Agent for deployment monitoring"
      value: "5DLabs-Bolt"
    - name: deployment-cli
      value: "claude"
    - name: deployment-model
      value: "claude-sonnet-4-20250514"
```

#### **Phase 2: Create Workflow Steps**
Add new workflow templates:
- `integration-check-template.yaml` (Atlas)
- `deployment-monitor-template.yaml` (Bolt)

#### **Phase 3: Event Sensors**
Create Argo Events sensors:
- `atlas-merge-conflict-sensor.yaml`
- `atlas-ready-to-merge-sensor.yaml`
- `bolt-deployment-monitor-sensor.yaml`

**Estimated Time:** 4-6 hours (design + implementation)

---

### **5. Testing & Validation**

**Current State:** No test scenarios defined  
**Required Actions:**

#### **Atlas Testing:**
1. Create test PR with intentional merge conflict
2. Trigger Atlas workflow
3. Verify conflict resolution
4. Validate PR update and re-validation

#### **Bolt Testing:**
1. Merge PR to main
2. Monitor ArgoCD sync
3. Verify Bolt detects deployment
4. Check health validation logic

**Estimated Time:** 2-3 hours per agent

---

## Recommended Implementation Plan

### **Phase 1: Foundation (Immediate - 2 hours)**
- [ ] Create GitHub Apps via setup script
- [ ] Store credentials in Vault
- [ ] Add ExternalSecrets to agent-secrets-external-secrets.yaml
- [ ] Update values.yaml with App IDs
- [ ] Add agents to cto-config.json

**Deliverable:** Atlas and Bolt can authenticate to GitHub

---

### **Phase 2: Workflow Design (1 day)**
- [ ] Define use cases for Atlas activation
- [ ] Define use cases for Bolt activation
- [ ] Design workflow integration architecture
- [ ] Create event trigger specifications
- [ ] Document handoff points with existing agents

**Deliverable:** Clear architectural design document

---

### **Phase 3: Workflow Implementation (2-3 days)**
- [ ] Add workflow parameters for integration/deployment agents
- [ ] Create Atlas workflow template
- [ ] Create Bolt workflow template
- [ ] Implement event sensors
- [ ] Add workflow steps to play-workflow-template

**Deliverable:** Working workflow infrastructure

---

### **Phase 4: Testing & Validation (1 day)**
- [ ] Atlas merge conflict resolution test
- [ ] Bolt deployment monitoring test
- [ ] Integration testing with full Rex → Cleo → Tess → Atlas → Bolt flow
- [ ] Performance and reliability testing

**Deliverable:** Production-ready agents

---

## Risk Assessment

### **High Priority Risks**

1. **Workflow Architecture Unclear**
   - Risk: Wasting effort on wrong integration approach
   - Mitigation: Design phase before implementation
   - Owner: Product/Architecture team

2. **Event Trigger Complexity**
   - Risk: Difficult to detect right conditions for agent activation
   - Mitigation: Start with manual triggers, add automation incrementally
   - Owner: Platform engineering

### **Medium Priority Risks**

3. **Agent Overlap with Existing Roles**
   - Risk: Atlas/Bolt duplicate work already done by Cleo/Tess
   - Mitigation: Clear role boundaries and use cases
   - Owner: Agent team leads

4. **Credential Security**
   - Risk: Private keys exposed during setup
   - Mitigation: Use .gitignore, Vault, proper secret handling
   - Owner: Security team

---

## Questions for Stakeholders

### **Product Questions:**
1. What specific merge conflict scenarios should Atlas handle?
2. When should Atlas activate? (All PRs? Only when conflicts detected?)
3. Should Bolt run on every deployment or only critical ones?
4. What deployment issues should Bolt detect and respond to?

### **Architecture Questions:**
5. Where do Atlas and Bolt fit in the Rex → Cleo → Tess workflow?
6. Should they be sequential or parallel stages?
7. What happens if Atlas/Bolt fail? Rollback? Alert?
8. How do we prevent infinite remediation loops?

### **Operations Questions:**
9. What Kubernetes permissions does Bolt need for monitoring?
10. Should Bolt have write access to fix deployment issues?
11. How do we test Atlas without creating real merge conflicts?
12. What metrics should we track for these agents?

---

## Summary

**Overall Progress: 35% Complete**

| Component | Status | Completion |
|-----------|--------|------------|
| Agent Definitions | ✅ Complete | 100% |
| Avatars & Assets | ✅ Complete | 100% |
| Documentation | ✅ Complete | 100% |
| Setup Scripts | ✅ Complete | 100% |
| Infrastructure Templates | ✅ Complete | 100% |
| GitHub Apps | ❌ Not Started | 0% |
| Vault Credentials | ❌ Not Started | 0% |
| ExternalSecrets | ❌ Not Started | 0% |
| MCP Configuration | ❌ Not Started | 0% |
| Workflow Integration | ❌ Not Started | 0% |
| Event Triggers | ❌ Not Started | 0% |
| Testing | ❌ Not Started | 0% |

**Next Critical Action:** Run GitHub App creation script and decide on workflow architecture

**Estimated Time to Full Completion:** 5-7 days of focused engineering work

---

## Conclusion

The foundation for Atlas and Bolt is solid - agent definitions are complete, professional, and well-thought-out. The setup automation is excellent. However, the **workflow integration is the critical missing piece** that requires architectural decisions before implementation.

Recommend: **Schedule a design session to define Atlas/Bolt activation points and workflow integration** before proceeding with credential setup and implementation.




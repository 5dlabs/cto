# Atlas & Bolt Agent Setup Guide

This guide walks through adding two new agents to the CTO platform:
- **Atlas**: Integration & Merge Specialist
- **Bolt**: DevOps & Deployment Specialist

## Quick Start

```bash
# For Atlas (Integration Agent)
./scripts/setup-new-agent.sh atlas "Integration & Merge Specialist" \
  "https://raw.githubusercontent.com/5dlabs/.github/main/profile/assets/atlas-avatar.png"

# For Bolt (DevOps Agent)
./scripts/setup-new-agent.sh bolt "DevOps & Deployment Specialist" \
  "https://raw.githubusercontent.com/5dlabs/.github/main/profile/assets/bolt-avatar.png"
```

---

## Avatar Generation Prompts

### Atlas - Integration & Merge Specialist

```
Sleek anthropomorphic silverback gorilla in cyberpunk style, wearing black-rimmed smart glasses, tailored techwear vest over compression shirt, holding a holographic tablet displaying merging git branches, surrounded by glowing code integration diagrams and flowing merge conflict visualizations with a 5DLabs badge, neon-lit integration lab with multiple code streams converging in the background, realistic digital art with structural metallic accents and green merge-success highlights.
```

**Character Rationale**:
- **Gorilla** = Strength, reliability, holds things together (Atlas holds the world)
- **Merging branches** = Integration specialist
- **Green highlights** = Successful conflict resolution
- **Converging streams** = Bringing code together

---

### Bolt - DevOps & Deployment Specialist

```
Sleek anthropomorphic cheetah in cyberpunk style, wearing black-rimmed smart glasses, tailored techwear jacket with lightning bolt accents, holding a holographic deployment trigger device, surrounded by glowing CI/CD pipeline visualizations and rapid-fire deployment status holograms with a 5DLabs badge, neon-lit cloud infrastructure control center with lightning effects in the background, realistic digital art with electric yellow and orange metallic accents.
```

**Character Rationale**:
- **Cheetah** = Speed, agility, precision (fastest deployments)
- **Lightning accents** = Bolt name reinforcement
- **Electric colors** = Energy and rapid execution
- **CI/CD visualizations** = DevOps mastery

---

## Step-by-Step Setup Process

### Step 1: Generate Avatars

1. Use the prompts above with DALL-E 3, Midjourney, or your preferred AI image generator
2. Generate at 512x512 or higher resolution
3. Export as PNG with transparent background (if possible)
4. Save as:
   - `atlas-avatar.png`
   - `bolt-avatar.png`

### Step 2: Upload Avatars to GitHub

```bash
# Clone the .github repository
git clone git@github.com:5dlabs/.github.git /tmp/5dlabs-github
cd /tmp/5dlabs-github

# Create avatars directory if it doesn't exist
mkdir -p profile/assets

# Copy your generated avatars
cp ~/Downloads/atlas-avatar.png profile/assets/
cp ~/Downloads/bolt-avatar.png profile/assets/

# Commit and push
git add profile/assets/atlas-avatar.png profile/assets/bolt-avatar.png
git commit -m "feat: add Atlas and Bolt agent avatars"
git push origin main
```

Avatar URLs will be:
- Atlas: `https://raw.githubusercontent.com/5dlabs/.github/main/profile/assets/atlas-avatar.png`
- Bolt: `https://raw.githubusercontent.com/5dlabs/.github/main/profile/assets/bolt-avatar.png`

### Step 3: Create GitHub Apps

#### For Atlas (Integration Agent)

1. Go to: https://github.com/organizations/5dlabs/settings/apps/new

2. **Fill in details**:
   - **GitHub App Name**: `5DLabs-Atlas`
   - **Description**: `AI Integration & Merge Specialist at 5D Labs - Automated merge conflict resolution for multi-agent development orchestration`
   - **Homepage URL**: `https://github.com/5dlabs/cto`
   - **Webhook**: Uncheck "Active" (not needed initially)

3. **Repository Permissions**:
   - **Contents**: Read & Write ✅
   - **Pull requests**: Read & Write ✅
   - **Workflows**: Read-only ✅
   - **Metadata**: Read-only ✅ (automatically selected)
   - **Checks**: Read-only ✅

4. **Subscribe to events**: (optional, can configure later)
   - Pull request
   - Pull request review
   - Push
   - Check suite
   - Status

5. **Where can this app be installed?**
   - Select: "Only on this account"

6. **Create the app** ✅

7. **After creation**:
   - Note the **App ID** (shown at top)
   - Note the **Client ID** (in "About" section)
   - Click "Generate a private key" and download the `.pem` file

#### For Bolt (DevOps Agent)

1. Go to: https://github.com/organizations/5dlabs/settings/apps/new

2. **Fill in details**:
   - **GitHub App Name**: `5DLabs-Bolt`
   - **Description**: `AI DevOps & Deployment Specialist at 5D Labs - Infrastructure operations and automated deployment management`
   - **Homepage URL**: `https://github.com/5dlabs/cto`
   - **Webhook**: Uncheck "Active" (not needed initially)

3. **Repository Permissions**:
   - **Contents**: Read & Write ✅
   - **Deployments**: Read & Write ✅
   - **Actions**: Read & Write ✅
   - **Metadata**: Read-only ✅ (automatically selected)
   - **Checks**: Read & Write ✅
   - **Pull requests**: Read-only ✅

4. **Subscribe to events**: (optional)
   - Deployment
   - Deployment status
   - Workflow run
   - Check suite
   - Push
   - Release

5. **Where can this app be installed?**
   - Select: "Only on this account"

6. **Create the app** ✅

7. **After creation**:
   - Note the **App ID**
   - Note the **Client ID**
   - Click "Generate a private key" and download the `.pem` file

### Step 4: Run Setup Script

```bash
# For Atlas
./scripts/setup-new-agent.sh atlas "Integration & Merge Specialist" \
  "https://raw.githubusercontent.com/5dlabs/.github/main/profile/assets/atlas-avatar.png"

# Follow the prompts to enter App ID, Client ID, and private key path

# For Bolt
./scripts/setup-new-agent.sh bolt "DevOps & Deployment Specialist" \
  "https://raw.githubusercontent.com/5dlabs/.github/main/profile/assets/bolt-avatar.png"

# Follow the prompts
```

The script will:
- ✅ Generate configuration snippets
- ✅ Store credentials in Vault (if available)
- ✅ Create temporary config files for easy copying
- ✅ Provide next steps

### Step 5: Update Configuration Files

#### 5a. Update `infra/charts/controller/values.yaml`

Add the generated snippets from `/tmp/agent-atlas-values-snippet.yaml` and `/tmp/agent-bolt-values-snippet.yaml` to the `agents:` section.

#### 5b. Update `infra/secret-store/agent-secrets-external-secrets.yaml`

Add the ExternalSecret configurations from `/tmp/agent-atlas-external-secrets.yaml` and `/tmp/agent-bolt-external-secrets.yaml`.

#### 5c. Store Credentials in Vault

If not done automatically:

```bash
# For Atlas
vault kv put secret/github-app-atlas \
  app_id="<APP_ID>" \
  client_id="<CLIENT_ID>" \
  private_key=@/path/to/atlas-private-key.pem

# For Bolt
vault kv put secret/github-app-bolt \
  app_id="<APP_ID>" \
  client_id="<CLIENT_ID>" \
  private_key=@/path/to/bolt-private-key.pem
```

### Step 6: Install Apps to Repositories

1. Go to each app's installation page:
   - Atlas: `https://github.com/organizations/5dlabs/settings/apps/5dlabs-atlas/installations`
   - Bolt: `https://github.com/organizations/5dlabs/settings/apps/5dlabs-bolt/installations`

2. Click "Install" and select repositories (or "All repositories")

3. Confirm installation

### Step 7: Commit Changes

```bash
git add infra/charts/controller/values.yaml
git add infra/secret-store/agent-secrets-external-secrets.yaml
git commit -m "feat: add Atlas and Bolt agents

- Atlas: Integration & Merge Specialist (5DLabs-Atlas)
- Bolt: DevOps & Deployment Specialist (5DLabs-Bolt)
- Add GitHub App configurations and ExternalSecrets
- Add agent system prompts and expertise definitions"

git push origin HEAD
```

### Step 8: Verify Deployment

After ArgoCD syncs the changes:

```bash
# Check ExternalSecrets created
kubectl get externalsecret -n agent-platform | grep -E "atlas|bolt"

# Check secrets populated
kubectl get secret -n agent-platform | grep -E "atlas|bolt"

# Verify agents configmap updated
kubectl get configmap -n agent-platform controller-agents -o yaml | grep -E "atlas|bolt"
```

---

## Workflow Integration

### Atlas Integration Points

**Triggered After**: Tess approval (PR has `approved` label)

**Workflow Addition**:
```yaml
# Add to play workflow after Tess approval
- name: atlas-merge-integration
  template: atlas-merge-work
  arguments:
    parameters:
      - name: pr-number
        value: "{{steps.implementation.outputs.parameters.pr-number}}"
```

**Atlas Responsibilities**:
1. Check PR for merge conflicts with main
2. If conflicts: Fetch main, rebase, resolve, push
3. If no conflicts: Proceed to merge
4. Run smoke tests after merge
5. Validate main branch health

### Bolt Integration Points

**Triggered After**: Atlas merges to main

**Workflow Addition**:
```yaml
# Add after Atlas merge
- name: bolt-deployment
  template: bolt-deploy-work
  arguments:
    parameters:
      - name: service-name
        value: "{{workflow.parameters.service}}"
```

**Bolt Responsibilities**:
1. Monitor ArgoCD sync status
2. Validate Kubernetes deployments
3. Check application health endpoints
4. Run smoke tests against deployed service
5. Rollback on failures
6. Update deployment status in GitHub

---

## New Agent Flow

**Complete Multi-Agent Flow**:

```
Morgan (Docs) 
    ↓
Rex/Blaze (Implementation) → Creates PR
    ↓
Cleo (Quality Review) → Approves/Requests Changes
    ↓
Cipher (Security Review) → Approves/Requests Changes
    ↓
Tess (QA Testing) → Approves/Requests Changes
    ↓
Atlas (Merge Integration) → Resolves conflicts, merges to main
    ↓
Bolt (DevOps) → Validates deployment, ensures accessibility
    ↓
🎉 Feature Live in Production
```

---

## Merge Conflict Detection Sensor

Create new sensor for Atlas activation:

```yaml
# infra/gitops/resources/github-webhooks/atlas-merge-conflict-sensor.yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: atlas-merge-conflict-detector
  namespace: argo
spec:
  dependencies:
    - name: pr-updated
      eventSourceName: github
      eventName: org
      filters:
        data:
          - path: headers.X-Github-Event
            value: ["pull_request"]
          - path: body.action
            value: ["synchronize", "labeled"]
          # Trigger Atlas when PR has 'approved' label and conflicts
          - path: body.pull_request.labels[*].name
            value: ["approved"]
          - path: body.pull_request.mergeable
            type: bool
            value: ["false"]
  
  triggers:
    - template:
        name: trigger-atlas-merge-work
        k8s:
          operation: create
          source:
            resource:
              apiVersion: agents.5dlabs.io/v1alpha1
              kind: CodeRun
              metadata:
                generateName: atlas-merge-
                namespace: agent-platform
              spec:
                taskId: "{{workflow.parameters.task-id}}"
                githubApp: "5DLabs-Atlas"
                service: "{{workflow.parameters.service}}"
                # Additional Atlas-specific configuration
```

---

## Testing the New Agents

### Test Atlas

```bash
# Create a test PR with conflicts
git checkout -b test-atlas-conflict
# Make changes that will conflict with main
git push origin test-atlas-conflict

# Create PR and get it approved by Tess
# Atlas should detect conflicts and resolve them
```

### Test Bolt

```bash
# After Atlas merges to main
# Watch Bolt monitor the deployment
kubectl logs -n agent-platform -l app.kubernetes.io/name=controller,agent=bolt --follow
```

---

## Troubleshooting

### GitHub App not appearing in agent list

Check:
```bash
kubectl get configmap -n agent-platform controller-agents -o yaml
```

Ensure values.yaml changes were synced by ArgoCD.

### Credentials not found

Check ExternalSecrets:
```bash
kubectl get externalsecret -n agent-platform github-app-atlas -o yaml
kubectl describe externalsecret -n agent-platform github-app-atlas
```

Verify Vault has the secrets:
```bash
vault kv get secret/github-app-atlas
vault kv get secret/github-app-bolt
```

### Agent not triggering

Check workflow templates include the new agent stages.

---

## Architecture Impact

### Agent Count
- Before: 6 agents (Morgan, Rex, Blaze, Cleo, Cipher, Tess)
- After: 8 agents (+ Atlas, + Bolt)

### Workflow Stages
- Before: 4 stages (Implementation, Quality, Security, Testing)
- After: 6 stages (+ Integration, + Deployment)

### Average Task Duration
- Before: 30-60 minutes per task
- After: 45-90 minutes per task (includes merge resolution + deployment validation)

---

## Cost Considerations

### API Costs (per task)
- **Atlas**: ~$0.10-0.50 (merge conflict resolution, usually quick)
- **Bolt**: ~$0.05-0.20 (deployment validation, monitoring)
- **Total increase**: ~$0.15-0.70 per task

### Infrastructure Costs
- 2 additional PVCs for agent workspaces: ~$0.20/month each
- Minimal compute overhead (agents run on-demand)

---

## Next Steps After Setup

1. **Create Atlas workflow template** (`infra/charts/controller/templates/workflowtemplates/atlas-merge-template.yaml`)
2. **Create Bolt workflow template** (`infra/charts/controller/templates/workflowtemplates/bolt-deploy-template.yaml`)
3. **Add Atlas/Bolt to play workflow** (insert between Tess and completion)
4. **Create Atlas container script** (`infra/charts/controller/agent-templates/code/claude/container-atlas.sh.hbs`)
5. **Create Bolt container script** (`infra/charts/controller/agent-templates/code/claude/container-bolt.sh.hbs`)
6. **Add merge conflict detection sensor** (Argo Events)
7. **Add deployment monitoring sensor** (Argo Events)
8. **Test end-to-end** with a simple task

---

## References

- [GitHub App Manifest API](./github-app-manifest-api.md)
- [Agent Persona Creator Design](./agent-persona-creator-design.md)
- [Multi-Agent Orchestration Architecture](../../README.md)




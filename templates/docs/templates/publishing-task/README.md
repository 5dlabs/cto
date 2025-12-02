# Publishing Task Template

This template is for the **final publishing/deployment task** that comes after all implementation and integration tasks are complete.

## Purpose

The publishing task ensures:
- All PRs are merged to main
- Application is deployed to Kubernetes
- Ngrok ingress is configured
- Public URLs are provided for access
- Application is smoke tested and verified

## Usage

Morgan (docs agent) should create this as the LAST task in the task list, after integration tasks.

## Task Structure

**Dependencies:** ALL previous tasks (including integration tasks)
**Agent Hint:** `deployment` or `integration`
**Priority:** `critical`

## Template Files

- **prompt.md** - Agent instructions for deployment and publishing
- **task.md** - Formatted task documentation
- **acceptance-criteria.md** - Deployment validation checklist
- **task.txt** - Plain text version
- **task.xml** - Structured metadata

## Key Responsibilities

1. **PR Merge Coordination** (FIRST STEP - MANDATORY)
   - Check all PRs are approved
   - Merge in dependency order
   - Verify main branch has all changes

2. **Build & Push**
   - Build Docker image from main
   - Push to container registry

3. **Kubernetes Deployment**
   - Create deployment manifests
   - Apply to cluster
   - Verify pods healthy

4. **Ngrok Ingress Setup**
   - Configure Ngrok ingress resource
   - Obtain public URL
   - Verify accessibility

5. **Smoke Testing**
   - Test health endpoints
   - Verify key API endpoints
   - Validate end-to-end flows

6. **Documentation**
   - Provide public URLs
   - Document deployment status
   - Create access instructions


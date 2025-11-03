# Bolt - Public Deployment Specialist Guide

**Agent:** Bolt (5DLabs-Bolt)  
**Role:** Public Deployment & Accessibility  
**Primary Mission:** Make applications publicly accessible via ArgoCD + ngrok

---

## What Bolt Does

Bolt is responsible for taking code that's been merged and making it **publicly accessible** on the internet. It bridges the gap between "code deployed to Kubernetes" and "application available to the world."

### Core Responsibilities

1. **Creates ArgoCD Applications** - Generates Application CRDs for services
2. **Waits for Deployment** - Monitors ArgoCD sync and health status
3. **Sets up ngrok Ingress** - Creates Tunnel CRDs for public access
4. **Verifies Accessibility** - Tests that the app responds to HTTP requests
5. **Posts Public URLs** - Comments on GitHub PRs with the live URL

---

## When Bolt Runs

Bolt fits into the multi-agent workflow **after code is merged** and **before QA testing**:

```
Rex implements code
  ↓
Cleo reviews quality
  ↓
Atlas merges PR
  ↓
🚀 BOLT PUBLISHES APPLICATION 🚀
  ↓
Tess runs E2E tests (using Bolt's public URL)
  ↓
Done
```

---

## How Bolt Works

### Step 1: Create ArgoCD Application

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: my-service
  namespace: argocd
spec:
  project: default
  source:
    repoURL: https://github.com/org/repo.git
    targetRevision: main
    path: helm  # Path to Helm chart
  destination:
    server: https://kubernetes.default.svc
    namespace: agent-platform
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
```

**What this does:**
- Tells ArgoCD to deploy the application
- Configures auto-sync so changes deploy automatically
- Sets up namespace and resource management

### Step 2: Wait for ArgoCD Sync

Bolt waits (up to 5 minutes) for:
- **Sync Status:** `Synced` (Git matches cluster)
- **Health Status:** `Healthy` (all resources running)

```bash
kubectl get application my-service -n argocd -o jsonpath='{.status.sync.status}'
# Expected: "Synced"

kubectl get application my-service -n argocd -o jsonpath='{.status.health.status}'
# Expected: "Healthy"
```

### Step 3: Create ngrok Tunnel

```yaml
apiVersion: ngrok.k8s.ngrok.com/v1alpha1
kind: Tunnel
metadata:
  name: my-service-ngrok
  namespace: agent-platform
spec:
  forwardsTo: my-service:80  # Internal service:port
```

**What this does:**
- Creates a public ngrok URL (e.g., `https://abc123.ngrok.io`)
- Forwards traffic to the Kubernetes service
- Provides instant public accessibility without DNS setup

### Step 4: Get Public URL

```bash
kubectl get tunnel my-service-ngrok -n agent-platform -o jsonpath='{.status.url}'
# Returns: "https://abc123.ngrok.io"
```

### Step 5: Verify Accessibility

```bash
curl -s -o /dev/null -w "%{http_code}" https://abc123.ngrok.io
# Expected: 200, 301, or 302
```

### Step 6: Post to GitHub PR

Bolt comments on the PR with:

```markdown
## 🚀 Bolt: Application Published

**Service:** `my-service`  
**Public URL:** https://abc123.ngrok.io  
**Status:** ✅ Application is live and publicly accessible

🔗 **Access your application:** https://abc123.ngrok.io

---
*Deployed via ArgoCD | Exposed via ngrok*
```

---

## Environment Variables

Bolt receives these from the workflow:

| Variable | Description | Default |
|----------|-------------|---------|
| `SERVICE` | Name of the service to deploy | *Required* |
| `NAMESPACE` | Kubernetes namespace | `agent-platform` |
| `REPOSITORY` | Git repository URL | From workflow |
| `BRANCH` | Git branch to deploy | `main` |
| `PR_NUMBER` | GitHub PR to comment on | From trigger |
| `ARGOCD_PATH` | Path to Helm chart in repo | `helm` |
| `SERVICE_PORT` | Internal service port | `80` |

---

## Example Workflow

### Scenario: Deploy a Web App

**User Action:** Merge PR #42 with a new React app

**Bolt's Workflow:**

1. **Trigger:** Deployment webhook or workflow completion
2. **Create ArgoCD App:**
   ```bash
   kubectl apply -f argocd-app.yaml
   # Creates: react-app in argocd namespace
   ```

3. **Wait for Sync:**
   ```
   Checking: react-app
   Sync: Synced, Health: Progressing... (30s)
   Sync: Synced, Health: Healthy ✅
   ```

4. **Create ngrok Tunnel:**
   ```bash
   kubectl apply -f ngrok-tunnel.yaml
   # Creates: react-app-ngrok in agent-platform
   ```

5. **Get URL:**
   ```
   Public URL: https://7a2b3c4d.ngrok.io ✅
   ```

6. **Verify:**
   ```bash
   curl -I https://7a2b3c4d.ngrok.io
   # HTTP/1.1 200 OK ✅
   ```

7. **Post to PR #42:**
   > 🚀 **Bolt: Application Published**
   >
   > Your React app is live at: https://7a2b3c4d.ngrok.io

**Result:** Team can immediately access and test the deployed app!

---

## Troubleshooting

### Issue: ArgoCD Application Not Syncing

**Symptoms:** Stuck at "OutOfSync" status

**Bolt's Actions:**
1. Check application status: `kubectl describe application <name> -n argocd`
2. Look for sync errors in status
3. Post diagnostic info to PR

**Common Causes:**
- Invalid Helm chart path
- Missing namespace
- RBAC permission issues
- Invalid resource definitions

### Issue: ngrok Tunnel No URL

**Symptoms:** Tunnel created but status.url is empty

**Bolt's Actions:**
1. Wait up to 30 seconds for provisioning
2. Check ngrok operator logs
3. Fall back to checking Ingress resources

**Common Causes:**
- ngrok operator not running
- ngrok API key not configured
- Rate limit exceeded
- Service doesn't exist yet

### Issue: Application Not Responding

**Symptoms:** Public URL exists but returns errors

**Bolt's Actions:**
1. Verify pods are Running: `kubectl get pods -n <namespace>`
2. Check service exists: `kubectl get svc <service>`
3. Review application logs
4. Post diagnostic steps to PR

**Common Causes:**
- Application crash on startup
- Wrong service port
- Container port mismatch
- Missing environment variables

---

## Future Enhancements

### Multiple Ingress Types

Eventually, Bolt will support other ingress types beyond ngrok:

```yaml
# Service annotation to specify ingress type
apiVersion: v1
kind: Service
metadata:
  annotations:
    bolt.5dlabs.ai/ingress-type: "alb"  # or nginx, traefik, etc.
```

**Supported Types (Future):**
- `ngrok` - Current default
- `alb` - AWS Application Load Balancer
- `nginx` - NGINX Ingress Controller
- `traefik` - Traefik Proxy
- `cloudflare` - Cloudflare Tunnel

### Custom Domains

Support for custom domain configuration:

```yaml
metadata:
  annotations:
    bolt.5dlabs.ai/domain: "app.example.com"
```

### SSL/TLS Certificates

Automatic certificate provisioning:
- Let's Encrypt via cert-manager
- AWS Certificate Manager (ACM)
- Custom certificate secrets

---

## Integration with Other Agents

### With Rex (Implementation)
- Rex creates the code
- Bolt deploys and publishes it
- Rex can reference Bolt's URLs in documentation

### With Atlas (Integration)
- Atlas ensures PRs are cleanly merged
- Bolt publishes the merged result
- Atlas validates deployment succeeded

### With Tess (QA/Testing)
- Tess waits for Bolt to provide public URL
- Tess runs E2E tests against Bolt's URL
- Tess validates functionality matches requirements

### With Cleo (Code Quality)
- Cleo ensures code quality before merge
- Bolt deploys the quality-checked code
- Bolt verifies deployment health

---

## Configuration Examples

### Basic Web App

```yaml
# Minimal configuration
service: my-web-app
namespace: agent-platform
repository: https://github.com/org/repo.git
branch: main
argocd_path: helm
service_port: 3000
```

### API Service

```yaml
# API with custom port
service: api-backend
namespace: production
repository: https://github.com/org/api.git
branch: release
argocd_path: charts/api
service_port: 8080
```

### Multi-Environment

```yaml
# Different namespaces per environment
dev:
  service: app-dev
  namespace: dev
  branch: develop

staging:
  service: app-staging
  namespace: staging
  branch: staging

production:
  service: app-prod
  namespace: production
  branch: main
```

---

## Success Metrics

Bolt considers deployment successful when:

- ✅ ArgoCD Application created (or validated)
- ✅ Application status: `Synced` + `Healthy`
- ✅ All pods in `Running` state
- ✅ ngrok Tunnel created successfully
- ✅ Public URL retrieved from tunnel
- ✅ Application responds to HTTP (200/301/302)
- ✅ Public URL posted to GitHub PR

---

## Summary

**Bolt makes deployment simple:**
1. Code merged? ✅
2. ArgoCD deploys it ✅
3. ngrok makes it public ✅
4. Team gets the URL ✅
5. Ready for testing ✅

**Zero manual work required. Just merge and go!** 🚀


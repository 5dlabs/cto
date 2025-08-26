# Component Integration Strategy

## Overview

This document outlines our approach to integrating best-in-class tools for the multi-agent platform, keeping production-tested solutions like NGrok and Cloudflare for tunneling while replacing proprietary VPN and task management systems with open-source alternatives.

## Final Technology Stack

### 1. Webhook Tunneling: NGrok & Cloudflare (KEEPING)

#### Rationale
- **Industry Standards**: Both are battle-tested, reliable solutions
- **Free Tiers**: Generous limits perfect for development
- **Developer Experience**: Excellent documentation and tooling
- **Production Ready**: Seamless path from development to production

#### Integration Strategy
```yaml
# Platform will support both providers
tunneling:
  provider: auto  # auto-select based on environment
  
  ngrok:
    enabled: true
    # Free tier for development
    # $8/month for persistent domains
    
  cloudflare:
    enabled: true  
    # Free tier with named tunnels
    # Zero Trust security built-in
```

See `TUNNELING_STRATEGY.md` for complete implementation details.

### 2. VPN/Remote Access: WireGuard (REPLACING Twingate)

#### Current Twingate Usage
- **Purpose**: Secure remote access to cluster services
- **Cost**: $5-10/user/month  
- **Files**: `twingate-pastoral.yaml`, `twingate-therapeutic.yaml`

#### WireGuard Replacement Strategy

##### Why WireGuard?
- **Fully Open Source**: No vendor lock-in, complete control
- **Kernel-Level Performance**: Built into Linux kernel 5.6+
- **Modern Cryptography**: ChaCha20, Poly1305, Curve25519
- **Minimal Attack Surface**: ~4,000 lines of code vs OpenVPN's 100,000+
- **Cross-Platform**: Works on Linux, Windows, macOS, iOS, Android
- **Simple Configuration**: Much easier than IPSec or OpenVPN

##### WireGuard Implementation
```yaml
# wireguard-server.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: wireguard-config
data:
  wg0.conf: |
    [Interface]
    Address = 10.0.0.1/24
    ListenPort = 51820
    PrivateKey = $(PRIVATE_KEY)
    
    [Peer]
    PublicKey = $(PEER_PUBLIC_KEY)
    AllowedIPs = 10.0.0.2/32
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: wireguard
spec:
  template:
    spec:
      containers:
      - name: wireguard
        image: linuxserver/wireguard:latest
        cap_add:
        - NET_ADMIN
        - SYS_MODULE
```

##### WireGuard Configuration for Platform
```yaml
# wireguard-server-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: wireguard-config
  namespace: agent-platform
data:
  wg0.conf: |
    [Interface]
    Address = 10.200.0.1/24
    ListenPort = 51820
    PrivateKey = ${SERVER_PRIVATE_KEY}
    PostUp = iptables -A FORWARD -i wg0 -j ACCEPT; iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE
    PostDown = iptables -D FORWARD -i wg0 -j ACCEPT; iptables -t nat -D POSTROUTING -o eth0 -j MASQUERADE
    
    # Agent platform admin
    [Peer]
    PublicKey = ${ADMIN_PUBLIC_KEY}
    AllowedIPs = 10.200.0.2/32
    
    # Developer access
    [Peer]
    PublicKey = ${DEV_PUBLIC_KEY}
    AllowedIPs = 10.200.0.3/32
```

##### Automated User Management
```go
// pkg/vpn/wireguard.go
package vpn

import (
    "golang.zx2c4.com/wireguard/wgctrl"
    "golang.zx2c4.com/wireguard/wgctrl/wgtypes"
)

type WireGuardManager struct {
    client     *wgctrl.Client
    configPath string
    subnet     string
}

func (w *WireGuardManager) AddUser(username string) (*UserConfig, error) {
    // Generate key pair
    privateKey, _ := wgtypes.GeneratePrivateKey()
    publicKey := privateKey.PublicKey()
    
    // Assign IP from pool
    ip := w.getNextAvailableIP()
    
    // Update server config
    peer := wgtypes.PeerConfig{
        PublicKey:  publicKey,
        AllowedIPs: []net.IPNet{{IP: ip, Mask: net.CIDRMask(32, 32)}},
    }
    
    // Generate client config
    clientConfig := fmt.Sprintf(`
[Interface]
Address = %s/24
PrivateKey = %s
DNS = 10.200.0.1

[Peer]
PublicKey = %s
Endpoint = %s:51820
AllowedIPs = 10.200.0.0/24, 10.96.0.0/12
PersistentKeepalive = 25
`,
        ip, privateKey.String(), w.serverPublicKey, w.endpoint)
    
    return &UserConfig{
        Username: username,
        Config:   clientConfig,
        QRCode:   generateQRCode(clientConfig),
    }, nil
}
```

##### Web UI for WireGuard Management
```yaml
# wg-easy-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: wg-easy
  namespace: agent-platform
spec:
  template:
    spec:
      containers:
      - name: wg-easy
        image: weejewel/wg-easy
        ports:
        - containerPort: 51820/udp  # WireGuard
        - containerPort: 51821/tcp  # Web UI
        env:
        - name: WG_HOST
          value: vpn.agent-platform.local
        - name: PASSWORD
          valueFrom:
            secretKeyRef:
              name: wg-easy-password
              key: password
        volumeMounts:
        - name: wireguard-data
          mountPath: /etc/wireguard
```

##### Integration with Platform Installer
```bash
$ agent-platform install

[5/10] Remote Access Configuration:
  How will team members access the cluster?
  
  > [1] WireGuard VPN (Recommended, open source)
    [2] kubectl port-forward only (simple but limited)
    [3] Direct access (I have public IPs)
    
  Selection: 1
  
  WireGuard Setup:
  ✓ Generating server keys...
  ✓ Creating network 10.200.0.0/24
  ✓ Deploying WireGuard server...
  
  Admin user configuration:
  Username: admin
  
  Your WireGuard configuration has been saved to:
  ~/.agent-platform/wireguard-admin.conf
  
  To connect:
  1. Install WireGuard client: https://www.wireguard.com/install/
  2. Import the configuration file
  3. Connect to 'agent-platform' VPN
  
  Web UI available at: http://10.200.0.1:51821
  Password: <auto-generated>
```

##### Alternative: Simple Port Forwarding (for Solo Developers)
```go
// pkg/access/portforward.go
package access

// For users who don't need full VPN
type PortForwardManager struct {
    kubeconfig string
}

func (p *PortForwardManager) CreateAccessScript() string {
    return `#!/bin/bash
# agent-platform-access.sh

echo "Starting port forwards for agent platform..."

# ArgoCD UI
kubectl port-forward -n argocd svc/argocd-server 8080:80 &

# Argo Workflows UI  
kubectl port-forward -n argo svc/argo-server 2746:2746 &

# Grafana
kubectl port-forward -n monitoring svc/grafana 3000:3000 &

echo "Access URLs:"
echo "  ArgoCD:         http://localhost:8080"
echo "  Argo Workflows: http://localhost:2746"
echo "  Grafana:        http://localhost:3000"
echo ""
echo "Press Ctrl+C to stop all port forwards"

wait
`
}
```

### 3. Taskmaster (Task Management System)

#### Current Usage
- **Purpose**: Task tracking and project management
- **Integration**: Unknown level of integration
- **Cost**: Likely subscription-based

#### Open-Source Alternatives

##### Option A: GitHub Projects Integration (Recommended)
```go
// pkg/tasks/github_projects.go
package tasks

import (
    "github.com/google/go-github/v50/github"
)

type GitHubProjectsBackend struct {
    client    *github.Client
    projectID int64
}

func (g *GitHubProjectsBackend) CreateTask(task Task) (*Task, error) {
    card := &github.ProjectCard{
        Note: github.String(task.Description),
    }
    
    created, _, err := g.client.Projects.CreateProjectCard(
        context.Background(),
        g.projectID,
        card,
    )
    
    if err != nil {
        return nil, err
    }
    
    task.ID = *created.ID
    return &task, nil
}

func (g *GitHubProjectsBackend) UpdateTaskStatus(taskID int64, status string) error {
    // Move card between columns based on status
    columnID := g.getColumnForStatus(status)
    
    _, err := g.client.Projects.MoveProjectCard(
        context.Background(),
        taskID,
        &github.ProjectCardMoveOptions{
            Position: "top",
            ColumnID: columnID,
        },
    )
    
    return err
}
```

**Pros**:
- Native GitHub integration
- Free with GitHub
- API access included

**Cons**:
- Limited features vs dedicated tools
- GitHub-specific

##### Option B: Linear API Integration
```typescript
// linear-integration.ts
import { LinearClient } from "@linear/sdk";

class LinearTaskBackend {
    private client: LinearClient;
    
    constructor(apiKey: string) {
        this.client = new LinearClient({ apiKey });
    }
    
    async createTask(task: Task) {
        const issue = await this.client.issueCreate({
            title: task.title,
            description: task.description,
            teamId: this.teamId,
            stateId: await this.getStateId("todo"),
        });
        
        return issue.issue;
    }
    
    async transitionTask(taskId: string, state: string) {
        const stateId = await this.getStateId(state);
        await this.client.issueUpdate(taskId, { stateId });
    }
}
```

**Pros**:
- Excellent API
- Developer-focused
- Good free tier

**Cons**:
- External service
- Requires account

##### Option C: Built-in Task System
```go
// pkg/tasks/internal.go
package tasks

type InternalTaskSystem struct {
    db *sql.DB
}

type Task struct {
    ID          string
    Title       string
    Description string
    Status      string
    Agent       string
    Created     time.Time
    Updated     time.Time
    Metadata    map[string]interface{}
}

func (t *InternalTaskSystem) CreateTask(task Task) error {
    query := `
        INSERT INTO tasks (id, title, description, status, agent, metadata)
        VALUES ($1, $2, $3, $4, $5, $6)
    `
    
    metadata, _ := json.Marshal(task.Metadata)
    _, err := t.db.Exec(query, 
        task.ID,
        task.Title,
        task.Description,
        task.Status,
        task.Agent,
        metadata,
    )
    
    return err
}
```

**Schema**:
```sql
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    description TEXT,
    status VARCHAR(50) DEFAULT 'pending',
    agent VARCHAR(50),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    metadata JSONB,
    parent_id UUID REFERENCES tasks(id),
    workflow_id VARCHAR(100),
    pr_number INTEGER,
    repository TEXT
);

CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_agent ON tasks(agent);
CREATE INDEX idx_tasks_workflow ON tasks(workflow_id);
```

##### Option D: Jira Integration
```java
// JiraTaskBackend.java
public class JiraTaskBackend implements TaskBackend {
    private JiraRestClient jiraClient;
    
    public Task createTask(TaskRequest request) {
        IssueInputBuilder issueBuilder = new IssueInputBuilder()
            .setProjectKey("AGENT")
            .setSummary(request.getTitle())
            .setDescription(request.getDescription())
            .setIssueType("Task");
            
        Issue issue = jiraClient.getIssueClient()
            .createIssue(issueBuilder.build())
            .claim();
            
        return convertToTask(issue);
    }
}
```

**Pros**:
- Enterprise standard
- Rich features
- Extensive integrations

**Cons**:
- Complex
- Expensive
- Heavy for this use case

#### Implementation Strategy
```go
// pkg/tasks/manager.go
package tasks

type TaskBackend interface {
    CreateTask(task Task) (*Task, error)
    UpdateTask(id string, updates TaskUpdate) error
    GetTask(id string) (*Task, error)
    ListTasks(filter TaskFilter) ([]*Task, error)
    TransitionTask(id string, newStatus string) error
}

type TaskManager struct {
    backend TaskBackend
}

func NewTaskManager(config TaskConfig) *TaskManager {
    var backend TaskBackend
    
    switch config.Provider {
    case "github":
        backend = &GitHubProjectsBackend{
            ProjectID: config.GitHubProjectID,
        }
    case "linear":
        backend = &LinearBackend{
            APIKey: config.LinearAPIKey,
        }
    case "internal":
        backend = &InternalTaskSystem{
            DB: config.Database,
        }
    case "jira":
        backend = &JiraBackend{
            URL:      config.JiraURL,
            Username: config.JiraUsername,
            APIToken: config.JiraAPIToken,
        }
    default:
        // Default to internal system
        backend = &InternalTaskSystem{}
    }
    
    return &TaskManager{backend: backend}
}
```

### 4. Mail Server (Mailu)

#### Current Usage
- **Purpose**: Email notifications and alerts
- **Deployment**: Full mail server stack
- **Domain**: mail.5dlabs.ai

#### Replacement Strategy

##### Option A: SMTP Relay (Recommended for Most Users)
```yaml
# config.yaml
notifications:
  email:
    enabled: true
    provider: smtp
    smtp:
      host: smtp.gmail.com
      port: 587
      username: ${SMTP_USERNAME}
      password: ${SMTP_PASSWORD}
      from: notifications@example.com
```

##### Option B: SendGrid API
```go
// pkg/notifications/sendgrid.go
func (s *SendGridNotifier) Send(notification Notification) error {
    from := mail.NewEmail("Agent Platform", s.fromEmail)
    to := mail.NewEmail(notification.Recipient, notification.RecipientEmail)
    
    message := mail.NewSingleEmail(
        from,
        notification.Subject,
        to,
        notification.Body,
        notification.HTMLBody,
    )
    
    client := sendgrid.NewSendClient(s.apiKey)
    _, err := client.Send(message)
    return err
}
```

##### Option C: Webhook Notifications (No Email)
```go
// pkg/notifications/webhook.go
func (w *WebhookNotifier) Send(notification Notification) error {
    payload := map[string]interface{}{
        "event":     notification.Type,
        "timestamp": time.Now(),
        "data":      notification.Data,
    }
    
    return w.postJSON(w.webhookURL, payload)
}
```

## Unified Configuration Approach

```yaml
# platform-config.yaml
version: "1.0.0"

# Access Management
access:
  type: "none"  # none, wireguard, headscale, port-forward
  wireguard:
    enabled: false
    subnet: "10.0.0.0/24"
  
# Webhook Tunneling (for local development)
tunnels:
  provider: "localtunnel"  # none, ngrok, localtunnel, cloudflare, bore
  config:
    subdomain: "my-agent-platform"
    
# Task Management
tasks:
  provider: "internal"  # internal, github, linear, jira
  github:
    project_id: "${GITHUB_PROJECT_ID}"
  linear:
    api_key: "${LINEAR_API_KEY}"
    team_id: "${LINEAR_TEAM_ID}"
    
# Notifications
notifications:
  email:
    enabled: false
    provider: "smtp"  # smtp, sendgrid, ses
  webhook:
    enabled: true
    url: "${WEBHOOK_URL}"
  slack:
    enabled: false
    webhook: "${SLACK_WEBHOOK}"
```

## Migration Utilities

```bash
#!/bin/bash
# migrate-proprietary.sh

echo "Migrating from proprietary components..."

# Remove NGrok
kubectl delete -f kubernetes/ngrok-operator.yaml 2>/dev/null
kubectl delete -f kubernetes/ngrok-gateway.yaml 2>/dev/null

# Remove Twingate
kubectl delete -f kubernetes/twingate-*.yaml 2>/dev/null

# Remove Mailu
kubectl delete namespace mail-system 2>/dev/null

# Install open-source alternatives based on config
if [ "$TUNNEL_PROVIDER" = "localtunnel" ]; then
    npm install -g localtunnel
    echo "LocalTunnel installed for webhook routing"
fi

if [ "$VPN_PROVIDER" = "wireguard" ]; then
    kubectl apply -f alternatives/wireguard/
    echo "WireGuard VPN deployed"
fi

echo "Migration complete. Update your config.yaml with new settings."
```

## Decision Matrix

| Component | Best for Local Dev | Best for Teams | Best for Production | Best Overall |
|-----------|-------------------|----------------|--------------------|--------------| 
| **Tunnels** | LocalTunnel (free) | Cloudflare (reliable) | Real domain + ingress | Cloudflare |
| **VPN** | Port-forward | WireGuard | WireGuard/Headscale | WireGuard |
| **Tasks** | Internal | GitHub Projects | Linear/Jira | GitHub Projects |
| **Mail** | Webhook only | SMTP relay | SendGrid API | SMTP relay |

## Cost Comparison

### Previous (Proprietary)
- NGrok Pro: $20/month
- Twingate Team: $50/month (10 users)
- Taskmaster: $30/month (estimated)
- **Total: $100/month**

### Open-Source Stack
- LocalTunnel: $0
- WireGuard: $0 (self-hosted)
- GitHub Projects: $0
- SMTP Relay: $0 (using existing email)
- **Total: $0/month**

### Hybrid Approach (Recommended)
- Cloudflare Tunnel: $0 (free tier)
- WireGuard: $0 (self-hosted)
- Linear: $0 (free tier)
- SendGrid: $0 (free tier, 100 emails/day)
- **Total: $0/month** (with upgrade paths available)

## Implementation Priority

1. **Phase 1**: Replace tunneling (1 day)
   - Implement LocalTunnel provider
   - Add Cloudflare as option
   
2. **Phase 2**: Remove VPN dependency (2 days)
   - Default to kubectl port-forward
   - Add WireGuard as option
   
3. **Phase 3**: Task system abstraction (3 days)
   - Build internal task system
   - Add GitHub Projects integration
   
4. **Phase 4**: Notification system (1 day)
   - Replace mail server with SMTP
   - Add webhook notifications

## Conclusion

All proprietary components can be replaced with open-source alternatives without loss of core functionality. The recommended approach:

1. **LocalTunnel** for development webhook routing (upgrade to Cloudflare for production)
2. **WireGuard** for secure access (or kubectl port-forward for simplicity)
3. **GitHub Projects** for task management (native integration)
4. **SMTP relay** for notifications (or webhooks for simplicity)

This provides a **$0/month operational cost** for the base platform while maintaining professional capabilities and clear upgrade paths for enterprise users.
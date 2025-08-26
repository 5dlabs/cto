# Proprietary Component Replacement Strategy

## Overview

This document outlines open-source and self-hosted alternatives to the proprietary components currently used in the 5D Labs infrastructure, enabling the platform to be fully distributable without external dependencies.

## Component Analysis & Replacements

### 1. NGrok (Webhook Tunnel Service)

#### Current Usage
- **Purpose**: Expose local webhook endpoints to GitHub for development
- **Cost**: $8-39/month for persistent domains
- **Files**: `ngrok-operator.yaml`, `ngrok-gateway.yaml`

#### Open-Source Alternatives

##### Option A: Cloudflare Tunnel (Recommended for Production)
```yaml
# cloudflare-tunnel.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cloudflared
spec:
  replicas: 1
  template:
    spec:
      containers:
      - name: cloudflared
        image: cloudflare/cloudflared:latest
        args:
        - tunnel
        - --no-autoupdate
        - run
        - --token=$(TUNNEL_TOKEN)
```

**Pros**: 
- Free tier available
- Production-grade reliability
- Built-in DDoS protection

**Cons**:
- Requires Cloudflare account
- Some configuration complexity

##### Option B: LocalTunnel (Fully Open Source)
```javascript
// localtunnel-server.js
const localtunnel = require('localtunnel');

async function createTunnel(port, subdomain) {
    const tunnel = await localtunnel({
        port: port,
        subdomain: subdomain, // optional
        host: 'https://localtunnel.me'
    });
    
    console.log(`Tunnel URL: ${tunnel.url}`);
    return tunnel;
}
```

**Pros**:
- Completely free
- No account required
- Simple to use

**Cons**:
- Less reliable than NGrok
- Random URLs unless self-hosted

##### Option C: Bore (Rust-based, Self-Hosted)
```bash
# Server side (public VPS)
bore server --domain bore.example.com

# Client side (local cluster)
bore local 8080 --to bore.example.com
```

**Pros**:
- Self-hosted control
- Minimal resource usage
- Written in Rust (fast)

**Cons**:
- Requires public server
- Manual DNS configuration

##### Option D: FRP (Fast Reverse Proxy)
```ini
# frpc.ini (client config)
[common]
server_addr = your-server.com
server_port = 7000

[webhook]
type = http
local_port = 8080
subdomain = webhook
```

**Pros**:
- Highly configurable
- Supports multiple protocols
- Good performance

**Cons**:
- Requires public server
- More complex setup

#### Implementation Strategy
```go
// pkg/tunnel/provider.go
package tunnel

type TunnelProvider interface {
    CreateTunnel(port int, subdomain string) (string, error)
    CloseTunnel() error
}

type TunnelFactory struct {
    providerType string
}

func (f *TunnelFactory) GetProvider() TunnelProvider {
    switch f.providerType {
    case "cloudflare":
        return &CloudflareTunnelProvider{}
    case "localtunnel":
        return &LocalTunnelProvider{}
    case "bore":
        return &BoreProvider{}
    case "none":
        return &NoOpProvider{} // For production with real domains
    default:
        return &LocalTunnelProvider{} // Default fallback
    }
}
```

### 2. Twingate (Zero-Trust VPN)

#### Current Usage
- **Purpose**: Secure remote access to cluster services
- **Cost**: $5-10/user/month
- **Files**: `twingate-pastoral.yaml`, `twingate-therapeutic.yaml`

#### Open-Source Alternatives

##### Option A: WireGuard (Recommended)
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

**Pros**:
- Modern, fast protocol
- Minimal overhead
- Built into Linux kernel

**Cons**:
- Manual user management
- No built-in web UI

##### Option B: Tailscale (Freemium)
```bash
# Install Tailscale in cluster
kubectl apply -f https://raw.githubusercontent.com/tailscale/tailscale/main/cmd/k8s-operator/deploy/manifests/operator.yaml

# Expose service via Tailscale
kubectl annotate service my-service tailscale.com/expose="true"
```

**Pros**:
- Zero-config mesh VPN
- Free for personal use
- Excellent UX

**Cons**:
- Requires account
- Limited free tier

##### Option C: Headscale (Open-Source Tailscale)
```yaml
# headscale-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: headscale
spec:
  template:
    spec:
      containers:
      - name: headscale
        image: headscale/headscale:latest
        args:
        - serve
        volumeMounts:
        - name: config
          mountPath: /etc/headscale
        - name: data
          mountPath: /var/lib/headscale
```

**Pros**:
- Self-hosted Tailscale
- No account required
- Full control

**Cons**:
- More complex setup
- Requires maintenance

##### Option D: OpenVPN (Traditional)
```bash
# Generate OpenVPN server config
docker run -v $PWD:/etc/openvpn --rm kylemanna/openvpn ovpn_genconfig -u udp://vpn.example.com
docker run -v $PWD:/etc/openvpn --rm -it kylemanna/openvpn ovpn_initpki
```

**Pros**:
- Battle-tested
- Wide client support
- Many management tools

**Cons**:
- Older protocol
- More overhead
- Complex configuration

#### Implementation Strategy
```go
// pkg/access/vpn.go
package access

type VPNProvider interface {
    CreateUser(username string) (*UserCredentials, error)
    RevokeUser(username string) error
    GetConnectionConfig(username string) ([]byte, error)
}

type AccessManager struct {
    provider VPNProvider
}

func NewAccessManager(providerType string) *AccessManager {
    var provider VPNProvider
    
    switch providerType {
    case "wireguard":
        provider = &WireGuardProvider{}
    case "headscale":
        provider = &HeadscaleProvider{}
    case "none":
        provider = &KubectlPortForwardProvider{} // Fallback to port-forward
    default:
        provider = &KubectlPortForwardProvider{}
    }
    
    return &AccessManager{provider: provider}
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
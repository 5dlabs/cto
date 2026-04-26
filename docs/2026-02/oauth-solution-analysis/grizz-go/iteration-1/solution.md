# Grizz's Solution: Token Proxy Sidecar Pattern

## Philosophy

Don't distribute tokens to every service. Instead, proxy all Linear API calls through a single token-aware gateway that handles refresh transparently.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   Token Proxy Approach                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│   PM Server ─────┐                                           │
│                  │                                           │
│   Controller ────┼───▶  LINEAR-PROXY (single service)       │
│                  │      ┌────────────────────────────────┐  │
│   MCP Server ────┘      │ - Holds current access_token   │  │
│                         │ - Auto-refreshes before expiry │  │
│                         │ - Proxies all Linear API calls │  │
│                         │ - Single point of token mgmt   │  │
│                         └────────────────────────────────┘  │
│                                      │                       │
│                                      ▼                       │
│                            https://api.linear.app            │
└─────────────────────────────────────────────────────────────┘
```

## Implementation

### 1. Linear Proxy Service (Go)

```go
// cmd/linear-proxy/main.go
package main

import (
    "net/http"
    "net/http/httputil"
    "sync"
    "time"
)

type TokenManager struct {
    mu           sync.RWMutex
    accessToken  string
    refreshToken string
    expiresAt    time.Time
    clientID     string
    clientSecret string
}

func (tm *TokenManager) GetToken() (string, error) {
    tm.mu.RLock()
    // Check if refresh needed (1 hour buffer)
    needsRefresh := time.Now().Add(time.Hour).After(tm.expiresAt)
    tm.mu.RUnlock()
    
    if needsRefresh {
        if err := tm.refresh(); err != nil {
            return "", err
        }
    }
    
    tm.mu.RLock()
    defer tm.mu.RUnlock()
    return tm.accessToken, nil
}

func (tm *TokenManager) refresh() error {
    tm.mu.Lock()
    defer tm.mu.Unlock()
    
    // Double-check after acquiring write lock
    if time.Now().Add(time.Hour).Before(tm.expiresAt) {
        return nil
    }
    
    resp, err := http.PostForm("https://api.linear.app/oauth/token", url.Values{
        "grant_type":    {"refresh_token"},
        "refresh_token": {tm.refreshToken},
        "client_id":     {tm.clientID},
        "client_secret": {tm.clientSecret},
    })
    if err != nil {
        return err
    }
    defer resp.Body.Close()
    
    var tokenResp TokenResponse
    if err := json.NewDecoder(resp.Body).Decode(&tokenResp); err != nil {
        return err
    }
    
    tm.accessToken = tokenResp.AccessToken
    tm.refreshToken = tokenResp.RefreshToken
    tm.expiresAt = time.Now().Add(time.Duration(tokenResp.ExpiresIn) * time.Second)
    
    // Persist to storage (K8s secret, file, etc.)
    tm.persist()
    
    log.Info("Token refreshed", "expires_at", tm.expiresAt)
    return nil
}

func main() {
    tm := NewTokenManager()
    
    // Reverse proxy to Linear API
    proxy := &httputil.ReverseProxy{
        Director: func(req *http.Request) {
            token, _ := tm.GetToken()
            req.URL.Scheme = "https"
            req.URL.Host = "api.linear.app"
            req.Host = "api.linear.app"
            req.Header.Set("Authorization", "Bearer "+token)
        },
    }
    
    // Health endpoint
    http.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
        if _, err := tm.GetToken(); err != nil {
            http.Error(w, err.Error(), 500)
            return
        }
        w.Write([]byte(`{"status":"healthy"}`))
    })
    
    // Proxy all other requests
    http.Handle("/", proxy)
    
    log.Info("Linear proxy starting on :8082")
    http.ListenAndServe(":8082", nil)
}
```

### 2. Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: linear-proxy
  namespace: cto
spec:
  replicas: 2  # HA
  selector:
    matchLabels:
      app: linear-proxy
  template:
    metadata:
      labels:
        app: linear-proxy
    spec:
      containers:
      - name: proxy
        image: ghcr.io/5dlabs/linear-proxy:latest
        ports:
        - containerPort: 8082
        env:
        - name: LINEAR_CLIENT_ID
          valueFrom:
            secretKeyRef:
              name: linear-app-morgan
              key: client_id
        - name: LINEAR_CLIENT_SECRET
          valueFrom:
            secretKeyRef:
              name: linear-app-morgan
              key: client_secret
        - name: INITIAL_ACCESS_TOKEN
          valueFrom:
            secretKeyRef:
              name: linear-app-morgan
              key: access_token
        - name: INITIAL_REFRESH_TOKEN
          valueFrom:
            secretKeyRef:
              name: linear-app-morgan
              key: refresh_token
        volumeMounts:
        - name: token-state
          mountPath: /var/lib/linear-proxy
      volumes:
      - name: token-state
        persistentVolumeClaim:
          claimName: linear-proxy-state
---
apiVersion: v1
kind: Service
metadata:
  name: linear-proxy
  namespace: cto
spec:
  selector:
    app: linear-proxy
  ports:
  - port: 80
    targetPort: 8082
```

### 3. Client Configuration

Services just point to the proxy instead of Linear API:

```yaml
# PM Server config
env:
- name: LINEAR_API_URL
  value: "http://linear-proxy.cto.svc.cluster.local"
  # Instead of: https://api.linear.app
```

```rust
// In PM server
impl LinearClient {
    pub fn new(token: Option<&str>) -> Self {
        // If using proxy, no token needed - proxy adds it
        let base_url = env::var("LINEAR_API_URL")
            .unwrap_or_else(|_| "https://api.linear.app".to_string());
        
        Self { base_url, token: token.map(String::from) }
    }
}
```

### 4. Token State Persistence

```go
// Persist token to PVC for restart recovery
func (tm *TokenManager) persist() error {
    data := TokenState{
        AccessToken:  tm.accessToken,
        RefreshToken: tm.refreshToken,
        ExpiresAt:    tm.expiresAt.Unix(),
    }
    
    bytes, _ := json.Marshal(data)
    return os.WriteFile("/var/lib/linear-proxy/state.json", bytes, 0600)
}

// Load on startup
func (tm *TokenManager) loadState() error {
    bytes, err := os.ReadFile("/var/lib/linear-proxy/state.json")
    if os.IsNotExist(err) {
        return nil // Use initial tokens from env
    }
    if err != nil {
        return err
    }
    
    var state TokenState
    if err := json.Unmarshal(bytes, &state); err != nil {
        return err
    }
    
    // Only use if not expired
    if time.Unix(state.ExpiresAt, 0).After(time.Now()) {
        tm.accessToken = state.AccessToken
        tm.refreshToken = state.RefreshToken
        tm.expiresAt = time.Unix(state.ExpiresAt, 0)
    }
    return nil
}
```

## Answers to Requirements

### 1. Where are tokens stored?
- Initial tokens in K8s Secret (bootstrap)
- Runtime tokens in proxy's memory + PVC (for restart)
- Single source of truth: the proxy itself

### 2. What triggers a refresh?
Every request checks token validity. Refreshes proactively if expiring within 1 hour.

### 3. How do services get the new token?
They don't! Services call proxy, proxy adds token. Services never see the token.

### 4. What happens if refresh fails?
- Proxy returns 503 to callers
- Prometheus metric for alerts
- PVC preserves last good state for debugging
- If refresh_token revoked, need manual re-auth (but this is rare)

### 5. How is this deployed/maintained?
- Single deployment with HA (2 replicas)
- PVC for state persistence
- Helm chart in `infra/gitops/apps/linear-proxy/`

## Local Development Story

Run the proxy locally:

```bash
# Start proxy
LINEAR_CLIENT_ID="..." \
LINEAR_CLIENT_SECRET="..." \
INITIAL_ACCESS_TOKEN="..." \
INITIAL_REFRESH_TOKEN="..." \
./linear-proxy

# Configure services to use it
export LINEAR_API_URL="http://localhost:8082"
```

Or add to `process-compose.yaml`:

```yaml
processes:
  linear-proxy:
    command: ./target/release/linear-proxy
    environment:
      LINEAR_CLIENT_ID: ${LINEAR_CLIENT_ID}
      # ... etc
```

## Pros

- **Zero token distribution** - services never see tokens
- **Instant refresh** - no pod restarts needed
- **Single point of management** - one place to debug
- **Transparent to services** - just change URL
- **Built-in HA** - multiple replicas share state via PVC

## Cons

- Additional network hop for all Linear calls
- Single point of failure (mitigated by HA)
- PVC required for state persistence
- More complex local dev setup
- If proxy goes down, all Linear calls fail

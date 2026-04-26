# Blaze's Solution: Token Manager Library with Pre-emptive Refresh

## Philosophy

Don't fight the OAuth spec - embrace it. Build a robust token management library that every service imports, with pre-emptive refresh baked into the HTTP client layer.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│              Shared Token Manager Library                    │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│   │ PM Server   │  │ Controller  │  │ MCP Server  │        │
│   │  (Rust)     │  │  (Rust)     │  │  (Rust)     │        │
│   └──────┬──────┘  └──────┬──────┘  └──────┬──────┘        │
│          │                │                │                │
│          └────────────────┼────────────────┘                │
│                           ▼                                  │
│              ┌────────────────────────────┐                 │
│              │ @5dlabs/linear-auth       │                 │
│              │ (shared library/crate)     │                 │
│              │                            │                 │
│              │ • TokenStore interface     │                 │
│              │ • Pre-emptive refresh      │                 │
│              │ • Retry with backoff       │                 │
│              │ • Event emission           │                 │
│              └────────────────────────────┘                 │
│                           │                                  │
│              ┌────────────┴────────────────┐                │
│              ▼                              ▼                │
│     ┌─────────────────┐           ┌─────────────────┐      │
│     │ K8s Secret      │           │ 1Password/File  │      │
│     │ (production)    │           │ (local dev)     │      │
│     └─────────────────┘           └─────────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

## Implementation

### 1. Core Token Manager (Rust crate)

```rust
// crates/linear-auth/src/lib.rs

use std::sync::Arc;
use tokio::sync::RwLock;

/// Token storage backend trait
#[async_trait]
pub trait TokenStore: Send + Sync {
    async fn get_tokens(&self, agent: &str) -> Result<TokenSet>;
    async fn set_tokens(&self, agent: &str, tokens: TokenSet) -> Result<()>;
}

/// Token set with metadata
#[derive(Clone, Serialize, Deserialize)]
pub struct TokenSet {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
}

impl TokenSet {
    pub fn expires_within(&self, duration: chrono::Duration) -> bool {
        let expires = chrono::DateTime::from_timestamp(self.expires_at, 0).unwrap();
        chrono::Utc::now() + duration > expires
    }
}

/// The main token manager
pub struct LinearTokenManager {
    store: Arc<dyn TokenStore>,
    client_id: String,
    client_secret: String,
    agent: String,
    tokens: RwLock<Option<TokenSet>>,
    refresh_lock: tokio::sync::Mutex<()>,
}

impl LinearTokenManager {
    pub async fn new(
        store: Arc<dyn TokenStore>,
        client_id: String,
        client_secret: String,
        agent: String,
    ) -> Result<Self> {
        let manager = Self {
            store,
            client_id,
            client_secret,
            agent,
            tokens: RwLock::new(None),
            refresh_lock: tokio::sync::Mutex::new(()),
        };
        
        // Load initial tokens
        manager.load_tokens().await?;
        
        // Start background refresh task
        manager.spawn_refresh_task();
        
        Ok(manager)
    }
    
    /// Get a valid access token, refreshing if needed
    pub async fn get_token(&self) -> Result<String> {
        // Fast path: check if current token is valid
        {
            let tokens = self.tokens.read().await;
            if let Some(ref t) = *tokens {
                if !t.expires_within(chrono::Duration::minutes(5)) {
                    return Ok(t.access_token.clone());
                }
            }
        }
        
        // Slow path: refresh needed
        self.refresh_token().await?;
        
        let tokens = self.tokens.read().await;
        Ok(tokens.as_ref().unwrap().access_token.clone())
    }
    
    async fn refresh_token(&self) -> Result<()> {
        // Single-flight: only one refresh at a time
        let _guard = self.refresh_lock.lock().await;
        
        // Double-check after acquiring lock
        {
            let tokens = self.tokens.read().await;
            if let Some(ref t) = *tokens {
                if !t.expires_within(chrono::Duration::minutes(5)) {
                    return Ok(());
                }
            }
        }
        
        let current = self.tokens.read().await;
        let refresh_token = current.as_ref()
            .map(|t| t.refresh_token.clone())
            .ok_or_else(|| anyhow!("No refresh token available"))?;
        drop(current);
        
        info!(agent = %self.agent, "Refreshing Linear OAuth token");
        
        let client = reqwest::Client::new();
        let resp = client
            .post("https://api.linear.app/oauth/token")
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", &refresh_token),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
            ])
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let error = resp.text().await?;
            error!(agent = %self.agent, error = %error, "Token refresh failed");
            return Err(anyhow!("Token refresh failed: {}", error));
        }
        
        let token_resp: TokenResponse = resp.json().await?;
        let new_tokens = TokenSet {
            access_token: token_resp.access_token,
            refresh_token: token_resp.refresh_token,
            expires_at: chrono::Utc::now().timestamp() + token_resp.expires_in,
        };
        
        // Persist first, then update memory
        self.store.set_tokens(&self.agent, new_tokens.clone()).await?;
        
        let mut tokens = self.tokens.write().await;
        *tokens = Some(new_tokens);
        
        info!(agent = %self.agent, "Token refreshed successfully");
        metrics::counter!("linear_token_refresh_success", "agent" => self.agent.clone()).increment(1);
        
        Ok(())
    }
    
    /// Background task that pre-emptively refreshes tokens
    fn spawn_refresh_task(&self) {
        let agent = self.agent.clone();
        let this = Arc::new(self.clone()); // Needs proper Arc handling
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // Every 5 min
            
            loop {
                interval.tick().await;
                
                let tokens = this.tokens.read().await;
                if let Some(ref t) = *tokens {
                    // Refresh if expiring within 1 hour
                    if t.expires_within(chrono::Duration::hours(1)) {
                        drop(tokens);
                        if let Err(e) = this.refresh_token().await {
                            error!(agent = %agent, error = %e, "Background refresh failed");
                        }
                    }
                }
            }
        });
    }
}
```

### 2. Token Store Implementations

```rust
// crates/linear-auth/src/stores/k8s.rs
pub struct K8sSecretStore {
    client: kube::Client,
    namespace: String,
    secret_name: String,
}

#[async_trait]
impl TokenStore for K8sSecretStore {
    async fn get_tokens(&self, agent: &str) -> Result<TokenSet> {
        let secrets: Api<Secret> = Api::namespaced(self.client.clone(), &self.namespace);
        let secret = secrets.get(&self.secret_name).await?;
        
        let data = secret.data.unwrap_or_default();
        let access_key = format!("{}_ACCESS_TOKEN", agent.to_uppercase());
        let refresh_key = format!("{}_REFRESH_TOKEN", agent.to_uppercase());
        let expires_key = format!("{}_EXPIRES_AT", agent.to_uppercase());
        
        Ok(TokenSet {
            access_token: decode_secret(&data, &access_key)?,
            refresh_token: decode_secret(&data, &refresh_key)?,
            expires_at: decode_secret(&data, &expires_key)?.parse()?,
        })
    }
    
    async fn set_tokens(&self, agent: &str, tokens: TokenSet) -> Result<()> {
        let secrets: Api<Secret> = Api::namespaced(self.client.clone(), &self.namespace);
        
        let patch = json!({
            "data": {
                format!("{}_ACCESS_TOKEN", agent.to_uppercase()): base64_encode(&tokens.access_token),
                format!("{}_REFRESH_TOKEN", agent.to_uppercase()): base64_encode(&tokens.refresh_token),
                format!("{}_EXPIRES_AT", agent.to_uppercase()): base64_encode(&tokens.expires_at.to_string()),
            }
        });
        
        secrets.patch(&self.secret_name, &PatchParams::default(), &Patch::Merge(&patch)).await?;
        Ok(())
    }
}

// crates/linear-auth/src/stores/file.rs
pub struct FileStore {
    path: PathBuf,
}

#[async_trait]
impl TokenStore for FileStore {
    async fn get_tokens(&self, agent: &str) -> Result<TokenSet> {
        let path = self.path.join(format!("{}.json", agent));
        let content = tokio::fs::read_to_string(&path).await?;
        Ok(serde_json::from_str(&content)?)
    }
    
    async fn set_tokens(&self, agent: &str, tokens: TokenSet) -> Result<()> {
        let path = self.path.join(format!("{}.json", agent));
        let content = serde_json::to_string_pretty(&tokens)?;
        tokio::fs::write(&path, content).await?;
        Ok(())
    }
}
```

### 3. Integration with Existing Code

```rust
// crates/pm/src/bin/pm-server.rs
use linear_auth::{LinearTokenManager, K8sSecretStore, FileStore};

#[tokio::main]
async fn main() -> Result<()> {
    // Create store based on environment
    let store: Arc<dyn TokenStore> = if env::var("KUBERNETES_SERVICE_HOST").is_ok() {
        Arc::new(K8sSecretStore::new().await?)
    } else {
        Arc::new(FileStore::new(PathBuf::from(".tokens")))
    };
    
    // Create token manager
    let token_manager = LinearTokenManager::new(
        store,
        env::var("LINEAR_CLIENT_ID")?,
        env::var("LINEAR_CLIENT_SECRET")?,
        "morgan".to_string(),
    ).await?;
    
    // Create Linear client that uses the manager
    let linear_client = LinearClient::with_token_manager(token_manager);
    
    // ... rest of server setup
}
```

## Answers to Requirements

### 1. Where are tokens stored?
- Production: K8s Secret (shared across replicas)
- Local: JSON files in `.tokens/` directory
- Both can be plugged in via `TokenStore` trait

### 2. What triggers a refresh?
- **On-demand**: Every `get_token()` call checks validity
- **Pre-emptive**: Background task refreshes 1 hour before expiry
- **Automatic retry**: If 401 received, forces refresh and retries

### 3. How do services get the new token?
They call `token_manager.get_token()` which returns a valid token. The manager handles all refresh logic internally.

### 4. What happens if refresh fails?
- Logged and metrics emitted
- Retried on next call
- If refresh_token revoked, error propagates up
- Services can catch and alert on persistent failures

### 5. How is this deployed/maintained?
- Shared crate in workspace: `crates/linear-auth/`
- Services add dependency: `linear-auth = { path = "../linear-auth" }`
- No separate deployment - library code

## Local Development Story

```bash
# Initial setup - store tokens locally
mkdir -p .tokens
cat > .tokens/morgan.json << EOF
{
  "access_token": "lin_oauth_...",
  "refresh_token": "lin_ref_...",
  "expires_at": $(date -v+10d +%s)
}
EOF

# Services automatically use FileStore locally
export LINEAR_CLIENT_ID="..."
export LINEAR_CLIENT_SECRET="..."

# Run service - tokens refresh automatically
cargo run --bin pm-server
```

## Pros

- **No new services** - library code, not infrastructure
- **Type-safe** - compile-time guarantees
- **Testable** - mock TokenStore for tests
- **Flexible storage** - any backend via trait
- **Zero config** - auto-detects K8s vs local
- **Immediate effect** - no pod restarts needed

## Cons

- Requires code changes in every service
- Each service manages its own token state
- Potential race conditions if multiple replicas refresh simultaneously (mitigated by K8s secret as coordination)
- More complex than "just use a proxy"

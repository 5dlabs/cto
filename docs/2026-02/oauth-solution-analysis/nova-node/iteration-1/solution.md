# Nova's Solution: Event-Driven Token Orchestrator with Linear App Webhook

## Philosophy

OAuth refresh shouldn't be a cron job - it should be reactive. Linear can tell us when tokens are about to expire via app webhooks. Build an event-driven system that responds to signals rather than polling.

## Key Insight

Linear OAuth apps can receive webhooks. We can use the Linear API to check token health and Linear's built-in notification system to trigger proactive refreshes.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│              Event-Driven Token Orchestrator                 │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│   Linear API ──────────────────────────────────────────┐    │
│   (App Events)                                          │    │
│                                                         ▼    │
│                                    ┌─────────────────────┐  │
│   PM Server API Call ─────────┐    │ Token Orchestrator  │  │
│   (401 error)                 │    │                     │  │
│                               │    │ Events:             │  │
│   Scheduled Check ────────────┼───▶│ • token.expiring    │  │
│   (daily health check)        │    │ • token.expired     │  │
│                               │    │ • token.refreshed   │  │
│   Manual Trigger ─────────────┘    │ • token.error       │  │
│   (emergency refresh)              │                     │  │
│                                    │ Actions:            │  │
│                                    │ • Refresh token     │  │
│                                    │ • Update secrets    │  │
│                                    │ • Notify services   │  │
│                                    │ • Alert on failure  │  │
│                                    └─────────────────────┘  │
│                                             │                │
│                                             ▼                │
│                    ┌────────────────────────────────────┐   │
│                    │ NATS / Redis Pub/Sub               │   │
│                    │ Channel: linear.tokens.{agent}     │   │
│                    └────────────────────────────────────┘   │
│                              │                               │
│              ┌───────────────┼───────────────┐              │
│              ▼               ▼               ▼              │
│        ┌──────────┐   ┌──────────┐   ┌──────────┐         │
│        │PM Server │   │Controller│   │MCP Server│         │
│        │(listens) │   │(listens) │   │(listens) │         │
│        └──────────┘   └──────────┘   └──────────┘         │
└─────────────────────────────────────────────────────────────┘
```

## Implementation

### 1. Token Orchestrator Service (Effect-TS)

```typescript
// services/token-orchestrator/src/index.ts
import { Effect, Schedule, Stream, Queue } from "effect"
import { Redis } from "@effect/redis"

// Events
type TokenEvent = 
  | { type: "token.check"; agent: string }
  | { type: "token.expiring"; agent: string; expiresAt: Date }
  | { type: "token.refresh"; agent: string }
  | { type: "token.refreshed"; agent: string; expiresAt: Date }
  | { type: "token.error"; agent: string; error: string }

// Token state
interface TokenState {
  accessToken: string
  refreshToken: string
  expiresAt: number
  lastRefresh: number
}

// Main orchestrator
const TokenOrchestrator = Effect.gen(function* () {
  const redis = yield* Redis
  const eventQueue = yield* Queue.unbounded<TokenEvent>()
  
  // Event processor
  const processEvents = Stream.fromQueue(eventQueue).pipe(
    Stream.tap((event) => 
      Effect.logInfo(`Processing event: ${event.type}`, { agent: event.agent })
    ),
    Stream.mapEffect((event) => handleEvent(event)),
    Stream.runDrain
  )
  
  // Scheduled health check (every 6 hours)
  const healthCheck = Effect.gen(function* () {
    const agents = ["morgan", "atlas", "rex", "blaze"]
    
    for (const agent of agents) {
      yield* Queue.offer(eventQueue, { type: "token.check", agent })
    }
  }).pipe(
    Effect.schedule(Schedule.fixed("6 hours"))
  )
  
  // HTTP server for webhooks and manual triggers
  const server = yield* createServer({
    // Linear webhook endpoint (if Linear supports app webhooks)
    "POST /webhook/linear": (req) => Effect.gen(function* () {
      const body = yield* req.json()
      
      if (body.type === "app.token_expiring") {
        yield* Queue.offer(eventQueue, { 
          type: "token.expiring", 
          agent: body.app_id,
          expiresAt: new Date(body.expires_at)
        })
      }
      
      return Response.ok()
    }),
    
    // Manual refresh trigger
    "POST /refresh/:agent": (req) => Effect.gen(function* () {
      const agent = req.params.agent
      yield* Queue.offer(eventQueue, { type: "token.refresh", agent })
      return Response.json({ queued: true })
    }),
    
    // Health endpoint
    "GET /health": () => Effect.succeed(Response.json({ status: "healthy" }))
  })
  
  // Run all
  yield* Effect.all([processEvents, healthCheck, server], { concurrency: "unbounded" })
})

// Event handler
const handleEvent = (event: TokenEvent) => Effect.gen(function* () {
  const redis = yield* Redis
  
  switch (event.type) {
    case "token.check": {
      const state = yield* getTokenState(event.agent)
      if (!state) return
      
      const hoursUntilExpiry = (state.expiresAt - Date.now()) / (1000 * 60 * 60)
      
      if (hoursUntilExpiry < 24) {
        yield* Effect.logWarning(`Token expiring soon`, { agent: event.agent, hours: hoursUntilExpiry })
        return yield* handleEvent({ type: "token.refresh", agent: event.agent })
      }
      
      yield* Effect.logInfo(`Token healthy`, { agent: event.agent, hours: hoursUntilExpiry })
      break
    }
    
    case "token.expiring":
    case "token.refresh": {
      yield* Effect.logInfo(`Refreshing token`, { agent: event.agent })
      
      const result = yield* refreshToken(event.agent).pipe(
        Effect.either
      )
      
      if (result._tag === "Left") {
        yield* Effect.logError(`Refresh failed`, { agent: event.agent, error: result.left })
        yield* publishEvent({ type: "token.error", agent: event.agent, error: String(result.left) })
        yield* sendAlert(event.agent, result.left)
      } else {
        const newState = result.right
        yield* publishEvent({ 
          type: "token.refreshed", 
          agent: event.agent, 
          expiresAt: new Date(newState.expiresAt) 
        })
      }
      break
    }
  }
})

// Token refresh logic
const refreshToken = (agent: string) => Effect.gen(function* () {
  const state = yield* getTokenState(agent)
  const config = yield* getAgentConfig(agent)
  
  const response = yield* Effect.tryPromise(() =>
    fetch("https://api.linear.app/oauth/token", {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: new URLSearchParams({
        grant_type: "refresh_token",
        refresh_token: state.refreshToken,
        client_id: config.clientId,
        client_secret: config.clientSecret,
      })
    })
  )
  
  if (!response.ok) {
    const error = yield* Effect.tryPromise(() => response.text())
    return yield* Effect.fail(new Error(`Refresh failed: ${error}`))
  }
  
  const tokenResponse = yield* Effect.tryPromise(() => response.json())
  
  const newState: TokenState = {
    accessToken: tokenResponse.access_token,
    refreshToken: tokenResponse.refresh_token,
    expiresAt: Date.now() + (tokenResponse.expires_in * 1000),
    lastRefresh: Date.now()
  }
  
  // Update K8s secret
  yield* updateK8sSecret(agent, newState)
  
  // Store in Redis for quick access
  yield* Redis.set(`token:${agent}`, JSON.stringify(newState))
  
  // Publish update to all listeners
  yield* Redis.publish(`linear.tokens.${agent}`, JSON.stringify({
    type: "token.updated",
    expiresAt: newState.expiresAt
  }))
  
  yield* Effect.logInfo(`Token refreshed`, { agent, expiresAt: new Date(newState.expiresAt) })
  
  return newState
})

// Publish event for service consumption
const publishEvent = (event: TokenEvent) => Effect.gen(function* () {
  const redis = yield* Redis
  yield* Redis.publish(`linear.tokens.events`, JSON.stringify(event))
})
```

### 2. Service Integration (Token Subscriber)

```typescript
// packages/linear-auth/src/subscriber.ts
import { Effect, Stream } from "effect"
import { Redis } from "@effect/redis"

export class TokenSubscriber {
  private currentToken: string | null = null
  private agent: string
  
  constructor(agent: string) {
    this.agent = agent
  }
  
  async start() {
    // Subscribe to token updates
    const redis = await Redis.connect()
    
    // Initial fetch
    const state = await redis.get(`token:${this.agent}`)
    if (state) {
      this.currentToken = JSON.parse(state).accessToken
    }
    
    // Listen for updates
    const subscriber = redis.subscribe(`linear.tokens.${this.agent}`)
    subscriber.on("message", (channel, message) => {
      const event = JSON.parse(message)
      if (event.type === "token.updated") {
        // Fetch new token
        redis.get(`token:${this.agent}`).then((state) => {
          if (state) {
            this.currentToken = JSON.parse(state).accessToken
            console.log(`Token updated for ${this.agent}`)
          }
        })
      }
    })
  }
  
  getToken(): string | null {
    return this.currentToken
  }
}
```

### 3. Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: token-orchestrator
  namespace: cto
spec:
  replicas: 1  # Single instance (event-driven, not load-balanced)
  selector:
    matchLabels:
      app: token-orchestrator
  template:
    spec:
      containers:
      - name: orchestrator
        image: ghcr.io/5dlabs/token-orchestrator:latest
        ports:
        - containerPort: 3000
        env:
        - name: REDIS_URL
          value: "redis://redis.cto.svc.cluster.local:6379"
        - name: LINEAR_MORGAN_CLIENT_ID
          valueFrom:
            secretKeyRef:
              name: linear-app-morgan
              key: client_id
        # ... more env vars
---
apiVersion: v1
kind: Service
metadata:
  name: token-orchestrator
  namespace: cto
spec:
  selector:
    app: token-orchestrator
  ports:
  - port: 80
    targetPort: 3000
```

### 4. Integration with PM Server (Rust)

```rust
// crates/pm/src/token_subscriber.rs
use redis::AsyncCommands;

pub struct TokenSubscriber {
    redis: redis::aio::MultiplexedConnection,
    agent: String,
    current_token: Arc<RwLock<Option<String>>>,
}

impl TokenSubscriber {
    pub async fn new(redis_url: &str, agent: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let mut conn = client.get_multiplexed_async_connection().await?;
        
        // Fetch initial token
        let state: Option<String> = conn.get(format!("token:{}", agent)).await?;
        let current_token = state.map(|s| {
            let parsed: serde_json::Value = serde_json::from_str(&s).unwrap();
            parsed["accessToken"].as_str().unwrap().to_string()
        });
        
        let subscriber = Self {
            redis: conn,
            agent: agent.to_string(),
            current_token: Arc::new(RwLock::new(current_token)),
        };
        
        // Start subscription in background
        subscriber.start_subscription();
        
        Ok(subscriber)
    }
    
    fn start_subscription(&self) {
        let token = self.current_token.clone();
        let agent = self.agent.clone();
        
        tokio::spawn(async move {
            let client = redis::Client::open("redis://localhost").unwrap();
            let mut pubsub = client.get_async_pubsub().await.unwrap();
            pubsub.subscribe(format!("linear.tokens.{}", agent)).await.unwrap();
            
            loop {
                let msg = pubsub.on_message().next().await;
                if let Some(msg) = msg {
                    // Refetch token on update notification
                    // ...
                }
            }
        });
    }
    
    pub async fn get_token(&self) -> Option<String> {
        self.current_token.read().await.clone()
    }
}
```

## Answers to Requirements

### 1. Where are tokens stored?
- Primary: K8s Secrets (source of truth)
- Cache: Redis (for quick pub/sub distribution)
- Orchestrator manages both

### 2. What triggers a refresh?
- Scheduled health check (every 6 hours)
- Manual trigger (`POST /refresh/:agent`)
- 401 error detection (services can POST to orchestrator)
- Potentially Linear webhooks (if supported)

### 3. How do services get the new token?
- Subscribe to Redis pub/sub channel
- Get notification when token changes
- Fetch new token from Redis cache
- **No restart required**

### 4. What happens if refresh fails?
- Event published: `token.error`
- Alert sent (Slack/PagerDuty)
- Services continue using old token until it expires
- Logged for debugging

### 5. How is this deployed/maintained?
- Single Token Orchestrator deployment
- Redis for pub/sub (can use existing cluster Redis)
- Services add small subscriber library

## Local Development Story

```bash
# Start Redis locally
docker run -d --name redis -p 6379:6379 redis

# Start token orchestrator
cd services/token-orchestrator
pnpm dev

# Manually trigger refresh
curl -X POST http://localhost:3000/refresh/morgan

# Services connect to Redis
export REDIS_URL="redis://localhost:6379"
cargo run --bin pm-server
```

## Pros

- **Reactive** - responds to events, not polling
- **Real-time** - services get updates immediately via pub/sub
- **Observable** - all events are trackable
- **No service restarts** - tokens update in-place
- **Centralized logic** - one place for all refresh logic
- **Extensible** - easy to add new agents or triggers

## Cons

- Requires Redis (additional infrastructure)
- More complex than simple cron job
- Single orchestrator = potential single point of failure
- Services need subscriber integration
- Effect-TS may be unfamiliar

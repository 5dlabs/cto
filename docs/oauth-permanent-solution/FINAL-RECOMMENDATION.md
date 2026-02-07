# Linear OAuth Permanent Solution - Final Recommendation

**Consensus from 4-Agent Analysis (Rex, Grizz, Nova, Blaze)**

---

## The Smoking Gun

All 4 agents independently identified the same root cause:

```rust
// server.rs line 88
#[allow(dead_code)]  // <-- THE PROBLEM
async fn get_agent_client_with_refresh(...)
```

**The fix already exists but is never called.** This is a 4-line change.

---

## Unanimous Agreement: Quick Wins (Do TODAY)

| Fix | Location | Time | Impact |
|-----|----------|------|--------|
| Remove `actor=app` from OAuth URL | `config.rs:280` | 5 min | Enables issue creation |
| Remove `#[allow(dead_code)]` | `server.rs:88` | 1 min | Enables auto-refresh |
| Replace `get_agent_client` → `get_agent_client_with_refresh` | `server.rs` (4 places) | 30 min | Uses the refresh logic |
| Increase expiry buffer from 5min to 1hr | `config.rs:119` | 5 min | Prevents mid-workflow expiration |
| Re-authorize all 13 agents with new OAuth URL | Browser | 30 min | Gets user-actor tokens |

**Total: ~1 hour to fix 80% of the problem**

---

## Key Discoveries from Cross-Critique

### What Rex Identified
- Dead code is the primary issue
- Token Health Manager with semaphore-based refresh limiting
- Proactive refresh at 1 hour before expiry

### What Grizz Identified
- Distributed state drift problem (Memory vs K8s vs ExternalSecrets)
- Need for leader election in multi-replica scenarios
- **Critical gap**: Initial authorization flow - nobody can avoid the first browser OAuth

### What Nova Identified
- **Atomic refresh token persistence** - must save new refresh token BEFORE anything else
- Linear rotates refresh tokens; a failed write = permanent lockout
- 401 vs 403 semantics matter for retry logic

### What Blaze Identified
- **UX is invisible in all technical solutions** - what does the user SEE when auth fails?
- Need dashboard for "single pane of glass" monitoring
- Alert hierarchy: Info → Warning → Critical → Emergency
- Graceful degradation messages in Linear comments

---

## Consensus Architecture

```
┌──────────────────────────────────────────────────────────────────────┐
│                         PM SERVER (Enhanced)                          │
├──────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌─────────────────┐   ┌─────────────────┐   ┌──────────────────┐   │
│  │ Token Health    │   │ Linear Client   │   │ OAuth Handlers   │   │
│  │ Manager (NEW)   │   │ (w/ interceptor)│   │ (existing)       │   │
│  │                 │   │                 │   │                  │   │
│  │ - Background    │   │ - Auto-401      │   │ - /oauth/start   │   │
│  │   refresh task  │   │   retry         │   │ - /callback      │   │
│  │ - Proactive at  │   │ - Atomic token  │   │ - /refresh       │   │
│  │   20% TTL left  │   │   persistence   │   │                  │   │
│  │ - Semaphore     │   │                 │   │                  │   │
│  │   limiting      │   │                 │   │                  │   │
│  └────────┬────────┘   └────────┬────────┘   └────────┬─────────┘   │
│           │                     │                      │             │
│           └─────────────────────┼──────────────────────┘             │
│                                 │                                    │
│                     ┌───────────▼───────────┐                        │
│                     │  K8s Secret Watch     │                        │
│                     │  (Single Source of    │                        │
│                     │   Truth)              │                        │
│                     └───────────────────────┘                        │
│                                 │                                    │
│                     ┌───────────▼───────────┐                        │
│                     │  Prometheus Metrics   │◄── Alertmanager        │
│                     │  + /health/tokens     │                        │
│                     └───────────────────────┘                        │
│                                                                       │
└──────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Timeline

### Phase 1: Immediate Fixes (Day 1) - CRITICAL

```bash
# 1. Fix OAuth URL (remove actor=app)
sed -i 's/&actor=app//g' crates/pm/src/config.rs

# 2. Enable refresh function
sed -i 's/#\[allow(dead_code)\]//' crates/pm/src/server.rs

# 3. Update call sites (4 places in server.rs)
# Change: let client = get_agent_client(&state.config, agent_name);
# To:     let client = get_agent_client_with_refresh(&state.config, &state.kube_client, agent_name).await;

# 4. Increase buffer (config.rs:119)
# Change: expires_at - now < 300
# To:     expires_at - now < 3600

# 5. Re-authorize all agents
for agent in morgan rex blaze grizz nova tap spark cleo cipher tess atlas bolt vex; do
  echo "Visit: https://pm.5dlabs.ai/oauth/start?agent=$agent"
done
```

### Phase 2: Token Health Manager (Days 2-4)

From Rex's analysis, implement as a background Tokio task:
- Sweep all tokens every 5 minutes
- Refresh when < 20% lifetime remaining (2 days for 10-day token)
- Semaphore to limit concurrent refreshes
- Exponential backoff on failures

Add Nova's **atomic persistence** fix:
```rust
// Always persist refresh token FIRST
async fn refresh_with_atomic_persistence(...) {
    let response = linear_refresh(...).await?;
    
    // CRITICAL: Save refresh token immediately, before anything else
    store_refresh_token_only(&response.refresh_token).await?;
    
    // Now safe to update access token and other state
    store_access_token(&response.access_token).await?;
    update_in_memory_config(...);
}
```

### Phase 3: Observability (Days 5-7)

From all analyses:
```yaml
# Prometheus metrics
linear_oauth_token_expiry_timestamp{agent="morgan"}
linear_oauth_refresh_total{agent="morgan",result="success|failure"}
linear_oauth_health{agent="morgan"} # 1=healthy, 0=unhealthy

# Alert rules
- alert: LinearTokenExpiringSoon
  expr: (linear_oauth_token_expiry_timestamp - time()) < 86400
  severity: warning

- alert: LinearTokenRefreshFailing
  expr: increase(linear_oauth_refresh_total{result="failure"}[1h]) > 3
  severity: critical
```

From Blaze: `/health/tokens` endpoint for dashboard integration.

### Phase 4: Dashboard & UX (Week 2+)

From Blaze's mockups:
- Token health overview (green/yellow/red per agent)
- TTL timeline visualization
- Manual refresh buttons
- Activity log

Defer the full Grizz K8s Operator - it's overengineered for the current scale.

---

## What NOT to Do

| Rejected Approach | Why |
|-------------------|-----|
| K8s Operator (CRD) | 5-week timeline, Go in Rust codebase, overkill for 13 agents |
| CronJob for refresh | External process, harder to debug, race conditions |
| Personal API keys | No refresh, less secure, loses agent attribution |
| Single service account | Single point of failure, loses audit trail |

---

## Success Criteria

From all 4 analyses:

| Metric | Target |
|--------|--------|
| Manual re-authorizations | 0 per quarter (normal operation) |
| Auto-refresh success rate | > 99% |
| Alert lead time | > 48 hours before expiration |
| MTTR (when manual intervention needed) | < 30 minutes |

---

## The One Thing Everyone Agrees On

> "The goal isn't 'refresh tokens automatically' - it's **make OAuth invisible**."

If the operator ever thinks about OAuth, we've failed.

---

## Action Items

### Immediate (Today)
- [ ] Apply 4-line code fix (remove dead_code, fix actor=app, increase buffer)
- [ ] Build and deploy updated PM server
- [ ] Re-authorize all 13 agents with corrected OAuth flow

### This Week
- [ ] Implement TokenHealthManager background task
- [ ] Add atomic refresh token persistence
- [ ] Add /health/tokens endpoint

### Next Week
- [ ] Add Prometheus metrics
- [ ] Configure alerting rules
- [ ] Create Grafana dashboard

### Future
- [ ] Dashboard UI (Blaze designs)
- [ ] CLI tool for initial authorization
- [ ] Runbook documentation

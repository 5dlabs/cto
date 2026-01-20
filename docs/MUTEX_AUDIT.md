# Mutex/RwLock Usage Audit

Generated: 2026-01-19  
Total references: 107

## Summary

The codebase correctly uses the appropriate mutex types for different contexts:

- **Async contexts**: `tokio::sync::Mutex` / `tokio::sync::RwLock` (safe across await points)
- **Sync contexts**: `std::sync::Mutex` / `std::sync::RwLock` (only in non-async code)

## tokio::sync Usage (Async-Safe) - 18 usages

All async code correctly uses tokio's async-aware locks:

| File | Type | Purpose |
|------|------|---------|
| tools/server/http_server.rs | RwLock, Mutex | HTTP server state |
| tools/recovery.rs | Mutex, RwLock | Recovery state |
| tools/health_monitor.rs | Mutex, RwLock | Health monitoring |
| tools/client.rs | Mutex | Client state |
| pm/tests/input_routing_tests.rs | RwLock | Test fixtures |
| pm/state/session_tracker.rs | RwLock | Session tracking |
| pm/handlers/agent_comms.rs | RwLock | Agent communication |
| healer/play/session.rs | RwLock | Play session state |
| healer/platform/workflow.rs | RwLock | Workflow tracking |
| healer/platform/alerts.rs | RwLock | Alert deduplication |
| healer/ci/tracker.rs | RwLock | CI failure tracking |
| healer/ci/server.rs | RwLock | CI server state |
| controller/tasks/security/rate_limit.rs | RwLock | Rate limiting |
| controller/tasks/security/audit.rs | RwLock | Security audit logs |
| controller/tasks/label/concurrent.rs | Mutex | Concurrent label ops |
| controller/cli/session.rs | RwLock | CLI session state |
| controller/cli/adapter_factory.rs | RwLock | Adapter caching |

## std::sync Usage (Sync-Only) - 3 usages

All sync code correctly uses standard library locks (no await points):

| File | Type | Context | Safe? |
|------|------|---------|-------|
| pm/src/config.rs | Mutex | Test serialization (`#[cfg(test)]`) | ✅ Yes |
| intake/src/ai/registry.rs | RwLock | Provider registry (sync methods only) | ✅ Yes |
| cost/src/tracking/tracker.rs | RwLock | Cost tracking (sync methods only) | ✅ Yes |

## Verification

Checked all `std::sync` usages for potential issues:

1. **pm/config.rs**: `ENV_MUTEX` is only used in `#[cfg(test)]` to serialize environment variable tests. No async code.

2. **intake/registry.rs**: `ProviderRegistry` methods (`register`, `get`, `get_for_model`) are all synchronous. Lock guards are dropped before any potential async operations.

3. **cost/tracker.rs**: `CostTracker` methods (`record`, `all_calls`, `calls_for_*`) are all synchronous. Used for in-memory cost aggregation.

## Conclusion

No unsafe lock-across-await patterns found. The codebase follows best practices:
- Async code uses `tokio::sync` locks
- Sync code uses `std::sync` locks
- No `std::sync` locks are held across `.await` points

**Status: PASSED - No action required**

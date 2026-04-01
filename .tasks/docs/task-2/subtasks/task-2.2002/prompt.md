Implement subtask 2002: Add per-pipeline-run caching layer to resolve_agent_delegates()

## Objective
Wrap the Linear Users API call inside resolve_agent_delegates with an in-memory cache scoped to a single pipeline run so repeated calls within the same run reuse the first result.

## Steps
1. Inside `resolve-agent-delegates.ts`, introduce a module-level cache variable (e.g., `let cachedDelegates: Map<string, string> | null = null`).
2. On first invocation, perform the API call and store the result. On subsequent invocations within the same process/run, return the cached result immediately.
3. Export a `clearDelegateCache()` function to allow explicit cache invalidation between pipeline runs or in tests.
4. Add a debug-level log line when serving from cache vs. fetching fresh.
5. Keep the cache simple — no TTL needed since pipeline runs are short-lived.

## Validation
Unit test: call resolve_agent_delegates twice with the same hints; assert the Linear API mock is called exactly once. Call clearDelegateCache(), then call again; assert the mock is called a second time.
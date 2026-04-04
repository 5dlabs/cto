Implement subtask 3004: Implement SkipProvider concrete implementation

## Objective
Implement SkipProvider that returns an empty research result with skipped=true and a descriptive reason, logging a warning. This provider never throws.

## Steps
1. Create `src/research/providers/skip-provider.ts`.
2. Constructor takes no arguments.
3. `execute()` method: immediately returns `{ content: '', provider: 'skip', skipped: true, reason: 'no research provider available', responseTimeMs: 0, contentLength: 0 }`.
4. Log a structured warning: `[research:skip] No research provider available — continuing pipeline without research enrichment`.
5. This provider must never throw under any circumstances.

## Validation
Unit test: (1) execute() returns a ResearchResult with skipped=true and reason='no research provider available'. (2) A warning log is emitted containing 'no research provider available'. (3) Calling execute() multiple times never throws.
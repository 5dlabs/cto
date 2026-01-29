---
type: research
title: Effect.ts Patterns for AlertHub
---

# Effect.ts Integration Patterns

This document provides research on Effect.ts patterns relevant to the AlertHub notification system.

## Key Concepts

### 1. Effect Services Pattern

Effect Services provide dependency injection and composable services:

```typescript
import { Context, Effect, Layer } from "effect"

class SlackService extends Context.Tag("SlackService")<
  SlackService,
  {
    deliver: (message: Message) => Effect.Effect<DeliveryResult, SlackError>
  }
>() {}
```

### 2. Error Handling with Tagged Errors

```typescript
class RateLimitError extends Schema.TaggedError<RateLimitError>("RateLimitError")({
  retryAfter: Schema.Number,
}) {}
```

### 3. Retry Schedules

```typescript
Effect.retry(
  Schedule.exponential("1 second").pipe(
    Schedule.compose(Schedule.recurs(3))
  )
)
```

## References

- https://effect.website/llms.txt - Effect documentation for AI agents
- https://effect.website/docs/guides/services - Services guide
- https://github.com/Effect-TS/effect - Effect repository

# Implementation Decisions: Task 6 - Mobile App

## Decision 1: Data Persistence

**Options:** AsyncStorage, SQLite, Realm
**Category:** architecture

### Recommendation
AsyncStorage + React Query caching
- Simple for preferences
- Good for cached data
- Query handles cache invalidation

## Decision 2: Background Behavior

**Options:** Re-auth, Check token, Resume
**Category:** user-experience

### Recommendation
Check token validity
- Secure but convenient
- Quick to resume
- Graceful re-auth if needed

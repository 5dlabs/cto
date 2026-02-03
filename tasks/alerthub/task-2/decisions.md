# Implementation Decisions: Task 2 - User Authentication

## Decision 1: JWT Token Expiration

**Options:** Short-lived (15min), Medium-lived (2hrs), Long-lived (24hrs)
**Category:** security

### Recommendation
Short-lived access tokens (15min) with refresh token rotation
- Good security posture
- Still reasonable UX with refresh
- Can adjust based on user feedback

---

## Decision 2: User Profile Structure

**Options:** Fixed schema, JSON column, Separate profile service
**Category:** data-model

### Recommendation
JSONB column for profile data
- Flexible for custom fields
- Still type-safe for common fields
- Simpler than separate service

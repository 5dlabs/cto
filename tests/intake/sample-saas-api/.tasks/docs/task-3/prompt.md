# Task 3: Build team management API endpoints

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 3.

## Goal

Implement CRUD operations for teams including creation, member management, and invite link generation with 7-day expiry

## Requirements

1. Create team and team_members database tables with foreign keys
2. Implement POST /api/teams with name, description validation
3. Add GET /api/teams/:id with member count aggregation
4. Build PATCH /api/teams/:id for team settings updates
5. Create invite link system with 7-day expiry using Redis
6. Add team membership validation middleware

```rust
#[derive(sqlx::FromRow, Serialize)]
struct Team {
    id: Uuid,
    name: String,
    description: Option<String>,
    created_at: DateTime<Utc>,
    member_count: i64,
}

// POST /api/teams
async fn create_team(auth: AuthUser, Json(payload): Json<CreateTeamRequest>) -> Result<Json<Team>>

// Invite link with Redis expiry
async fn generate_invite(team_id: Uuid, redis: &redis::Client) -> Result<String>
```

## Acceptance Criteria

API integration tests for all team endpoints, test invite link expiry behavior, validate team ownership permissions, test member count accuracy

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-3): Build team management API endpoints`

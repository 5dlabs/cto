# PM Integration Parity Plan

This document outlines the plan to bring PM (Project Management) platform integrations to parity with our Linear implementation.

## Current Linear Integration (Reference Implementation)

Our Linear integration provides the following core capabilities for **ingest**:

### Core Components

1. **GraphQL Client** (`crates/integrations/src/client.rs`)
   - Authentication (API tokens + OAuth)
   - Issue CRUD operations
   - Project management
   - Team/workflow state management
   - Label operations
   - Agent activity emission

2. **Webhook Handling** (`crates/integrations/src/webhooks.rs`)
   - HMAC-SHA256 signature verification
   - Timestamp validation
   - Agent session events (created, prompted)
   - Issue/comment events

3. **Activity System** (`crates/integrations/src/activities.rs`)
   - Thought, Action, Response, Error, Elicitation types
   - Agent plan steps (pending, in-progress, completed, canceled)
   - Signals (auth, select)

4. **Intake Workflow** (`crates/linear/src/handlers/intake.rs`)
   - PRD extraction from issues
   - CTO config from labels/frontmatter
   - Tech stack detection
   - Task issue creation with dependencies
   - Workflow state transitions

5. **Models** (`crates/integrations/src/models.rs`)
   - Issue, Project, Team, Label, Comment, Document
   - Workflow states
   - Agent sessions
   - Task status mapping

### Key Ingest Flow

```
1. Webhook received (issue delegated to agent)
   ↓
2. Extract PRD content from issue description
   ↓
3. Parse CTO config from labels/frontmatter
   ↓
4. Submit intake workflow (Kubernetes/Argo)
   ↓
5. Receive callback with generated tasks
   ↓
6. Create sub-issues for each task
   ↓
7. Set up dependency relationships
   ↓
8. Emit completion activity
```

---

## Platform Comparison Matrix

| Feature | Linear | Jira | ClickUp | Monday | Notion | Trello | Asana |
|---------|--------|------|---------|--------|--------|--------|-------|
| API Type | GraphQL | REST | REST | GraphQL | REST | REST | REST |
| Webhooks | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Signature Verification | HMAC-SHA256 | HMAC-SHA256 | HMAC-SHA256 | HMAC-SHA256 | HMAC-SHA256 | Callback verify | X-Hook-Secret |
| Issue Hierarchy | Parent/Sub-issue | Epic/Story/Task | List/Task/Subtask | Board/Item/Subitem | Page/Subpage | List/Card/Checklist | Project/Task/Subtask |
| Labels/Tags | ✅ Labels | ✅ Labels | ✅ Tags | ✅ Labels (column) | ✅ Multi-select | ✅ Labels | ✅ Tags |
| Dependencies | ✅ Blocking | ✅ Linking | ✅ Dependencies | ❌ (via columns) | ❌ (via relations) | ❌ | ✅ Dependencies |
| Agent System | ✅ Native | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Comments | ✅ | ✅ | ✅ | ✅ Updates | ✅ | ✅ | ✅ Stories |

---

## Platform-Specific Implementation Plans

### 1. Jira (Atlassian)

**Documentation Sources:**
- REST API Reference: https://developer.atlassian.com/cloud/jira/platform/rest/v3/intro/
- Server/Data Center API: https://developer.atlassian.com/server/jira/platform/rest/v11002/
- Webhooks Guide: https://developer.atlassian.com/server/jira/platform/webhooks/
- JQL Reference: https://support.atlassian.com/jira-software-cloud/docs/use-advanced-search-with-jira-query-language-jql/

**Authentication:**
- Cloud: OAuth 2.0 or API Token (email + token)
- Server: Personal Access Token or Basic Auth

**Implementation Plan:**

```rust
// Core client structure
pub struct JiraClient {
    client: reqwest::Client,
    base_url: String,  // e.g., https://your-domain.atlassian.net
    auth: JiraAuth,    // Email + API Token or OAuth
}

// Issue hierarchy mapping
// Linear Issue → Jira Story/Task
// Linear Sub-issue → Jira Sub-task
// Linear Labels → Jira Labels
// Linear Priority → Jira Priority field
```

**Webhook Events:**
- `jira:issue_created`
- `jira:issue_updated`
- `comment_created`

**Nuances:**
- **CTO Config via Labels:** Use label names like `cto-cli:cursor`, `cto-model:opus`
- **PRD Content:** Issue description field (can be large, no frontmatter support natively - parse markdown)
- **Frontmatter:** Parse from description start `---\ncto:\n  cli: cursor\n---`
- **Task Creation:** Use subtask issue type with parent link
- **Dependencies:** Use issue links with "blocks"/"is blocked by" type

**Priority Mapping:**
| Linear Priority | Jira Priority |
|-----------------|---------------|
| 1 (Urgent) | Highest |
| 2 (High) | High |
| 3 (Normal) | Medium |
| 4 (Low) | Low |

---

### 2. ClickUp

**Documentation Sources:**
- API Reference: https://developer.clickup.com/reference/intro
- Webhooks: https://developer.clickup.com/docs/webhooks
- Task Webhooks Payloads: https://developer.clickup.com/docs/webhooktaskpayloads
- Authentication: https://developer.clickup.com/docs/authentication

**Authentication:**
- Personal Token (header: `Authorization: pk_xxx`)
- OAuth 2.0 for apps

**Implementation Plan:**

```rust
pub struct ClickUpClient {
    client: reqwest::Client,
    api_token: String,
    team_id: String,  // Workspace ID
}

// Hierarchy mapping
// Linear Project → ClickUp List
// Linear Issue → ClickUp Task
// Linear Sub-issue → ClickUp Subtask
// Linear Labels → ClickUp Tags
```

**Webhook Events:**
- `taskCreated`
- `taskUpdated`
- `taskCommentPosted`

**Nuances:**
- **CTO Config via Tags:** Use tag names directly (e.g., `cursor`, `opus`)
- **PRD Content:** Task description (markdown supported)
- **Custom Fields:** Can create custom fields for CTO config if tags aren't enough
- **Hierarchy:** Workspace → Space → Folder → List → Task → Subtask
- **Dependencies:** Native task dependencies with wait/blocking types

**Special Considerations:**
- ClickUp has "Statuses" per list (like workflow states)
- Custom fields can be used for structured config
- Checklists can represent subtasks alternatively

---

### 3. Monday.com

**Documentation Sources:**
- GraphQL API: https://developer.monday.com/api-reference/
- Webhooks: https://developer.monday.com/api-reference/reference/webhooks
- Items API: https://developer.monday.com/api-reference/reference/items
- GraphQL Overview: https://developer.monday.com/api-reference/docs/introduction-to-graphql

**Authentication:**
- API Token (header: `Authorization: api-token`)
- OAuth 2.0 for apps

**Implementation Plan:**

```rust
pub struct MondayClient {
    client: reqwest::Client,
    api_token: String,
}

const MONDAY_API_URL: &str = "https://api.monday.com/v2";

// GraphQL mutations (similar pattern to Linear!)
const CREATE_ITEM_MUTATION: &str = r#"
    mutation ($board_id: ID!, $item_name: String!, $column_values: JSON) {
        create_item(board_id: $board_id, item_name: $item_name, column_values: $column_values) {
            id
            name
        }
    }
"#;
```

**Webhook Events:**
- `create_item`
- `change_column_value`
- `create_update` (comments)

**Nuances:**
- **CTO Config via Columns:** Use dropdown column for CLI, text column for model
- **PRD Content:** Long text column or item updates (comments section)
- **Labels:** Monday uses "Labels" column type (predefined options)
- **No Native Dependencies:** Use status columns or linked items
- **Subitems:** Native subitem support on boards

**Column Value Mapping:**
```json
{
  "status": {"label": "In Progress"},
  "priority": {"label": "High"},
  "cto_cli": {"text": "cursor"},
  "cto_model": {"text": "claude-opus-4-20250514"}
}
```

---

### 4. Notion

**Documentation Sources:**
- API Reference: https://developers.notion.com/reference/intro
- Webhooks: https://developers.notion.com/reference/webhooks
- Create Page: https://developers.notion.com/reference/post-page
- Database Query: https://developers.notion.com/reference/post-database-query

**Authentication:**
- Internal Integration Token
- OAuth for public integrations

**Implementation Plan:**

```rust
pub struct NotionClient {
    client: reqwest::Client,
    integration_token: String,
}

const NOTION_API_URL: &str = "https://api.notion.com/v1";
const NOTION_VERSION: &str = "2022-06-28";

// Notion uses JSON blocks for content
// Page properties for metadata
```

**Webhook Events:**
- `page.created`
- `page.content_updated`
- `page.properties_updated`
- `database.content_updated`

**Nuances:**
- **CTO Config via Properties:** Multi-select for tags, Select for CLI
- **PRD Content:** Page content blocks (rich text, not just markdown)
- **Hierarchy:** Database → Page → Subpage (via parent)
- **No Dependencies:** Use Relations property to link pages
- **Labels/Tags:** Multi-select property type
- **Content Blocks:** Paragraph, heading, code, list blocks

**Property Mapping:**
```json
{
  "Status": {"select": {"name": "In Progress"}},
  "Priority": {"select": {"name": "High"}},
  "CTO CLI": {"select": {"name": "cursor"}},
  "Tags": {"multi_select": [{"name": "backend"}, {"name": "rust"}]}
}
```

**Special Considerations:**
- Content is block-based, need to convert markdown ↔ blocks
- Relations can simulate dependencies
- Rollups can aggregate child status

---

### 5. Trello

**Documentation Sources:**
- REST API: https://developer.atlassian.com/cloud/trello/rest/
- Webhooks: https://developer.atlassian.com/cloud/trello/guides/rest-api/webhooks/
- Cards: https://developer.atlassian.com/cloud/trello/rest/api-group-cards/
- Authentication: https://developer.atlassian.com/cloud/trello/guides/rest-api/authorization/

**Authentication:**
- API Key + Token (query params: `key=xxx&token=xxx`)

**Implementation Plan:**

```rust
pub struct TrelloClient {
    client: reqwest::Client,
    api_key: String,
    token: String,
}

const TRELLO_API_URL: &str = "https://api.trello.com/1";

// Simple REST client
// Cards are the main entity
```

**Webhook Events:**
- `action` events filtered by type
- `createCard`, `updateCard`, `commentCard`

**Nuances:**
- **CTO Config via Labels:** Use label names (limited colors)
- **PRD Content:** Card description (markdown)
- **Hierarchy:** Board → List → Card → Checklist (no real subtasks)
- **No Dependencies:** Manual via custom fields power-up or description
- **Labels:** Color-based with optional names
- **Checklists:** Can represent subtasks but not true sub-cards

**Webhook Setup:**
- Must verify callback URL responds to HEAD request
- Webhooks are per-model (board, card, etc.)

**Special Considerations:**
- Very simple model, limited for complex workflows
- Power-Ups can add custom fields
- Butler automation for advanced workflows
- Best for simple kanban-style intake

---

### 6. Asana

**Documentation Sources:**
- API Reference: https://developers.asana.com/reference/rest-api-reference
- Webhooks: https://developers.asana.com/docs/webhooks-guide
- Tasks: https://developers.asana.com/reference/tasks
- Create Webhook: https://developers.asana.com/reference/createwebhook

**Authentication:**
- Personal Access Token (header: `Authorization: Bearer xxx`)
- OAuth 2.0 for apps

**Implementation Plan:**

```rust
pub struct AsanaClient {
    client: reqwest::Client,
    access_token: String,
    workspace_gid: String,
}

const ASANA_API_URL: &str = "https://app.asana.com/api/1.0";

// Asana has great task hierarchy
// Projects contain tasks, tasks can have subtasks
```

**Webhook Events:**
- `added`, `changed`, `removed`, `deleted`, `undeleted`
- Resources: `task`, `project`, `story` (comment)

**Nuances:**
- **CTO Config via Tags:** Global tags applied to tasks
- **PRD Content:** Task notes field (markdown-like)
- **Hierarchy:** Workspace → Project → Section → Task → Subtask
- **Dependencies:** Native `addDependencies` endpoint
- **Tags:** Shared across workspace
- **Custom Fields:** Can add structured CTO config

**Webhook Verification:**
- Initial handshake with `X-Hook-Secret` header
- Must respond with same secret in body

**Task Dependency Types:**
- `depends_on` - this task waits for another
- `dependents` - other tasks wait for this one

---

## Implementation Priority

Based on market share and feature completeness:

| Priority | Platform | Rationale |
|----------|----------|-----------|
| 1 | **Jira** | Enterprise standard, robust API, full feature parity possible |
| 2 | **Asana** | Strong task management, native dependencies, good webhook support |
| 3 | **ClickUp** | Growing platform, good API, native dependencies |
| 4 | **Monday.com** | GraphQL API (familiar pattern), popular |
| 5 | **Notion** | Popular but requires block conversion, no native dependencies |
| 6 | **Trello** | Simple but limited for complex workflows |

---

## Common Abstraction Layer

To avoid duplicating logic, create a trait-based abstraction:

```rust
// crates/integrations/src/pm/traits.rs

#[async_trait]
pub trait PmClient: Send + Sync {
    type Issue: PmIssue;
    type Project: PmProject;
    
    // Issue operations
    async fn get_issue(&self, id: &str) -> Result<Self::Issue>;
    async fn create_issue(&self, input: IssueCreateInput) -> Result<Self::Issue>;
    async fn update_issue(&self, id: &str, input: IssueUpdateInput) -> Result<Self::Issue>;
    
    // Project operations
    async fn get_project(&self, id: &str) -> Result<Self::Project>;
    async fn create_project(&self, input: ProjectCreateInput) -> Result<Self::Project>;
    
    // Labels/Tags
    async fn get_or_create_label(&self, scope: &str, name: &str) -> Result<Label>;
    async fn set_issue_labels(&self, issue_id: &str, label_ids: &[String]) -> Result<()>;
    
    // Dependencies
    async fn create_dependency(&self, from: &str, to: &str, dep_type: DependencyType) -> Result<()>;
    
    // Activity (optional, falls back to comments)
    async fn emit_activity(&self, context: &str, content: ActivityContent) -> Result<()>;
}

pub trait PmIssue {
    fn id(&self) -> &str;
    fn identifier(&self) -> &str;  // Human-readable (e.g., "TSK-1", "PROJ-123")
    fn title(&self) -> &str;
    fn description(&self) -> Option<&str>;
    fn labels(&self) -> &[Label];
    fn extract_cto_config(&self) -> CtoConfig;
}

pub trait PmWebhook {
    fn verify_signature(&self, body: &[u8], signature: &str, secret: &str) -> bool;
    fn parse_payload(&self, body: &[u8]) -> Result<WebhookEvent>;
}
```

---

## CTO Config Extraction Strategy

Each platform needs a strategy to extract CTO config:

| Platform | Primary Method | Fallback |
|----------|---------------|----------|
| Linear | Labels (`CTO CLI/xxx`) | Frontmatter in description |
| Jira | Labels | Custom fields, frontmatter |
| ClickUp | Tags | Custom fields, frontmatter |
| Monday | Columns (select/text) | Description frontmatter |
| Notion | Properties (select/multi-select) | Page content frontmatter |
| Trello | Labels | Card description frontmatter |
| Asana | Tags | Custom fields, notes frontmatter |

**Universal Frontmatter Parser:**
```rust
/// Parse CTO config from any markdown-like content
pub fn parse_cto_config_universal(content: &str) -> Option<CtoConfig> {
    // Try YAML frontmatter
    if let Some(config) = parse_yaml_frontmatter(content) {
        return Some(config);
    }
    
    // Try JSON frontmatter
    if let Some(config) = parse_json_frontmatter(content) {
        return Some(config);
    }
    
    // Try comment-style config
    // <!-- cto: cli=cursor model=opus -->
    if let Some(config) = parse_comment_config(content) {
        return Some(config);
    }
    
    None
}
```

---

## Next Steps

1. **Phase 1: Jira Implementation** (2-3 weeks)
   - [ ] REST client with authentication
   - [ ] Issue CRUD operations
   - [ ] Webhook handler with signature verification
   - [ ] Label-based CTO config extraction
   - [ ] Subtask creation with dependencies
   - [ ] Integration tests

2. **Phase 2: Asana Implementation** (2 weeks)
   - [ ] REST client
   - [ ] Webhook setup with handshake
   - [ ] Task/subtask hierarchy
   - [ ] Native dependency support
   - [ ] Tag-based config

3. **Phase 3: Remaining Platforms** (4-6 weeks)
   - [ ] ClickUp
   - [ ] Monday.com
   - [ ] Notion
   - [ ] Trello

4. **Phase 4: Abstraction Layer** (1 week)
   - [ ] Trait definitions
   - [ ] Unified intake handler
   - [ ] Platform-agnostic workflow submission

---

## Testing Strategy

Each platform integration should have:

1. **Unit Tests:** Client method serialization/deserialization
2. **Integration Tests:** Real API calls (with sandbox/test accounts)
3. **Webhook Tests:** Signature verification, payload parsing
4. **E2E Tests:** Full intake flow in staging environment

---

## References

- Linear API: https://developers.linear.app/docs/graphql/working-with-the-graphql-api
- Jira Cloud API: https://developer.atlassian.com/cloud/jira/platform/rest/v3/intro/
- Asana API: https://developers.asana.com/docs/overview
- ClickUp API: https://developer.clickup.com/docs
- Monday.com API: https://developer.monday.com/api-reference/
- Notion API: https://developers.notion.com/
- Trello API: https://developer.atlassian.com/cloud/trello/rest/


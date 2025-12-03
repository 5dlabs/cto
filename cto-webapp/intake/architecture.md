<rpg-method>
# Repository Planning Graph (RPG) Method - Documents Platform PRD

This PRD follows the RPG methodology from Microsoft Research, separating WHAT (functional capabilities) from HOW (structural organization), then connecting them with explicit dependencies for optimal task generation.

## Document Metadata
- **Project:** CTO Platform - Documents Module
- **Status:** Draft
- **Type:** Paid Service Feature
- **Created:** December 2, 2025
- **Tech Stack:** Next.js 14+, TypeScript, PostgreSQL, BlockNote, shadcn/ui, TailwindCSS
</rpg-method>

---

<overview>

## Problem Statement

Development teams and technical leaders need a centralized, version-controlled document management system that:

1. **Eliminates context switching** - Currently, teams must jump between GitHub for markdown files, Notion for collaborative docs, and various other tools for PRDs/specs
2. **Lacks bi-directional sync** - Changes made in web interfaces don't automatically propagate to Git repositories, leading to stale documentation
3. **No agent integration** - AI coding assistants cannot programmatically read or modify documentation, limiting automation potential
4. **Fragmented workflows** - PRDs, task lists, and specs live in different systems with no unified view

## Target Users

### Primary: Technical Leaders & CTOs
- **Workflow:** Create PRDs, review specs, manage technical documentation
- **Pain Point:** Documentation scattered across tools, manual sync with repos
- **Goal:** Single source of truth for all technical docs with GitHub integration

### Secondary: Development Teams
- **Workflow:** Reference specs during implementation, update docs as code evolves
- **Pain Point:** Outdated docs, no easy editing from within development workflow
- **Goal:** Edit docs without leaving their environment, automatic sync with code repos

### Tertiary: AI Agents
- **Workflow:** Read requirements, update task status, generate/modify documentation
- **Pain Point:** No programmatic access to documentation systems
- **Goal:** MCP tool access to read/write documents as part of autonomous workflows

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Document sync latency | < 5 seconds | Time from save to GitHub commit appearing |
| Conflict resolution rate | > 95% auto-resolved | % of syncs that don't require manual intervention |
| Editor load time | < 2 seconds | Time to interactive for document editor |
| Agent API response time | < 500ms | P95 latency for MCP tool calls |
| User adoption | 80% of paid users | % using documents feature within 30 days |

</overview>

---

<functional-decomposition>

## Capability Tree

### Capability: Document Storage & Management
Core CRUD operations for markdown documents with metadata, versioning, and workspace isolation.

#### Feature: Document CRUD Operations
- **Description**: Create, read, update, and delete markdown documents with full metadata support
- **Inputs**: Document content (markdown/JSON), path, title, type, tags, workspace_id
- **Outputs**: Document object with ID, timestamps, sync status
- **Behavior**: 
  - Validate path uniqueness within workspace
  - Store both raw markdown and BlockNote JSON representation
  - Auto-generate version on each save
  - Enforce workspace isolation (multi-tenancy)

#### Feature: Document Versioning
- **Description**: Track all changes to documents with ability to view history and restore previous versions
- **Inputs**: Document ID, version number (for restore)
- **Outputs**: Version history list, specific version content, diff between versions
- **Behavior**:
  - Create new version entry on every content change
  - Store change summary (auto-generated or user-provided)
  - Calculate and store diff metadata
  - Support restore to any previous version

#### Feature: Document Search & Query
- **Description**: Full-text search across document content with metadata filtering
- **Inputs**: Search query, filters (type, tags, repo, date range)
- **Outputs**: Ranked list of matching documents with snippets
- **Behavior**:
  - Use PostgreSQL full-text search (tsvector/tsquery)
  - Rank by relevance with title matches weighted higher
  - Support exact phrase matching
  - Filter by document type, tags, linked repo

#### Feature: Document Organization
- **Description**: Hierarchical organization via paths with folder-like navigation
- **Inputs**: Document path (e.g., "docs/prds/feature-x.md")
- **Outputs**: Tree structure of documents, breadcrumb navigation data
- **Behavior**:
  - Parse paths into virtual folder hierarchy
  - Support rename/move operations updating all references
  - Generate tree view data for sidebar navigation

---

### Capability: WYSIWYG Markdown Editor
Block-based rich text editing with Notion-like UX using BlockNote + shadcn/ui components.

#### Feature: BlockNote Editor Integration
- **Description**: Initialize and render BlockNote editor with shadcn/ui theming
- **Inputs**: Initial content (markdown or BlockNote JSON), editor configuration
- **Outputs**: Interactive editor component, content change events
- **Behavior**:
  - Create editor instance via `useCreateBlockNote` hook
  - Render with `BlockNoteView` and shadcn components
  - Configure TailwindCSS theming (light/dark mode support)
  - Handle editor lifecycle (mount/unmount/focus)

#### Feature: Markdown Import/Export
- **Description**: Convert between markdown text and BlockNote's internal block format
- **Inputs**: Markdown string OR BlockNote Block[] array
- **Outputs**: Converted format (Block[] from markdown, markdown string from blocks)
- **Behavior**:
  - Use `editor.tryParseMarkdownToBlocks()` for import
  - Use `editor.blocksToMarkdownLossy()` for export
  - Handle conversion edge cases (unsupported block types)
  - Preserve frontmatter if present

#### Feature: Editor Toolbar & Commands
- **Description**: Formatting toolbar and slash command menu for block operations
- **Inputs**: User interactions (clicks, keyboard shortcuts, slash commands)
- **Outputs**: Updated document content, menu visibility state
- **Behavior**:
  - Support standard formatting (bold, italic, code, headings)
  - Slash menu for block type insertion (paragraphs, lists, code blocks, images)
  - Keyboard shortcuts for power users
  - Customizable toolbar with shadcn components

#### Feature: Auto-save & Change Detection
- **Description**: Automatically save changes with debouncing and dirty state tracking
- **Inputs**: Editor content changes, save triggers
- **Outputs**: Save status (saving, saved, error), dirty state indicator
- **Behavior**:
  - Debounce saves (500ms after last keystroke)
  - Track dirty state for unsaved changes warning
  - Optimistic UI updates with rollback on error
  - Persist both markdown and JSON representations

---

### Capability: GitHub Two-Way Sync
Bidirectional synchronization between platform documents and GitHub repositories.

#### Feature: GitHub OAuth Integration
- **Description**: Authenticate users with GitHub for repository access
- **Inputs**: OAuth authorization code, user session
- **Outputs**: GitHub access token, user's accessible repositories list
- **Behavior**:
  - Implement OAuth 2.0 flow with GitHub
  - Request minimal scopes (repo contents, webhooks)
  - Securely store encrypted tokens per workspace
  - Handle token refresh/expiration

#### Feature: Push to GitHub
- **Description**: Push document changes to linked GitHub repository
- **Inputs**: Document with changes, target repo/branch/path
- **Outputs**: Commit SHA, sync status update
- **Behavior**:
  - Fetch current file SHA from GitHub (if exists)
  - Compare SHAs to detect conflicts
  - Use GitHub Contents API to create/update file
  - Generate commit message with document title
  - Update local github_sha after successful push

#### Feature: Pull from GitHub (Webhooks)
- **Description**: Receive and process GitHub webhook events for incoming changes
- **Inputs**: GitHub webhook payload (push event)
- **Outputs**: Updated local documents, conflict flags
- **Behavior**:
  - Verify webhook signature (HMAC-SHA256)
  - Filter for .md files in tracked paths
  - Fetch changed file content from GitHub
  - Update local document if no local changes pending
  - Flag conflict if local changes exist

#### Feature: Conflict Detection & Resolution
- **Description**: Detect sync conflicts and provide resolution UI
- **Inputs**: Local document, remote GitHub version
- **Outputs**: Conflict status, diff view, resolution options
- **Behavior**:
  - Compare local and remote SHAs
  - Generate side-by-side diff view
  - Offer resolution options: keep local, keep remote, manual merge
  - Track resolution history for audit

#### Feature: Sync Configuration
- **Description**: Configure which repositories/paths to sync for a workspace
- **Inputs**: GitHub repo, branch, path prefixes, auto-sync settings
- **Outputs**: Sync configuration object, webhook registration status
- **Behavior**:
  - Store sync config per workspace
  - Register/unregister GitHub webhooks
  - Map local paths to repo paths
  - Enable/disable auto-sync per config

---

### Capability: Documents API
REST API and MCP tools for programmatic document access.

#### Feature: REST API Endpoints
- **Description**: RESTful HTTP API for all document operations
- **Inputs**: HTTP requests with JSON payloads, authentication headers
- **Outputs**: JSON responses with document data, pagination, error details
- **Behavior**:
  - Standard CRUD endpoints (/api/documents/*)
  - Query parameters for filtering and pagination
  - Consistent error response format
  - Rate limiting per workspace

#### Feature: MCP Tool Definitions
- **Description**: Model Context Protocol tools for AI agent integration
- **Inputs**: MCP tool calls with parameters
- **Outputs**: Structured responses suitable for LLM consumption
- **Behavior**:
  - Define tools: documents_list, documents_get, documents_create, documents_update, documents_search
  - Authenticate via existing MCP infrastructure
  - Return concise, structured data
  - Include sync status in responses

#### Feature: Bulk Operations
- **Description**: Batch create/update/delete operations for efficiency
- **Inputs**: Array of document operations
- **Outputs**: Array of results (success/failure per operation)
- **Behavior**:
  - Process operations in transaction
  - Return partial success results
  - Validate all operations before execution
  - Support atomic or best-effort modes

---

### Capability: User Interface Components
shadcn-based UI components for document management experience.

#### Feature: Document Sidebar & Tree View
- **Description**: Hierarchical navigation sidebar showing document structure
- **Inputs**: Document list, current selection, expanded folders
- **Outputs**: Interactive tree component with selection callbacks
- **Behavior**:
  - Render virtual folder hierarchy from document paths
  - Support expand/collapse folders
  - Highlight current document
  - Context menu for document actions (rename, delete, move)
  - Drag-and-drop reordering (future)

#### Feature: Document Toolbar
- **Description**: Action bar with save, sync, history, and settings controls
- **Inputs**: Document state, sync status, user permissions
- **Outputs**: Toolbar component with action handlers
- **Behavior**:
  - Show sync status indicator
  - Save button with dirty state awareness
  - History button to open version panel
  - Settings dropdown for document metadata
  - GitHub link to view in repo

#### Feature: Sync Status Indicator
- **Description**: Real-time visual indicator of sync state
- **Inputs**: Sync status (synced, local_changes, remote_changes, conflict)
- **Outputs**: Icon/badge with tooltip showing status details
- **Behavior**:
  - Color-coded status (green=synced, yellow=pending, red=conflict)
  - Show last sync timestamp
  - Click to trigger manual sync
  - WebSocket updates for real-time status

#### Feature: Version History Panel
- **Description**: Slide-out panel showing document version history
- **Inputs**: Document ID, version list
- **Outputs**: Scrollable version list with diff viewer
- **Behavior**:
  - List all versions with timestamps and authors
  - Click to preview version content
  - Compare any two versions with diff view
  - Restore button to revert to selected version

#### Feature: Conflict Resolution Dialog
- **Description**: Modal dialog for resolving sync conflicts
- **Inputs**: Local content, remote content, conflict metadata
- **Outputs**: Resolution choice, merged content (if manual merge)
- **Behavior**:
  - Side-by-side diff view
  - Buttons for keep local, keep remote
  - Manual merge editor for custom resolution
  - Save merged result and update sync status

</functional-decomposition>

---

<structural-decomposition>

## Repository Structure

```
src/
├── app/
│   ├── api/
│   │   ├── documents/
│   │   │   ├── route.ts                    # GET (list), POST (create)
│   │   │   ├── [id]/
│   │   │   │   ├── route.ts                # GET, PUT, DELETE single doc
│   │   │   │   ├── sync/
│   │   │   │   │   └── route.ts            # POST trigger sync
│   │   │   │   ├── diff/
│   │   │   │   │   └── route.ts            # GET diff with GitHub
│   │   │   │   ├── resolve/
│   │   │   │   │   └── route.ts            # POST resolve conflict
│   │   │   │   └── history/
│   │   │   │       └── route.ts            # GET version history
│   │   │   ├── search/
│   │   │   │   └── route.ts                # GET full-text search
│   │   │   ├── bulk/
│   │   │   │   └── route.ts                # POST bulk operations
│   │   │   └── sync-all/
│   │   │       └── route.ts                # POST sync all linked docs
│   │   └── webhooks/
│   │       └── github/
│   │           └── route.ts                # GitHub webhook handler
│   └── (dashboard)/
│       └── documents/
│           ├── page.tsx                    # Documents list/browse page
│           └── [id]/
│               └── page.tsx                # Document editor page
│
├── components/
│   └── documents/
│       ├── document-editor.tsx             # Main editor component
│       ├── document-sidebar.tsx            # Tree navigation sidebar
│       ├── document-toolbar.tsx            # Action toolbar
│       ├── document-breadcrumb.tsx         # Path breadcrumb
│       ├── sync-status-indicator.tsx       # Sync status badge
│       ├── version-history-panel.tsx       # Version history drawer
│       ├── conflict-resolution-dialog.tsx  # Conflict modal
│       ├── diff-viewer.tsx                 # Side-by-side diff
│       └── document-meta-dialog.tsx        # Metadata editor
│
├── lib/
│   └── documents/
│       ├── index.ts                        # Public exports
│       ├── types.ts                        # TypeScript interfaces
│       ├── schemas.ts                      # Zod validation schemas
│       ├── db.ts                           # Database operations
│       ├── search.ts                       # Full-text search logic
│       ├── versioning.ts                   # Version management
│       ├── github-sync.ts                  # GitHub sync operations
│       ├── webhook-handler.ts              # Webhook processing
│       ├── conflict-resolution.ts          # Conflict detection/resolution
│       ├── markdown-utils.ts               # Markdown conversion helpers
│       └── mcp-tools.ts                    # MCP tool definitions
│
├── hooks/
│   └── documents/
│       ├── use-document.ts                 # Single document state
│       ├── use-documents.ts                # Documents list state
│       ├── use-document-editor.ts          # Editor state management
│       ├── use-sync-status.ts              # Real-time sync status
│       └── use-version-history.ts          # Version history state
│
└── styles/
    └── blocknote-theme.css                 # BlockNote/shadcn theming

prisma/
└── schema.prisma                           # Database schema (documents tables)

```

## Module Definitions

### Module: `lib/documents/types.ts`
- **Maps to capability**: All capabilities (shared types)
- **Responsibility**: TypeScript type definitions for documents domain
- **Exports**:
  - `Document` - Core document interface
  - `DocumentVersion` - Version history entry
  - `GitHubSyncConfig` - Sync configuration
  - `SyncStatus` - Sync state enum
  - `ConflictResolution` - Resolution options type
  - `DocumentType` - Document type enum (prd, task, doc, spec)

### Module: `lib/documents/schemas.ts`
- **Maps to capability**: Document Storage (validation)
- **Responsibility**: Zod schemas for request/response validation
- **Exports**:
  - `createDocumentSchema` - Validate document creation
  - `updateDocumentSchema` - Validate document updates
  - `searchQuerySchema` - Validate search parameters
  - `syncConfigSchema` - Validate sync configuration

### Module: `lib/documents/db.ts`
- **Maps to capability**: Document Storage & Management
- **Responsibility**: Database operations via Prisma
- **Exports**:
  - `createDocument(data)` - Insert new document
  - `getDocument(id)` - Fetch single document
  - `updateDocument(id, data)` - Update document content
  - `deleteDocument(id)` - Soft/hard delete
  - `listDocuments(filters)` - Query with pagination
  - `getDocumentByPath(workspaceId, path)` - Path-based lookup

### Module: `lib/documents/versioning.ts`
- **Maps to capability**: Document Versioning
- **Responsibility**: Version history management
- **Exports**:
  - `createVersion(documentId, content)` - Save new version
  - `getVersionHistory(documentId)` - List all versions
  - `getVersion(documentId, version)` - Get specific version
  - `restoreVersion(documentId, version)` - Restore to version
  - `generateDiff(v1, v2)` - Compute diff between versions

### Module: `lib/documents/search.ts`
- **Maps to capability**: Document Search & Query
- **Responsibility**: Full-text search implementation
- **Exports**:
  - `searchDocuments(query, filters)` - Execute search
  - `buildSearchQuery(params)` - Construct SQL query
  - `rankResults(results)` - Score and sort results

### Module: `lib/documents/github-sync.ts`
- **Maps to capability**: GitHub Two-Way Sync
- **Responsibility**: GitHub API operations for sync
- **Exports**:
  - `pushToGitHub(document)` - Push changes to repo
  - `pullFromGitHub(repo, path)` - Fetch file content
  - `getFileSha(repo, path)` - Get current file SHA
  - `createOrUpdateFile(repo, path, content, sha?)` - GitHub Contents API
  - `registerWebhook(repo, webhookUrl)` - Setup webhook
  - `unregisterWebhook(repo, webhookId)` - Remove webhook

### Module: `lib/documents/webhook-handler.ts`
- **Maps to capability**: Pull from GitHub (Webhooks)
- **Responsibility**: Process incoming GitHub webhooks
- **Exports**:
  - `verifyWebhookSignature(payload, signature, secret)` - HMAC verification
  - `handlePushEvent(payload)` - Process push events
  - `syncFileFromGitHub(repo, branch, path)` - Sync single file
  - `filterMarkdownFiles(commits)` - Extract .md file changes

### Module: `lib/documents/conflict-resolution.ts`
- **Maps to capability**: Conflict Detection & Resolution
- **Responsibility**: Detect and resolve sync conflicts
- **Exports**:
  - `detectConflict(local, remote)` - Check for conflicts
  - `generateConflictDiff(local, remote)` - Create diff view
  - `resolveConflict(documentId, resolution)` - Apply resolution
  - `ConflictError` - Custom error class

### Module: `lib/documents/markdown-utils.ts`
- **Maps to capability**: WYSIWYG Markdown Editor
- **Responsibility**: Markdown conversion utilities
- **Exports**:
  - `markdownToBlocks(editor, markdown)` - Convert to BlockNote
  - `blocksToMarkdown(editor)` - Convert to markdown
  - `extractFrontmatter(content)` - Parse YAML frontmatter
  - `injectFrontmatter(content, meta)` - Add frontmatter

### Module: `lib/documents/mcp-tools.ts`
- **Maps to capability**: Documents API (MCP)
- **Responsibility**: MCP tool definitions for agents
- **Exports**:
  - `documentTools` - Tool definitions object
  - `handleDocumentsTool(name, params)` - Tool handler

### Module: `components/documents/document-editor.tsx`
- **Maps to capability**: WYSIWYG Markdown Editor
- **Responsibility**: Main BlockNote editor component
- **Exports**:
  - `DocumentEditor` - Editor React component
  - `DocumentEditorProps` - Component props interface

### Module: `components/documents/document-sidebar.tsx`
- **Maps to capability**: Document Organization, UI Components
- **Responsibility**: Navigation sidebar with tree view
- **Exports**:
  - `DocumentSidebar` - Sidebar React component
  - `DocumentTree` - Tree sub-component

### Module: `hooks/documents/use-document-editor.ts`
- **Maps to capability**: WYSIWYG Markdown Editor
- **Responsibility**: Editor state management hook
- **Exports**:
  - `useDocumentEditor(documentId)` - Main editor hook
  - Returns: `{ document, content, save, isDirty, syncStatus }`

### Module: `hooks/documents/use-sync-status.ts`
- **Maps to capability**: Sync Status Indicator
- **Responsibility**: Real-time sync status via WebSocket
- **Exports**:
  - `useSyncStatus(documentId)` - Sync status hook
  - Returns: `{ status, lastSynced, triggerSync }`

</structural-decomposition>

---

<dependency-graph>

## Dependency Chain

### Foundation Layer (Phase 0)
No dependencies - these are built first.

- **`lib/documents/types.ts`**: TypeScript type definitions shared across all modules
- **`lib/documents/schemas.ts`**: Zod validation schemas (depends only on types)
- **`prisma/schema.prisma`**: Database schema definitions for documents tables

### Data Layer (Phase 1)
Depends on Foundation Layer.

- **`lib/documents/db.ts`**: Depends on [types.ts, schemas.ts, prisma schema]
- **`lib/documents/search.ts`**: Depends on [types.ts, db.ts]
- **`lib/documents/versioning.ts`**: Depends on [types.ts, db.ts]

### Sync Layer (Phase 2)
Depends on Data Layer.

- **`lib/documents/markdown-utils.ts`**: Depends on [types.ts] (uses BlockNote APIs)
- **`lib/documents/github-sync.ts`**: Depends on [types.ts, db.ts]
- **`lib/documents/conflict-resolution.ts`**: Depends on [types.ts, db.ts, github-sync.ts]
- **`lib/documents/webhook-handler.ts`**: Depends on [types.ts, db.ts, github-sync.ts, conflict-resolution.ts]

### API Layer (Phase 3)
Depends on Sync Layer.

- **`app/api/documents/route.ts`**: Depends on [db.ts, schemas.ts, search.ts]
- **`app/api/documents/[id]/route.ts`**: Depends on [db.ts, schemas.ts, versioning.ts]
- **`app/api/documents/[id]/sync/route.ts`**: Depends on [db.ts, github-sync.ts, conflict-resolution.ts]
- **`app/api/documents/[id]/history/route.ts`**: Depends on [db.ts, versioning.ts]
- **`app/api/webhooks/github/route.ts`**: Depends on [webhook-handler.ts]
- **`lib/documents/mcp-tools.ts`**: Depends on [db.ts, search.ts, github-sync.ts]

### Hooks Layer (Phase 4)
Depends on API Layer.

- **`hooks/documents/use-document.ts`**: Depends on [types.ts, API routes]
- **`hooks/documents/use-documents.ts`**: Depends on [types.ts, API routes]
- **`hooks/documents/use-document-editor.ts`**: Depends on [use-document.ts, markdown-utils.ts]
- **`hooks/documents/use-sync-status.ts`**: Depends on [types.ts, WebSocket]
- **`hooks/documents/use-version-history.ts`**: Depends on [types.ts, API routes]

### UI Components Layer (Phase 5)
Depends on Hooks Layer.

- **`components/documents/sync-status-indicator.tsx`**: Depends on [types.ts, use-sync-status.ts]
- **`components/documents/document-toolbar.tsx`**: Depends on [types.ts, use-document.ts, sync-status-indicator.tsx]
- **`components/documents/document-sidebar.tsx`**: Depends on [types.ts, use-documents.ts]
- **`components/documents/document-breadcrumb.tsx`**: Depends on [types.ts]
- **`components/documents/diff-viewer.tsx`**: Depends on [types.ts]
- **`components/documents/version-history-panel.tsx`**: Depends on [types.ts, use-version-history.ts, diff-viewer.tsx]
- **`components/documents/conflict-resolution-dialog.tsx`**: Depends on [types.ts, diff-viewer.tsx]
- **`components/documents/document-meta-dialog.tsx`**: Depends on [types.ts, schemas.ts]
- **`components/documents/document-editor.tsx`**: Depends on [types.ts, use-document-editor.ts, markdown-utils.ts, document-toolbar.tsx]

### Pages Layer (Phase 6)
Depends on UI Components Layer.

- **`app/(dashboard)/documents/page.tsx`**: Depends on [document-sidebar.tsx, use-documents.ts]
- **`app/(dashboard)/documents/[id]/page.tsx`**: Depends on [document-editor.tsx, document-sidebar.tsx, version-history-panel.tsx, conflict-resolution-dialog.tsx]

</dependency-graph>

---

<implementation-roadmap>

## Development Phases

### Phase 0: Foundation
**Goal**: Establish type system, validation schemas, and database schema for documents domain.

**Entry Criteria**: Clean repository with existing Next.js + Prisma + shadcn setup

**Tasks**:
- [ ] Create TypeScript interfaces for Document, DocumentVersion, GitHubSyncConfig (depends on: none)
  - Acceptance criteria: All types exported from `lib/documents/types.ts`
  - Test strategy: TypeScript compilation passes, types used in subsequent modules

- [ ] Create Zod validation schemas for all API inputs (depends on: types.ts)
  - Acceptance criteria: Schemas validate all edge cases, export from `lib/documents/schemas.ts`
  - Test strategy: Unit tests for valid/invalid payloads

- [ ] Add Prisma schema for documents, document_versions, github_sync_configs tables (depends on: none)
  - Acceptance criteria: Migrations run successfully, all indexes created
  - Test strategy: Database can be migrated up/down, seed data works

**Exit Criteria**: `npm run build` passes, types can be imported, database migrations applied

**Delivers**: Type-safe foundation for all subsequent development

---

### Phase 1: Data Layer
**Goal**: Implement core database operations for document CRUD, versioning, and search.

**Entry Criteria**: Phase 0 complete

**Tasks**:
- [ ] Implement document database operations in `lib/documents/db.ts` (depends on: types.ts, schemas.ts, prisma)
  - Acceptance criteria: All CRUD operations work, workspace isolation enforced
  - Test strategy: Integration tests against test database

- [ ] Implement version history operations in `lib/documents/versioning.ts` (depends on: types.ts, db.ts)
  - Acceptance criteria: Versions created on save, history retrievable, restore works
  - Test strategy: Unit tests for version creation, integration tests for restore flow

- [ ] Implement full-text search in `lib/documents/search.ts` (depends on: types.ts, db.ts)
  - Acceptance criteria: Search returns ranked results, filters work correctly
  - Test strategy: Integration tests with sample documents

**Exit Criteria**: Can create, read, update, delete documents with versioning; search returns relevant results

**Delivers**: Core document storage and retrieval functionality

---

### Phase 2: Sync Layer
**Goal**: Implement GitHub two-way sync with conflict detection.

**Entry Criteria**: Phase 1 complete

**Tasks**:
- [ ] Create markdown conversion utilities in `lib/documents/markdown-utils.ts` (depends on: types.ts)
  - Acceptance criteria: Round-trip markdown → blocks → markdown preserves content
  - Test strategy: Unit tests with various markdown formats

- [ ] Implement GitHub API operations in `lib/documents/github-sync.ts` (depends on: types.ts, db.ts)
  - Acceptance criteria: Can push/pull files, handle OAuth tokens
  - Test strategy: Integration tests with GitHub API (mock in CI)

- [ ] Implement conflict detection in `lib/documents/conflict-resolution.ts` (depends on: types.ts, db.ts, github-sync.ts)
  - Acceptance criteria: Conflicts detected when SHAs mismatch, diff generated
  - Test strategy: Unit tests for conflict scenarios

- [ ] Implement webhook handler in `lib/documents/webhook-handler.ts` (depends on: types.ts, db.ts, github-sync.ts, conflict-resolution.ts)
  - Acceptance criteria: Webhooks processed, signature verified, files synced
  - Test strategy: Integration tests with mock webhook payloads

**Exit Criteria**: Documents can sync bidirectionally with GitHub, conflicts detected and flagged

**Delivers**: Complete GitHub integration for document sync

---

### Phase 3: API Layer
**Goal**: Expose REST API endpoints and MCP tools for all document operations.

**Entry Criteria**: Phase 2 complete

**Tasks**:
- [ ] Implement document list/create API routes (depends on: db.ts, schemas.ts, search.ts)
  - Acceptance criteria: GET returns paginated list, POST creates document
  - Test strategy: API integration tests

- [ ] Implement single document API routes (depends on: db.ts, schemas.ts, versioning.ts)
  - Acceptance criteria: GET/PUT/DELETE work correctly, versions tracked
  - Test strategy: API integration tests

- [ ] Implement sync API routes (depends on: db.ts, github-sync.ts, conflict-resolution.ts)
  - Acceptance criteria: Sync triggers work, conflicts returned properly
  - Test strategy: API integration tests with mock GitHub

- [ ] Implement GitHub webhook endpoint (depends on: webhook-handler.ts)
  - Acceptance criteria: Webhook processes push events, returns 200
  - Test strategy: Integration tests with mock payloads

- [ ] Implement MCP tool definitions (depends on: db.ts, search.ts, github-sync.ts)
  - Acceptance criteria: Tools callable via MCP, return structured data
  - Test strategy: MCP tool invocation tests

**Exit Criteria**: All API endpoints return correct responses, MCP tools functional

**Delivers**: Programmatic access to all document functionality

---

### Phase 4: Hooks Layer
**Goal**: Create React hooks for client-side state management.

**Entry Criteria**: Phase 3 complete

**Tasks**:
- [ ] Implement `useDocument` hook for single document state (depends on: types.ts, API routes)
  - Acceptance criteria: Fetches document, provides update functions
  - Test strategy: React Testing Library tests

- [ ] Implement `useDocuments` hook for document list (depends on: types.ts, API routes)
  - Acceptance criteria: Fetches list with filters, handles pagination
  - Test strategy: React Testing Library tests

- [ ] Implement `useDocumentEditor` hook for editor state (depends on: use-document.ts, markdown-utils.ts)
  - Acceptance criteria: Manages content, dirty state, save operations
  - Test strategy: React Testing Library tests

- [ ] Implement `useSyncStatus` hook with WebSocket (depends on: types.ts)
  - Acceptance criteria: Real-time sync status updates
  - Test strategy: WebSocket mock tests

- [ ] Implement `useVersionHistory` hook (depends on: types.ts, API routes)
  - Acceptance criteria: Fetches version list, supports restore
  - Test strategy: React Testing Library tests

**Exit Criteria**: All hooks functional, state management works correctly

**Delivers**: Reusable state management for UI components

---

### Phase 5: UI Components
**Goal**: Build all shadcn-based UI components for document management.

**Entry Criteria**: Phase 4 complete

**Tasks**:
- [ ] Implement sync status indicator component (depends on: types.ts, use-sync-status.ts)
  - Acceptance criteria: Shows correct status colors, tooltips
  - Test strategy: Visual regression tests, component tests

- [ ] Implement document toolbar component (depends on: types.ts, use-document.ts, sync-status-indicator.tsx)
  - Acceptance criteria: All actions work, state reflected correctly
  - Test strategy: Component tests with mocked hooks

- [ ] Implement document sidebar with tree view (depends on: types.ts, use-documents.ts)
  - Acceptance criteria: Tree renders correctly, navigation works
  - Test strategy: Component tests, interaction tests

- [ ] Implement diff viewer component (depends on: types.ts)
  - Acceptance criteria: Side-by-side diff renders, changes highlighted
  - Test strategy: Snapshot tests with various diffs

- [ ] Implement version history panel (depends on: types.ts, use-version-history.ts, diff-viewer.tsx)
  - Acceptance criteria: Versions listed, diff viewable, restore works
  - Test strategy: Component tests

- [ ] Implement conflict resolution dialog (depends on: types.ts, diff-viewer.tsx)
  - Acceptance criteria: Options work, resolution saves correctly
  - Test strategy: Component tests with mock conflicts

- [ ] Implement document editor component with BlockNote (depends on: types.ts, use-document-editor.ts, markdown-utils.ts, document-toolbar.tsx)
  - Acceptance criteria: Editor renders, content editable, saves work
  - Test strategy: Component tests, E2E tests

**Exit Criteria**: All components render correctly, interactions work as expected

**Delivers**: Complete UI component library for documents feature

---

### Phase 6: Pages & Integration
**Goal**: Build final pages and integrate all components.

**Entry Criteria**: Phase 5 complete

**Tasks**:
- [ ] Build documents list/browse page (depends on: document-sidebar.tsx, use-documents.ts)
  - Acceptance criteria: Page loads, navigation works, search functional
  - Test strategy: E2E tests

- [ ] Build document editor page (depends on: document-editor.tsx, document-sidebar.tsx, version-history-panel.tsx, conflict-resolution-dialog.tsx)
  - Acceptance criteria: Full editing flow works, sync functional
  - Test strategy: E2E tests covering full workflows

- [ ] Implement real-time sync status WebSocket (depends on: all components)
  - Acceptance criteria: Status updates in real-time across clients
  - Test strategy: Multi-client E2E tests

- [ ] Polish and optimize (depends on: all previous)
  - Acceptance criteria: Load times meet targets, no console errors
  - Test strategy: Performance testing, error monitoring

**Exit Criteria**: Feature fully functional end-to-end, all tests pass

**Delivers**: Production-ready Documents Platform feature

</implementation-roadmap>

---

<test-strategy>

## Test Pyramid

```
        /\
       /E2E\       ← 10% (Full user flows, sync scenarios)
      /------\
     /Integration\ ← 30% (API routes, database operations, GitHub API)
    /------------\
   /  Unit Tests  \ ← 60% (Pure functions, validation, utilities)
  /----------------\
```

## Coverage Requirements
- Line coverage: 80% minimum
- Branch coverage: 75% minimum
- Function coverage: 85% minimum
- Statement coverage: 80% minimum

## Critical Test Scenarios

### Document CRUD (`lib/documents/db.ts`)
**Happy path**:
- Create document with all fields, verify saved correctly
- Read document by ID, verify all fields returned
- Update document content, verify version created
- Delete document, verify soft delete behavior
- Expected: Operations succeed, data integrity maintained

**Edge cases**:
- Create document with duplicate path in workspace
- Create document with very long content (10MB+)
- Update document that doesn't exist
- Expected: Appropriate errors thrown, no data corruption

**Error cases**:
- Database connection failure during save
- Invalid workspace_id reference
- Expected: Transaction rollback, clear error messages

**Integration points**:
- Version created automatically on update
- Search index updated on content change
- Expected: Side effects execute correctly

---

### GitHub Sync (`lib/documents/github-sync.ts`)
**Happy path**:
- Push new file to GitHub, verify commit created
- Push update to existing file, verify SHA updated
- Pull file changes from webhook, verify local updated
- Expected: Sync completes, SHAs match

**Edge cases**:
- Push when file was modified on GitHub (conflict)
- Webhook for file outside tracked paths
- Sync file with special characters in path
- Expected: Conflicts detected, non-tracked files ignored

**Error cases**:
- GitHub API rate limit exceeded
- Invalid OAuth token
- Repository not found / no access
- Expected: Clear error messages, no data loss

**Integration points**:
- Conflict detection triggers on SHA mismatch
- Webhook updates correct document
- Expected: Full sync flow works end-to-end

---

### Conflict Resolution (`lib/documents/conflict-resolution.ts`)
**Happy path**:
- Detect conflict when local and remote differ
- Generate accurate diff between versions
- Resolve with "keep local" - local pushed to GitHub
- Resolve with "keep remote" - local updated from GitHub
- Expected: Resolution clears conflict status

**Edge cases**:
- Conflict on newly created file (no previous SHA)
- Both versions identical (false conflict)
- Very large diff (thousands of lines)
- Expected: Handled gracefully, no hangs

**Error cases**:
- GitHub unreachable during resolution
- Document deleted during conflict resolution
- Expected: Clear errors, recovery possible

---

### BlockNote Editor (`components/documents/document-editor.tsx`)
**Happy path**:
- Load document, editor initializes with content
- Make edits, dirty state reflects changes
- Save document, content persisted
- Expected: Editing experience smooth

**Edge cases**:
- Load empty document
- Load document with unsupported markdown elements
- Very large document (1000+ blocks)
- Expected: Graceful handling, no crashes

**Error cases**:
- Save fails due to network error
- Concurrent edit from another client
- Expected: User notified, content not lost

---

### Search (`lib/documents/search.ts`)
**Happy path**:
- Search returns relevant results ranked correctly
- Filter by type returns only matching documents
- Filter by tags returns tagged documents
- Expected: Results accurate and fast

**Edge cases**:
- Search query with special characters
- Empty search query
- No results found
- Expected: No errors, empty results handled

---

## Test Generation Guidelines

1. **Unit tests**: Use Vitest with mocking for external dependencies
2. **Integration tests**: Use test database with seeded data
3. **E2E tests**: Use Playwright for full browser testing
4. **Mock strategy**: Mock GitHub API in CI, use real API in local integration tests
5. **Test data**: Create fixtures for documents, versions, sync configs
6. **Naming convention**: `describe('module')` → `it('should behavior when condition')`

</test-strategy>

---

<architecture>

## System Components

### Frontend (Next.js App Router)
- **Pages**: Server components for initial data loading
- **Components**: Client components for interactivity (shadcn/ui)
- **Editor**: BlockNote with shadcn integration
- **State**: React Query for server state, local state for UI

### Backend (Next.js API Routes)
- **REST API**: Standard CRUD + sync endpoints
- **Webhooks**: GitHub push event handler
- **MCP Tools**: Agent-accessible document operations

### Database (PostgreSQL)
- **Primary storage**: Documents, versions, sync configs
- **Full-text search**: tsvector indexes on content
- **JSON storage**: BlockNote editor state (JSONB)

### External Services
- **GitHub API**: OAuth, Contents API, Webhooks
- **WebSocket**: Real-time sync status (via existing infrastructure)

## Data Models

### Core Document Schema
```typescript
interface Document {
  id: string;                    // UUID
  workspace_id: string;          // Multi-tenant isolation
  path: string;                  // Virtual path (e.g., "docs/prds/feature.md")
  title: string;                 // Document title
  content: string;               // Raw markdown content
  content_json?: object;         // BlockNote JSON (for WYSIWYG state)
  
  // GitHub sync metadata
  github_repo?: string;          // e.g., "org/repo"
  github_path?: string;          // Path in repo
  github_sha?: string;           // Last known commit SHA
  github_branch: string;         // Default: "main"
  sync_status: SyncStatus;       // synced | local_changes | remote_changes | conflict
  last_synced_at?: Date;
  
  // Document metadata
  type: DocumentType;            // prd | task | doc | spec | other
  tags: string[];
  
  // Timestamps & audit
  created_at: Date;
  updated_at: Date;
  created_by: string;
  updated_by: string;
}

type SyncStatus = 'synced' | 'local_changes' | 'remote_changes' | 'conflict';
type DocumentType = 'prd' | 'task' | 'doc' | 'spec' | 'other';
```

## Technology Stack

| Layer | Technology | Rationale |
|-------|------------|-----------|
| Framework | Next.js 14+ (App Router) | Existing stack, SSR, API routes |
| Language | TypeScript | Type safety, better DX |
| Database | PostgreSQL | Full-text search, JSONB, existing cluster |
| ORM | Prisma | Type-safe queries, migrations |
| UI | shadcn/ui + TailwindCSS | Existing design system |
| Editor | BlockNote | Notion-like UX, shadcn support, MIT license |
| Validation | Zod | Runtime type checking |
| State | React Query | Server state management |
| Testing | Vitest + Playwright | Unit/E2E coverage |

## Key Design Decisions

**Decision: BlockNote over Tiptap**
- **Rationale**: Native shadcn support, Notion-like UX out of box, less custom code
- **Trade-offs**: Slightly larger bundle, React-only
- **Alternatives**: Tiptap (more flexible, more work), Lexical (Meta backing, less mature)

**Decision: PostgreSQL over Supabase**
- **Rationale**: Self-hosted for full control, existing cluster infrastructure, no external dependencies
- **Trade-offs**: More ops overhead, no real-time features built-in
- **Alternatives**: Supabase (managed, real-time), SQLite (simpler, less scalable)

**Decision: Optimistic sync with conflict detection**
- **Rationale**: Better UX than locking, matches Git mental model
- **Trade-offs**: Conflict resolution UI needed
- **Alternatives**: Pessimistic locking (simpler, worse UX), CRDT (complex, overkill)

**Decision: Store both markdown and JSON**
- **Rationale**: Markdown for GitHub sync/display, JSON for editor state
- **Trade-offs**: Dual storage, potential drift
- **Alternatives**: JSON only (lossy on GitHub), markdown only (editor state loss)

</architecture>

---

<risks>

## Technical Risks

**Risk: BlockNote markdown conversion is lossy**
- **Impact**: High - Complex formatting may not round-trip correctly
- **Likelihood**: Medium - Known limitation documented by BlockNote
- **Mitigation**: Store both formats, warn users about unsupported features, test edge cases thoroughly
- **Fallback**: Fall back to raw markdown editing for problematic documents

**Risk: GitHub API rate limits during heavy sync**
- **Impact**: Medium - Sync delays, user frustration
- **Likelihood**: Medium - Depends on usage patterns
- **Mitigation**: Implement request queuing, caching, exponential backoff
- **Fallback**: Manual sync mode when rate limited

**Risk: Conflict resolution complexity**
- **Impact**: High - Data loss if handled incorrectly
- **Likelihood**: Medium - Teams editing same docs
- **Mitigation**: Clear conflict UI, always preserve both versions, audit logging
- **Fallback**: Fork document on unresolvable conflict

## Dependency Risks

**Risk: BlockNote API changes**
- **Impact**: Medium - Breaking changes require updates
- **Likelihood**: Low - Stable API, MIT license allows forking
- **Mitigation**: Pin versions, monitor releases, abstract editor interface
- **Fallback**: Fork BlockNote or migrate to Tiptap

**Risk: GitHub webhook delivery reliability**
- **Impact**: Medium - Missed updates, stale local data
- **Likelihood**: Low - GitHub webhooks generally reliable
- **Mitigation**: Implement webhook delivery verification, periodic full sync
- **Fallback**: Manual sync button always available

## Scope Risks

**Risk: Real-time collaboration scope creep**
- **Impact**: High - Significant additional complexity
- **Likelihood**: Medium - Users may request Notion-like collab
- **Mitigation**: Explicitly defer to "Future Enhancements", focus on single-user + GitHub sync
- **Fallback**: Document limitation clearly in product

**Risk: Supporting all markdown flavors**
- **Impact**: Medium - Edge cases in parsing/rendering
- **Likelihood**: High - Users have varied markdown sources
- **Mitigation**: Document supported syntax, provide preview mode
- **Fallback**: Raw markdown mode for unsupported content

</risks>

---

<appendix>

## References

- [BlockNote Documentation](https://www.blocknotejs.org/docs)
- [BlockNote shadcn Integration](https://www.blocknotejs.org/docs/getting-started/shadcn)
- [BlockNote Format Interoperability](https://www.blocknotejs.org/docs/foundations/supported-formats)
- [GitHub Contents API](https://docs.github.com/en/rest/repos/contents)
- [GitHub Webhooks](https://docs.github.com/en/webhooks)
- [PostgreSQL Full-Text Search](https://www.postgresql.org/docs/current/textsearch.html)

## Glossary

- **Block**: Single unit of content in BlockNote (paragraph, heading, list, etc.)
- **SHA**: Git commit hash, used to track file versions on GitHub
- **Sync Status**: Current state of document relative to GitHub (synced, local_changes, remote_changes, conflict)
- **Workspace**: Multi-tenant isolation unit for documents and configs
- **MCP**: Model Context Protocol - Standard for AI agent tool integration

## Open Questions

1. **Collaboration model**: Single user per document or allow concurrent editing?
   - Current assumption: Single user, GitHub as collaboration layer
   
2. **Offline support**: Should documents work offline?
   - Current assumption: No, requires network for sync
   
3. **File upload handling**: Where to store images/attachments?
   - Options: S3/R2, base64 inline, external links only
   
4. **Version retention policy**: How many versions to keep?
   - Proposal: Configurable per workspace, default 50 versions

</appendix>

---

<task-master-integration>

## How Task Master Uses This PRD

When parsing this PRD, Task Master will:

1. **Extract capabilities** → Top-level tasks (e.g., "Document Storage & Management")
2. **Extract features** → Subtasks under each capability
3. **Parse dependency graph** → Set task dependencies for correct ordering
4. **Use phases** → Prioritize tasks (Phase 0 = highest priority)
5. **Use test strategy** → Guide test generation during implementation

## Expected Task Structure

```
Task 1: Foundation - Types & Schemas (Phase 0)
  - Subtask 1.1: Create TypeScript interfaces
  - Subtask 1.2: Create Zod validation schemas
  - Subtask 1.3: Add Prisma schema

Task 2: Data Layer - Database Operations (Phase 1)
  - Subtask 2.1: Implement document CRUD
  - Subtask 2.2: Implement versioning
  - Subtask 2.3: Implement search
  Dependencies: [1]

Task 3: Sync Layer - GitHub Integration (Phase 2)
  - Subtask 3.1: Markdown utilities
  - Subtask 3.2: GitHub sync operations
  - Subtask 3.3: Conflict detection
  - Subtask 3.4: Webhook handler
  Dependencies: [2]

... and so on
```

## Recommended Parse Command

```bash
task-master parse-prd .taskmaster/docs/prd.txt --research
```

</task-master-integration>


# Documents Platform Architecture

> **Status:** Draft  
> **Type:** Paid Service Feature  
> **Last Updated:** December 2, 2025

## Overview

The Documents Platform is a markdown-based document management system that provides:
- Two-way synchronization with GitHub repositories
- WYSIWYG markdown editing with a modern UI
- PRD and task document management
- API access for programmatic operations (agent integration)

This is part of the CTO Platform paid service offering.

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           CTO Web Application                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                      Documents UI (shadcn)                       │    │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌────────────────┐   │    │
│  │  │   Document      │  │   Sidebar/      │  │   Version      │   │    │
│  │  │   Editor        │  │   Tree View     │  │   History      │   │    │
│  │  │   (WYSIWYG)     │  │                 │  │                │   │    │
│  │  └────────┬────────┘  └────────┬────────┘  └───────┬────────┘   │    │
│  └───────────┼────────────────────┼───────────────────┼────────────┘    │
│              │                    │                   │                  │
│              └────────────────────┼───────────────────┘                  │
│                                   │                                      │
│  ┌────────────────────────────────┼────────────────────────────────┐    │
│  │                    Documents API Layer                           │    │
│  │                                                                  │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐   │    │
│  │  │   CRUD       │  │   Search/    │  │   GitHub Sync        │   │    │
│  │  │   Operations │  │   Query      │  │   Controller         │   │    │
│  │  └──────────────┘  └──────────────┘  └──────────┬───────────┘   │    │
│  └─────────────────────────────────────────────────┼───────────────┘    │
│                                                    │                     │
└────────────────────────────────────────────────────┼─────────────────────┘
                                                     │
                 ┌───────────────────────────────────┼───────────────────┐
                 │                                   │                   │
                 ▼                                   ▼                   ▼
┌─────────────────────────┐    ┌─────────────────────────┐    ┌──────────────────┐
│     Documents DB        │    │      GitHub API          │    │   Agent API      │
│  (PostgreSQL/Supabase)  │    │   (Webhooks + REST)      │    │   (MCP Server)   │
└─────────────────────────┘    └─────────────────────────┘    └──────────────────┘
```

---

## Core Components

### 1. Document Storage

**Primary Storage:** Self-hosted PostgreSQL (existing cluster infrastructure)

```typescript
interface Document {
  id: string;                    // UUID
  workspace_id: string;          // Multi-tenant workspace
  path: string;                  // e.g., "docs/prds/feature-x.md"
  title: string;
  content: string;               // Markdown content
  content_json?: object;         // Editor JSON format (for WYSIWYG)
  
  // Sync metadata
  github_repo?: string;          // e.g., "5dlabs/project"
  github_path?: string;          // Path in repo
  github_sha?: string;           // Last known commit SHA
  sync_status: 'synced' | 'local_changes' | 'remote_changes' | 'conflict';
  last_synced_at?: Date;
  
  // Standard metadata
  created_at: Date;
  updated_at: Date;
  created_by: string;
  updated_by: string;
  
  // Document type
  type: 'prd' | 'task' | 'doc' | 'spec' | 'other';
  tags: string[];
}
```

### 2. GitHub Two-Way Sync

#### Sync Flow: Platform → GitHub

```
1. User edits document in web UI
2. Save triggers debounced sync check
3. If document has github_path:
   a. Fetch current file SHA from GitHub
   b. Compare with stored github_sha
   c. If match: Push changes via GitHub Contents API
   d. If mismatch: Flag conflict, show diff UI
4. Update local github_sha on successful push
```

#### Sync Flow: GitHub → Platform

```
1. GitHub webhook fires on push event
2. Webhook handler:
   a. Filter for .md files in tracked paths
   b. For each changed file:
      - Fetch new content from GitHub
      - Compare with local document
      - If local unchanged: Update content + SHA
      - If local changed: Flag conflict
3. Real-time notification to connected clients
```

#### Conflict Resolution Strategy

```typescript
type ConflictResolution = 
  | 'keep_local'      // Overwrite GitHub with local
  | 'keep_remote'     // Overwrite local with GitHub
  | 'merge'           // Manual merge (show diff)
  | 'fork'            // Create new document from local
```

### 3. Documents API

#### REST Endpoints

```yaml
# Document CRUD
GET    /api/documents                    # List documents (with filters)
GET    /api/documents/:id                # Get document
POST   /api/documents                    # Create document
PUT    /api/documents/:id                # Update document
DELETE /api/documents/:id                # Delete document

# Bulk operations
POST   /api/documents/bulk               # Bulk create/update
DELETE /api/documents/bulk               # Bulk delete

# GitHub Sync
POST   /api/documents/:id/sync           # Trigger sync for document
POST   /api/documents/sync-all           # Sync all linked documents
GET    /api/documents/:id/diff           # Get diff with GitHub version
POST   /api/documents/:id/resolve        # Resolve conflict

# Search & Query
GET    /api/documents/search?q=          # Full-text search
GET    /api/documents/by-type/:type      # Filter by type
GET    /api/documents/by-repo/:repo      # Filter by GitHub repo

# Version History
GET    /api/documents/:id/history        # Get version history
GET    /api/documents/:id/versions/:ver  # Get specific version
POST   /api/documents/:id/restore/:ver   # Restore version
```

#### Agent API (MCP Integration)

For agent access, expose the same API with authentication via the existing MCP infrastructure:

```typescript
// MCP Tool definitions for agents
const documentTools = {
  'documents_list': {
    description: 'List documents with optional filters',
    parameters: { workspace_id, type?, tags?, repo? }
  },
  'documents_get': {
    description: 'Get document content by ID or path',
    parameters: { id | path }
  },
  'documents_create': {
    description: 'Create a new document',
    parameters: { path, title, content, type, github_path? }
  },
  'documents_update': {
    description: 'Update document content',
    parameters: { id, content, sync_to_github? }
  },
  'documents_search': {
    description: 'Search documents by content or metadata',
    parameters: { query, type?, tags? }
  }
};
```

---

## WYSIWYG Editor SDK Options

We evaluated the top open-source WYSIWYG editor SDKs for React/TypeScript. All options below are **open source** and can be **self-hosted** with no external dependencies.

---

### Top 5 Editor SDKs

#### 1. BlockNote ⭐ Recommended

| | |
|---|---|
| **Website** | https://www.blocknotejs.org |
| **GitHub** | https://github.com/TypeCellOS/BlockNote |
| **Stars** | 8.8k |
| **License** | MIT (fully open source) |
| **Bundle Size** | ~150kb (gzipped) |

**What it is:** A Notion-style block-based editor built on top of Tiptap and ProseMirror.

**Key Features:**
- ✅ Native shadcn/ui integration (official docs)
- ✅ Block-based editing (drag/drop, slash commands)
- ✅ Markdown import/export built-in
- ✅ Yjs collaboration support
- ✅ Customizable themes and components
- ✅ TypeScript-first

**Best For:** Notion-like editing experience with minimal setup.

```typescript
import { useCreateBlockNote } from "@blocknote/react";
import { BlockNoteView } from "@blocknote/shadcn";
import "@blocknote/shadcn/style.css";

const editor = useCreateBlockNote();
return <BlockNoteView editor={editor} />;
```

**Drawbacks:** React-only, heavier than minimal editors.

---

#### 2. Tiptap

| | |
|---|---|
| **Website** | https://tiptap.dev |
| **GitHub** | https://github.com/ueberdosis/tiptap |
| **Stars** | 34k |
| **License** | MIT (core is open source) |
| **Bundle Size** | ~45kb core (gzipped) |

**What it is:** A headless, framework-agnostic rich text editor framework built on ProseMirror.

**Key Features:**
- ✅ Smallest core bundle size
- ✅ 100+ official extensions
- ✅ Framework-agnostic (React, Vue, vanilla JS)
- ✅ Highly extensible and customizable
- ✅ Markdown extension available
- ✅ Yjs collaboration support
- ✅ Excellent documentation

**Best For:** Maximum flexibility, custom UI requirements.

```typescript
import { useEditor, EditorContent } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import Markdown from '@tiptap/extension-markdown';

const editor = useEditor({
  extensions: [StarterKit, Markdown],
  content: '# Hello World',
});
return <EditorContent editor={editor} />;
```

**Drawbacks:** Requires building your own UI, some "pro" extensions are paid.

---

#### 3. Plate

| | |
|---|---|
| **Website** | https://platejs.org |
| **GitHub** | https://github.com/udecode/plate |
| **Stars** | 16k |
| **License** | MIT |
| **Bundle Size** | ~200kb+ (varies by plugins) |

**What it is:** A rich-text editor framework for React built on Slate.js with extensive plugin ecosystem.

**Key Features:**
- ✅ 50+ plugins available
- ✅ AI-powered features (copilot, suggestions)
- ✅ Native shadcn/ui components
- ✅ Server-side editing support
- ✅ Comments, mentions, tables built-in
- ✅ Markdown support
- ✅ Paid templates available (but core is free)

**Best For:** Feature-rich editor with AI capabilities.

```typescript
import { Plate, PlateContent } from '@udecode/plate-common';
import { createPlugins } from '@udecode/plate-core';

const plugins = createPlugins([
  // Add plugins here
]);

return (
  <Plate plugins={plugins}>
    <PlateContent />
  </Plate>
);
```

**Drawbacks:** React-only, larger bundle, Hocuspocus-only for collaboration.

---

#### 4. Lexical

| | |
|---|---|
| **Website** | https://lexical.dev |
| **GitHub** | https://github.com/facebook/lexical |
| **Stars** | 23k |
| **License** | MIT |
| **Bundle Size** | ~70kb (gzipped) |

**What it is:** An extensible text editor framework from Meta (Facebook), used in Facebook and other Meta products.

**Key Features:**
- ✅ Backed by Meta (active development)
- ✅ Framework-agnostic core
- ✅ React package available
- ✅ Headless server-side editing
- ✅ iOS Swift package
- ✅ Markdown import/export
- ✅ Yjs collaboration (with caveats)

**Best For:** Projects needing Meta-level support and cross-platform.

```typescript
import { LexicalComposer } from '@lexical/react/LexicalComposer';
import { RichTextPlugin } from '@lexical/react/LexicalRichTextPlugin';
import { ContentEditable } from '@lexical/react/LexicalContentEditable';

const config = { namespace: 'MyEditor', nodes: [] };

return (
  <LexicalComposer initialConfig={config}>
    <RichTextPlugin contentEditable={<ContentEditable />} />
  </LexicalComposer>
);
```

**Drawbacks:** Still maturing (no 1.0 yet), collaboration has edge cases, lacks pure decorations.

---

#### 5. Slate

| | |
|---|---|
| **Website** | https://docs.slatejs.org |
| **GitHub** | https://github.com/ianstormtaylor/slate |
| **Stars** | 31k |
| **License** | MIT |
| **Bundle Size** | ~60kb (gzipped) |

**What it is:** A completely customizable framework for building rich text editors.

**Key Features:**
- ✅ Complete control over rendering
- ✅ Framework-agnostic core
- ✅ React package available
- ✅ Yjs collaboration via slate-yjs
- ✅ Excellent documentation
- ✅ Used by Discord, Grafana, Sanity

**Best For:** Maximum customization, building from scratch.

```typescript
import { createEditor } from 'slate';
import { Slate, Editable, withReact } from 'slate-react';

const editor = useMemo(() => withReact(createEditor()), []);

return (
  <Slate editor={editor} initialValue={initialValue}>
    <Editable />
  </Slate>
);
```

**Drawbacks:** Fewer out-of-the-box features, requires building most UI yourself.

---

### Comparison Matrix

| Feature | BlockNote | Tiptap | Plate | Lexical | Slate |
|---------|-----------|--------|-------|---------|-------|
| **License** | MIT | MIT | MIT | MIT | MIT |
| **Self-Hosted** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **shadcn Support** | ✅ Official | Custom | ✅ Official | Custom | Custom |
| **Block-Based** | ✅ | Plugin | ✅ | Plugin | Custom |
| **Markdown** | ✅ Built-in | ✅ Extension | ✅ Plugin | ✅ Package | Custom |
| **Yjs Collab** | ✅ | ✅ | Hocuspocus | ✅ (caveats) | ✅ |
| **Server-Side** | Via ProseMirror | Via ProseMirror | ✅ | ✅ Headless | ❌ |
| **Bundle Size** | Medium | Small | Large | Medium | Small |
| **Learning Curve** | Low | Medium | Medium | High | High |
| **Maturity** | Good | Excellent | Good | Growing | Excellent |

---

### Recommendation

**Primary: BlockNote**
- Best balance of features vs. effort
- Native shadcn integration saves significant time
- Notion-style UX matches modern expectations
- Markdown conversion critical for GitHub sync

**Alternative: Tiptap**
- If we need more control or custom block types
- Smaller bundle if that becomes a concern
- Can always "eject" to Tiptap since BlockNote is built on it

---

### Editor Integration Example (BlockNote + shadcn)

```typescript
// components/document-editor.tsx
import { useCreateBlockNote } from "@blocknote/react";
import { BlockNoteView } from "@blocknote/shadcn";
import "@blocknote/shadcn/style.css";
import { 
  Block,
  BlockNoteEditor,
  PartialBlock 
} from "@blocknote/core";

interface DocumentEditorProps {
  initialContent?: string;  // Markdown
  onChange?: (markdown: string) => void;
}

export function DocumentEditor({ initialContent, onChange }: DocumentEditorProps) {
  const editor = useCreateBlockNote({
    initialContent: initialContent 
      ? await editor.tryParseMarkdownToBlocks(initialContent)
      : undefined,
  });

  const handleChange = async () => {
    const markdown = await editor.blocksToMarkdownLossy(editor.document);
    onChange?.(markdown);
  };

  return (
    <BlockNoteView 
      editor={editor} 
      onChange={handleChange}
      theme="light"
    />
  );
}
```

```typescript
// Markdown conversion utilities
export async function markdownToBlocks(
  editor: BlockNoteEditor, 
  markdown: string
): Promise<Block[]> {
  return await editor.tryParseMarkdownToBlocks(markdown);
}

export async function blocksToMarkdown(
  editor: BlockNoteEditor
): Promise<string> {
  return await editor.blocksToMarkdownLossy(editor.document);
}
```

---

## Data Model

### Database Schema

```sql
-- Documents table
CREATE TABLE documents (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  workspace_id UUID NOT NULL REFERENCES workspaces(id),
  
  -- Content
  path TEXT NOT NULL,
  title TEXT NOT NULL,
  content TEXT NOT NULL,
  content_json JSONB,
  
  -- GitHub sync
  github_repo TEXT,
  github_path TEXT,
  github_sha TEXT,
  github_branch TEXT DEFAULT 'main',
  sync_status TEXT DEFAULT 'synced',
  last_synced_at TIMESTAMPTZ,
  
  -- Metadata
  type TEXT DEFAULT 'doc',
  tags TEXT[] DEFAULT '{}',
  
  -- Timestamps
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  created_by UUID REFERENCES users(id),
  updated_by UUID REFERENCES users(id),
  
  UNIQUE(workspace_id, path)
);

-- Version history
CREATE TABLE document_versions (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
  version INTEGER NOT NULL,
  content TEXT NOT NULL,
  content_json JSONB,
  github_sha TEXT,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  created_by UUID REFERENCES users(id),
  change_summary TEXT,
  
  UNIQUE(document_id, version)
);

-- GitHub sync config
CREATE TABLE github_sync_configs (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  workspace_id UUID NOT NULL REFERENCES workspaces(id),
  github_repo TEXT NOT NULL,
  github_branch TEXT DEFAULT 'main',
  sync_path TEXT DEFAULT 'docs/',  -- Path prefix in repo
  local_path TEXT DEFAULT 'docs/', -- Path prefix locally
  auto_sync BOOLEAN DEFAULT true,
  webhook_secret TEXT,
  last_webhook_at TIMESTAMPTZ,
  
  UNIQUE(workspace_id, github_repo)
);

-- Indexes
CREATE INDEX idx_documents_workspace ON documents(workspace_id);
CREATE INDEX idx_documents_path ON documents(workspace_id, path);
CREATE INDEX idx_documents_github ON documents(github_repo, github_path);
CREATE INDEX idx_documents_type ON documents(workspace_id, type);
CREATE INDEX idx_documents_content_search ON documents USING GIN(to_tsvector('english', content));
```

---

## UI Components (shadcn-based)

### Document Editor View

```
┌────────────────────────────────────────────────────────────────────────┐
│ ┌──────────────────────────────────────────────────────────────────┐   │
│ │ [←] Documents / PRDs / Feature X                    [Sync ↻] [⋮] │   │
│ └──────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│ ┌─────────┐ ┌─────────────────────────────────────────────────────┐    │
│ │         │ │                                                     │    │
│ │  Tree   │ │  # Feature X PRD                                    │    │
│ │  View   │ │                                                     │    │
│ │         │ │  ## Overview                                        │    │
│ │ ▼ docs  │ │                                                     │    │
│ │   prds  │ │  This feature allows users to...                    │    │
│ │   └ x   │ │                                                     │    │
│ │   tasks │ │  ## Requirements                                    │    │
│ │         │ │                                                     │    │
│ │         │ │  - [ ] Task 1                                       │    │
│ │         │ │  - [ ] Task 2                                       │    │
│ │         │ │                                                     │    │
│ └─────────┘ └─────────────────────────────────────────────────────┘    │
│                                                                         │
│ ┌──────────────────────────────────────────────────────────────────┐   │
│ │ Synced with GitHub • Last sync: 2 min ago    [View on GitHub →]  │   │
│ └──────────────────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────────────────┘
```

### Key UI Components

```typescript
// Components to build with shadcn
const components = {
  // Layout
  'DocumentSidebar':     'Tree view of documents with folders',
  'DocumentBreadcrumb':  'Navigation breadcrumb',
  'DocumentToolbar':     'Actions: save, sync, history, settings',
  
  // Editor
  'DocumentEditor':      'BlockNote/Tiptap wrapper',
  'EditorToolbar':       'Formatting toolbar',
  'SlashMenu':           'Slash command menu',
  
  // Sync
  'SyncStatus':          'Sync status indicator',
  'ConflictDialog':      'Conflict resolution dialog',
  'DiffViewer':          'Side-by-side diff view',
  
  // History
  'VersionHistory':      'Version list sidebar',
  'VersionCompare':      'Compare two versions',
  
  // Meta
  'DocumentMeta':        'Type, tags, linked repo',
  'DocumentSettings':    'Document settings dialog',
};
```

---

## GitHub Integration Details

### Webhook Setup

```typescript
// Webhook handler
app.post('/api/webhooks/github', async (req, res) => {
  const signature = req.headers['x-hub-signature-256'];
  const event = req.headers['x-github-event'];
  
  // Verify signature
  if (!verifyWebhookSignature(req.body, signature, config.webhookSecret)) {
    return res.status(401).send('Invalid signature');
  }
  
  if (event === 'push') {
    const { repository, commits, ref } = req.body;
    const branch = ref.replace('refs/heads/', '');
    
    for (const commit of commits) {
      // Process added/modified markdown files
      const mdFiles = [...commit.added, ...commit.modified]
        .filter(f => f.endsWith('.md'));
      
      for (const file of mdFiles) {
        await syncFileFromGitHub(repository.full_name, branch, file);
      }
      
      // Handle deleted files
      for (const file of commit.removed.filter(f => f.endsWith('.md'))) {
        await markDocumentDeleted(repository.full_name, file);
      }
    }
  }
  
  res.status(200).send('OK');
});
```

### GitHub API Operations

```typescript
// Push document to GitHub
async function pushToGitHub(doc: Document): Promise<void> {
  const octokit = getOctokit(doc.workspace_id);
  const [owner, repo] = doc.github_repo.split('/');
  
  // Get current file (if exists)
  let currentSha: string | undefined;
  try {
    const { data } = await octokit.repos.getContent({
      owner, repo, path: doc.github_path,
    });
    currentSha = (data as { sha: string }).sha;
  } catch (e) {
    // File doesn't exist yet
  }
  
  // Check for conflicts
  if (currentSha && currentSha !== doc.github_sha) {
    throw new ConflictError('Remote has changed', { currentSha });
  }
  
  // Push update
  const { data } = await octokit.repos.createOrUpdateFileContents({
    owner, repo,
    path: doc.github_path,
    message: `Update ${doc.title}`,
    content: Buffer.from(doc.content).toString('base64'),
    sha: currentSha,
  });
  
  // Update local SHA
  await db.documents.update(doc.id, {
    github_sha: data.content.sha,
    sync_status: 'synced',
    last_synced_at: new Date(),
  });
}
```

---

## Implementation Phases

### Phase 1: Core Editor (Week 1-2)
- [ ] Set up BlockNote with shadcn integration
- [ ] Document CRUD API
- [ ] Basic document tree/sidebar
- [ ] Markdown import/export

### Phase 2: GitHub Sync (Week 3-4)
- [ ] GitHub OAuth integration
- [ ] Push to GitHub functionality
- [ ] Webhook handler for incoming changes
- [ ] Conflict detection and resolution UI

### Phase 3: Version History (Week 5)
- [ ] Version tracking on save
- [ ] Version history UI
- [ ] Diff viewer
- [ ] Version restore

### Phase 4: Agent Integration (Week 6)
- [ ] MCP tool definitions
- [ ] Agent API authentication
- [ ] Document search for agents
- [ ] Sync status in agent responses

### Phase 5: Polish (Week 7-8)
- [ ] Real-time sync status
- [ ] Bulk operations
- [ ] Document templates (PRD, Task, etc.)
- [ ] Export functionality

---

## Technical Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Editor | BlockNote | Notion-like UX, shadcn support, MIT license, open source |
| Database | Self-hosted PostgreSQL | Full-text search, JSONB, existing cluster infra, no external deps |
| Sync Strategy | Optimistic with conflict detection | Better UX than lock-based |
| Storage | DB primary, GitHub as sync target | Faster reads, offline capability |
| Real-time | WebSocket for sync status | Immediate feedback on changes |
| API | REST + MCP tools | Standard for UI, MCP for agents |
| Hosting | Self-hosted, open source only | No SaaS dependencies, full control |

---

## Security Considerations

1. **GitHub Tokens:** Store encrypted, scope to minimal permissions
2. **Webhook Verification:** Always verify GitHub webhook signatures  
3. **Access Control:** Per-workspace document permissions
4. **Audit Logging:** Track all document operations
5. **Content Sanitization:** Sanitize markdown on render (XSS prevention)

---

## Future Enhancements

- **Real-time Collaboration:** Yjs integration for multi-user editing
- **AI Features:** AI-assisted writing, summarization, PRD generation
- **Templates:** Pre-built templates for PRDs, specs, runbooks
- **Comments:** Inline commenting and discussions
- **Integrations:** Linear, Jira, Notion import

---

## References

- [BlockNote Documentation](https://www.blocknotejs.org/)
- [BlockNote shadcn Guide](https://www.blocknotejs.org/docs/getting-started/shadcn)
- [Tiptap Documentation](https://tiptap.dev/)
- [GitHub Contents API](https://docs.github.com/en/rest/repos/contents)
- [GitHub Webhooks](https://docs.github.com/en/webhooks)
- [Liveblocks Editor Comparison](https://liveblocks.io/blog/which-rich-text-editor-framework-should-you-choose-in-2025)


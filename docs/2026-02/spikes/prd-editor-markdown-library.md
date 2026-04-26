# Spike: PRD Editor Markdown Library Selection

**Date:** 2026-01-31  
**Author:** Pixel  
**Status:** Research / Awaiting Decision

## Context

The CTO App needs a markdown editor for PRD creation and editing with these requirements:

### Requirements
1. **Markdown-first** — Human-readable format, primary authoring experience
2. **JSON export** — Transform markdown → JSON for intake agent processing
3. **Model-driven creation** — AI generates initial PRDs in markdown
4. **Human editing** — Rich editing experience for manual refinement
5. **Import existing markdown** — Load .md files from disk/repos
6. **React integration** — Tauri + React stack

### Out of Scope
- Forking VS Code (too heavy)
- Building from scratch (reinventing the wheel)

---

## Options

### 1. Monaco Editor
**What:** The editor that powers VS Code, available as standalone library  
**Stack:** TypeScript, vanilla JS (React wrapper available)  
**License:** MIT

**Pros:**
- Industry-standard, battle-tested
- Excellent syntax highlighting, IntelliSense
- Native markdown support with preview
- Highly extensible (custom languages, themes, commands)
- Great TypeScript support
- Active Microsoft maintenance
- Can embed side-by-side preview

**Cons:**
- Large bundle size (~2-3MB minified)
- Not WYSIWYG — shows raw markdown
- Steeper learning curve for customization
- Overkill if we just want markdown editing

**Best For:** Code-focused PRDs, technical users comfortable with raw markdown

**Example:**
```tsx
import Editor from '@monaco-editor/react';

<Editor
  defaultLanguage="markdown"
  value={markdownContent}
  onChange={handleChange}
  theme="vs-dark"
/>
```

---

### 2. TipTap
**What:** Headless prosemirror-based WYSIWYG editor  
**Stack:** React, Vue, vanilla (framework-agnostic core)  
**License:** MIT

**Pros:**
- True WYSIWYG — see rendered markdown while editing
- Headless architecture — bring your own UI
- Excellent React integration via `@tiptap/react`
- Extensible via prosemirror plugins
- Collaborative editing support (Yjs integration)
- Markdown input/output via extensions
- Modern, actively maintained

**Cons:**
- Requires more setup (headless = you build the UI)
- Markdown export requires extension configuration
- Larger API surface to learn
- May feel "too rich" for simple markdown docs

**Best For:** Rich, document-focused PRDs with formatting toolbar

**Example:**
```tsx
import { useEditor, EditorContent } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import Markdown from '@tiptap/extension-markdown';

const editor = useEditor({
  extensions: [StarterKit, Markdown],
  content: markdownContent,
});

<EditorContent editor={editor} />
```

---

### 3. CodeMirror 6
**What:** Extensible code editor framework, successor to CodeMirror 5  
**Stack:** TypeScript, vanilla JS (React wrapper needed)  
**License:** MIT

**Pros:**
- Modern architecture (2021+ rewrite)
- Smaller bundle than Monaco (~500KB)
- Excellent markdown mode with syntax highlighting
- Highly performant (handles large documents)
- Plugin system for extensions
- Good mobile support

**Cons:**
- Lower-level API than Monaco
- Not as feature-rich out-of-box
- React integration requires wrapper library
- Documentation less comprehensive than Monaco

**Best For:** Lightweight markdown editing, performance-critical apps

**Example:**
```tsx
import CodeMirror from '@uiw/react-codemirror';
import { markdown } from '@codemirror/lang-markdown';

<CodeMirror
  value={markdownContent}
  extensions={[markdown()]}
  onChange={handleChange}
/>
```

---

### 4. Milkdown
**What:** Plugin-driven WYSIWYG markdown editor built on prosemirror  
**Stack:** React, Vue, vanilla (framework-agnostic)  
**License:** MIT

**Pros:**
- WYSIWYG with markdown syntax support
- Plugin architecture for extensibility
- Themes and customization out-of-box
- Good React integration via `@milkdown/react`
- Live markdown preview while typing
- Smaller than TipTap

**Cons:**
- Smaller community than TipTap/Monaco
- Less mature (2020+)
- Documentation could be better
- Fewer resources/examples

**Best For:** Balanced WYSIWYG + markdown syntax experience

**Example:**
```tsx
import { Editor } from '@milkdown/react';
import { commonmark } from '@milkdown/preset-commonmark';

<Editor
  content={markdownContent}
  config={(ctx) => ctx.use(commonmark)}
/>
```

---

### 5. Novel (NEW - 2024)
**What:** Notion-style WYSIWYG editor built on TipTap  
**Stack:** React, Next.js focused  
**License:** Apache 2.0

**Pros:**
- Beautiful Notion-like UI out-of-box
- Slash commands, AI integration primitives
- TipTap under the hood (proven tech)
- Markdown import/export built-in
- Very modern, active development
- Great for AI-assisted editing

**Cons:**
- Opinionated UI (may not fit our design)
- Newer (less battle-tested)
- Heavier than minimal editors
- Tailwind dependency

**Best For:** Notion-style PRD creation with AI features

**Repo:** https://github.com/steven-tey/novel

---

### 6. Lexical (NEW - Meta 2022)
**What:** Extensible text editor framework from Meta (powers Facebook, WhatsApp)  
**Stack:** React-first, TypeScript  
**License:** MIT

**Pros:**
- Meta-backed, production-proven
- React-first architecture
- Excellent accessibility
- Collaborative editing support
- Markdown plugin available
- Very performant

**Cons:**
- Lower-level framework (build your own UI)
- Still maturing (rapid API changes)
- Markdown support is plugin, not core
- Steeper learning curve

**Best For:** Long-term investment, custom editor requirements

**Repo:** https://github.com/facebook/lexical

---

## Comparison Matrix

| Feature | Monaco | TipTap | CodeMirror 6 | Milkdown | Novel | Lexical |
|---------|--------|--------|--------------|----------|-------|---------|
| **Bundle Size** | ~3MB | ~400KB | ~500KB | ~300KB | ~600KB | ~200KB |
| **WYSIWYG** | ❌ | ✅ | ❌ | ✅ | ✅ | ✅ |
| **Markdown Native** | ✅ | 🟡 (via ext) | ✅ | ✅ | ✅ | 🟡 (via plugin) |
| **React Integration** | 🟡 (wrapper) | ✅ | 🟡 (wrapper) | ✅ | ✅ | ✅ |
| **Customization** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Learning Curve** | Steep | Medium | Medium | Medium | Easy | Steep |
| **Maturity** | Very High | High | High | Medium | Low | Medium |
| **AI-Ready** | ❌ | ✅ | ❌ | 🟡 | ✅ | ✅ |

---

## Recommendation

### Top Pick: **TipTap**
**Rationale:**
- WYSIWYG fits "human-readable" requirement perfectly
- Markdown import/export via `@tiptap/extension-markdown`
- Headless = we control the UI (matches shadcn/Tauri design)
- Proven in production (GitLab, Substack use it)
- Good balance of features vs. complexity
- AI integration friendly (easy to inject model-generated content)

### Runner-up: **Novel**
**Rationale:**
- If we want Notion-style UX out-of-box
- Built on TipTap (same foundation)
- Already has AI primitives (slash commands, etc.)
- Trade-off: more opinionated UI

### Dark Horse: **Monaco**
**Rationale:**
- If we want a "code editor for PRDs" feel
- Users comfortable with raw markdown
- Side-by-side preview mode
- Trade-off: larger bundle, not WYSIWYG

---

## Next Steps

### Option A: Prototype with TipTap
1. Install `@tiptap/react`, `@tiptap/starter-kit`, `@tiptap/extension-markdown`
2. Build basic editor component in CTO App
3. Add markdown export to JSON for intake agent
4. Test import of existing .md files

### Option B: Prototype with Novel
1. Install `novel` package
2. Integrate into CTO App (may need to customize UI)
3. Test markdown export pipeline
4. Evaluate if opinionated UI fits our needs

### Option C: Spike Both
Build minimal prototypes of TipTap and Novel side-by-side, compare UX.

---

## Research Sources
- [Monaco Editor Docs](https://microsoft.github.io/monaco-editor/)
- [TipTap Docs](https://tiptap.dev/)
- [Novel GitHub](https://github.com/steven-tey/novel)
- [Lexical Docs](https://lexical.dev/)
- [CodeMirror 6 Docs](https://codemirror.net/)
- [Milkdown Docs](https://milkdown.dev/)

## Linear's Editor (Bonus Research)
Linear uses **ProseMirror** directly (TipTap is built on ProseMirror). They have a custom implementation with:
- Markdown input mode
- Slash commands
- Real-time collaboration
- Custom node types for Linear-specific features

This suggests **TipTap** or **Lexical** are the right tier of abstraction for our needs.

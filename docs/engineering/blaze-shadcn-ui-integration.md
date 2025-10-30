# Blaze + shadcn/ui Integration

**Status:** ✅ Complete  
**Date:** 2025-01-30

---

## Overview

Blaze is now configured to use shadcn/ui for all frontend UI generation. This document explains the architecture and how it works.

## Key Understanding: shadcn/ui is NOT npm

**Critical Concept:** shadcn/ui is NOT a component library installed from npm.

It's a **code distribution system** that COPIES production-ready component source code into your project:

```bash
npx shadcn@latest add button card form
# Creates these files in YOUR codebase:
# components/ui/button.tsx
# components/ui/card.tsx  
# components/ui/form.tsx
```

**You own the code.** Every component comes with:
- ✅ WCAG AA accessibility
- ✅ Dark mode support
- ✅ TypeScript types
- ✅ Responsive design
- ✅ Beautiful defaults

## Blaze's Role

**Blaze does NOT create components** - they already exist and are perfect.

**Blaze's job:**
1. **Identify** which components are needed from PRD
2. **Copy** them: `npx shadcn@latest add [components]`
3. **Compose** them into pages/features
4. **Add** business logic and data fetching

## Implementation Details

### 1. Design System Documentation

**Location:** `/infra/charts/controller/agent-templates/design-system.md`

Contains:
- Available components reference
- PRD → component mapping
- Composition patterns (dashboard, forms, tables, landing pages)
- Responsive design patterns
- Quality checklist

### 2. Agent Templates Updated

All 5 CLI templates updated with shadcn/ui understanding:

- `code/claude/agents-blaze.md.hbs` ✅
- `code/codex/agents-blaze.md.hbs` ✅
- `code/cursor/agents-blaze.md.hbs` ✅
- `code/factory/agents-blaze.md.hbs` ✅
- `code/opencode/agents-blaze.md.hbs` ✅

Each template now includes:
- Correct explanation of shadcn/ui (not npm)
- Reference to design system documentation
- Quick component reference
- PRD → component mapping

### 3. Technology Stack

**Framework:** Next.js 15 (App Router)  
**Language:** TypeScript 5 (strict mode)  
**Styling:** Tailwind CSS 4  
**Components:** shadcn/ui (copied into repo)  
**Forms:** React Hook Form + Zod  
**Icons:** lucide-react

## Example Workflow

### PRD Requirement
"User management with list, search, and add user functionality"

### Blaze Execution

**Step 1: Identify components needed**
- Table (user list)
- Input (search)
- Button (actions)
- Dialog (add user modal)
- Form (user form)
- Badge (status indicators)

**Step 2: Copy components**
```bash
npx shadcn@latest add table input button dialog form badge
```

**Step 3: Compose page**
```tsx
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Table } from "@/components/ui/table"
import { Dialog } from "@/components/ui/dialog"
import { Form } from "@/components/ui/form"
import { Badge } from "@/components/ui/badge"

export default function UsersPage() {
  return (
    <div className="container mx-auto px-4 py-8">
      {/* Compose components together */}
      <div className="flex justify-between mb-8">
        <h1>Users</h1>
        <Dialog>
          <Button>Add User</Button>
          {/* Form in dialog */}
        </Dialog>
      </div>
      
      <Input placeholder="Search..." />
      
      <Table>
        {/* User rows with Badge for status */}
      </Table>
    </div>
  )
}
```

**Step 4: Add business logic**
- Data fetching
- State management
- Form validation
- API calls

**Result:** Production-ready UI in minutes

## Component Reference

### Most Common Components

**Forms:**
```bash
npx shadcn@latest add button input textarea select checkbox radio-group switch form label
```

**Layout:**
```bash
npx shadcn@latest add card table separator tabs badge avatar
```

**Feedback:**
```bash
npx shadcn@latest add dialog alert-dialog toast alert skeleton
```

**Navigation:**
```bash
npx shadcn@latest add navigation-menu breadcrumb command dropdown-menu
```

**Advanced:**
```bash
npx shadcn@latest add data-table calendar date-picker chart
```

## PRD → Component Mapping

| PRD Requirement | shadcn/ui Command |
|-----------------|-------------------|
| Login/signup form | `add form input button` |
| User dashboard | `add card badge separator` |
| Data table | `add table input button` |
| Sortable table | `add data-table input` |
| Settings page | `add tabs form switch` |
| Landing page | `add button card` |
| Profile page | `add card avatar badge button` |
| Search palette | `add command` |
| Notifications | `add toast` |
| Confirmations | `add alert-dialog` |

## Quality Checklist

Blaze verifies before creating PR:

- ✅ Used shadcn/ui components (no custom CSS)
- ✅ Responsive: 375px (mobile), 768px (tablet), 1920px (desktop)
- ✅ Dark mode works
- ✅ TypeScript strict (no `any` types)
- ✅ Build succeeds: `pnpm build`
- ✅ No console errors

## Benefits

### For Development Speed
- **No design decisions needed** - components are beautiful by default
- **No accessibility work** - WCAG AA built-in
- **No dark mode complexity** - automatic with next-themes
- **No TypeScript setup** - strict types included

### For Code Quality
- **Production-ready from start** - no "TODO: make accessible" comments
- **Consistent UI** - all components match design system
- **Full customization** - source code in your repo
- **AI-friendly** - GPT-4 knows shadcn/ui patterns well

### For Maintenance
- **You own the code** - full control, no black boxes
- **Easy debugging** - see exactly what's happening
- **Simple updates** - just edit the component files
- **No version conflicts** - not a dependency

## Local Documentation

shadcn/ui documentation cloned to:
`/docs/ui/`

Components live at:
`/docs/ui/apps/v4/content/docs/components/`

Component examples at:
`/docs/ui/apps/v4/registry/new-york-v4/`

## Next Steps

When Blaze runs its first frontend task:

1. **Reads** PRD from task files
2. **References** design-system.md for component mapping
3. **Initializes** Next.js project
4. **Runs** `npx shadcn@latest init`
5. **Adds** needed components: `npx shadcn@latest add [components]`
6. **Composes** them into pages
7. **Adds** business logic
8. **Creates** PR with beautiful, accessible UI

---

**Key Takeaway:** Blaze focuses on **composition and business logic**, not component creation. shadcn/ui provides the perfect building blocks.


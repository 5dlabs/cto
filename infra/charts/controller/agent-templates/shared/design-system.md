# Blaze Design System

**Version:** 1.0  
**Framework:** Next.js 15 + React 19 + TypeScript 5  
**Components:** shadcn/ui (copied into your repo, not from npm)  
**Styling:** Tailwind CSS 4

---

## 🚫 CRITICAL RULE: NO Material-UI

**Material-UI (MUI) is PROHIBITED. Always use shadcn/ui for React components.**

### ✅ DO:
- Use `npx shadcn@latest add button card form` to install components
- Use shadcn/ui components exclusively for all UI elements
- Copy shadcn components into your repo (they're source code, not npm packages)

### ❌ NEVER:
- Install `@mui/material`, `@mui/core`, or any Material-UI packages
- Use Material-UI components like `<Button>`, `<Card>`, `<TextField>` from MUI
- Import from `@mui/*` or `material-ui` packages
- Use Material-UI styling solutions

**Enforcement:** Cleo (quality agent) will REJECT any PR containing Material-UI dependencies or imports.

---

## Core Concept

**shadcn/ui is NOT a component library you install from npm.**

It's a **code distribution system** - you copy production-ready component SOURCE CODE into your project:

```bash
# This COPIES components into components/ui/
npx shadcn@latest add button card form input table

# Components are now in YOUR codebase:
# components/ui/button.tsx
# components/ui/card.tsx
# components/ui/form.tsx
```

**You own the code.** Modify anything. Components come with:
- ✅ Accessibility (WCAG AA)
- ✅ Dark mode support
- ✅ TypeScript types
- ✅ Responsive design
- ✅ Beautiful defaults

## Blaze's Job

**NOT to create components** (they're already perfect)

**BUT to:**
1. Identify which components are needed from PRD
2. Copy them into project: `npx shadcn@latest add [components]`
3. Compose them into pages/features
4. Add business logic and data fetching

---

## Philosophy

Design principles:
- **Modern & Minimal** - shadcn/ui provides this by default
- **Accessible First** - Components are WCAG AA compliant out of box
- **Mobile First** - All components responsive by design
- **Dark Mode Native** - Automatic with next-themes
- **Composition Over Creation** - Assemble, don't build from scratch

---

## Quick Start

### 1. Initialize shadcn/ui (one-time)
```bash
npx shadcn@latest init --yes --defaults --base-color zinc
```

This creates:
- `components.json` (configuration)
- `lib/utils.ts` (utility functions)
- `globals.css` (CSS variables for theming)

### 2. Add components you need
```bash
# Identify from PRD, then copy into your project:
npx shadcn@latest add button card form input table dialog badge
```

### 3. Use them
```tsx
import { Button } from "@/components/ui/button"
import { Card } from "@/components/ui/card"

<Card>
  <Button>Click me</Button>
</Card>
```

**That's it.** Components are production-ready with dark mode, accessibility, TypeScript.

---

## Available Components

**All components from:** `/workspace/docs/ui/apps/v4/content/docs/components/`

### Most Common Components

**Add with:** `npx shadcn@latest add [component-names]`

**Forms & Input:**
```bash
npx shadcn@latest add button input textarea select checkbox radio-group switch form label
```
- `button` - Actions, CTAs
- `input` - Text, email, password, number
- `textarea` - Long text
- `select` - Dropdowns
- `checkbox` - Multiple selection
- `radio-group` - Single selection
- `switch` - On/off toggle
- `form` - Form wrapper with react-hook-form + zod

**Layout & Display:**
```bash
npx shadcn@latest add card table separator tabs badge avatar
```
- `card` - Content containers
- `table` - Data tables (basic)
- `separator` - Visual dividers
- `tabs` - Content organization
- `badge` - Status indicators, labels
- `avatar` - User profile images

**Feedback & Dialogs:**
```bash
npx shadcn@latest add dialog alert-dialog toast alert skeleton
```
- `dialog` - Modals (forms, confirmations)
- `alert-dialog` - Destructive action confirmations
- `toast` - Notifications (success/error messages)
- `alert` - Inline important messages
- `skeleton` - Loading placeholders

**Navigation:**
```bash
npx shadcn@latest add navigation-menu breadcrumb command dropdown-menu popover
```
- `navigation-menu` - Primary nav
- `breadcrumb` - Hierarchical navigation
- `command` - ⌘K search palette
- `dropdown-menu` - Action menus
- `popover` - Contextual content

**Advanced:**
```bash
npx shadcn@latest add data-table calendar date-picker chart
```
- `data-table` - Sortable/filterable tables
- `calendar` - Date selection
- `date-picker` - Date input
- `chart` - Data visualization

---

## Common UI Patterns

### Pattern 1: Dashboard Layout
```tsx
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"

export default function Dashboard() {
  return (
    <div className="container max-w-7xl mx-auto px-4 py-8">
      <div className="mb-8">
        <h1 className="text-3xl font-bold tracking-tight">Dashboard</h1>
        <p className="text-muted-foreground">Welcome back! Here's your overview.</p>
      </div>
      
      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader>
            <CardTitle>Total Revenue</CardTitle>
            <CardDescription>+20.1% from last month</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">$45,231.89</div>
          </CardContent>
        </Card>
        {/* More cards... */}
      </div>
    </div>
  )
}
```

### Pattern 2: Data Table with Actions
```tsx
import { Button } from "@/components/ui/button"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"

export default function DataTable() {
  return (
    <div className="container max-w-7xl mx-auto px-4 py-8">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-3xl font-bold">Users</h1>
        <Button>Add User</Button>
      </div>
      
      <div className="border rounded-lg">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>Email</TableHead>
              <TableHead>Role</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {/* Table rows... */}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
```

### Pattern 3: Form with Validation
```tsx
import { Button } from "@/components/ui/button"
import { Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from "@/components/ui/form"
import { Input } from "@/components/ui/input"
import { useForm } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import * as z from "zod"

const formSchema = z.object({
  email: z.string().email(),
  password: z.string().min(8),
})

export default function LoginForm() {
  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
  })

  return (
    <div className="container max-w-md mx-auto px-4 py-16">
      <div className="mb-8 text-center">
        <h1 className="text-3xl font-bold">Sign In</h1>
        <p className="text-muted-foreground">Enter your credentials to continue</p>
      </div>
      
      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
          <FormField
            control={form.control}
            name="email"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Email</FormLabel>
                <FormControl>
                  <Input placeholder="you@example.com" {...field} />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
          
          <FormField
            control={form.control}
            name="password"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Password</FormLabel>
                <FormControl>
                  <Input type="password" {...field} />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
          
          <Button type="submit" className="w-full">Sign In</Button>
        </form>
      </Form>
    </div>
  )
}
```

### Pattern 4: Hero Section
```tsx
import { Button } from "@/components/ui/button"

export default function Hero() {
  return (
    <section className="py-24 md:py-32 lg:py-40">
      <div className="container max-w-5xl mx-auto px-4 text-center">
        <h1 className="text-4xl md:text-5xl lg:text-6xl font-bold tracking-tight mb-6">
          Build Beautiful Apps Faster
        </h1>
        <p className="text-lg md:text-xl text-muted-foreground mb-8 max-w-2xl mx-auto">
          Production-ready components built with Radix UI and Tailwind CSS.
          Accessible, customizable, and open source.
        </p>
        <div className="flex flex-col sm:flex-row gap-4 justify-center">
          <Button size="lg">Get Started</Button>
          <Button size="lg" variant="outline">Learn More</Button>
        </div>
      </div>
    </section>
  )
}
```

---

## PRD → Component Mapping

**When PRD says...** → **Components to add + compose:**

| PRD Requirement | shadcn/ui add command | Composition |
|-----------------|----------------------|-------------|
| "User dashboard" | `add card badge separator` | Card grid with stats + Badge for statuses |
| "Login/signup form" | `add form input button` | Form + Input (email/password) + Button |
| "User list/table" | `add table input button dialog` | Table + search Input + "Add" Button + Dialog for create |
| "Data table (sortable)" | `add data-table input` | DataTable + Input for search |
| "Settings page" | `add tabs form input switch` | Tabs with Form sections + Input/Switch for settings |
| "Landing page" | `add button card` | Hero section + Card grid for features |
| "Profile page" | `add card avatar badge button` | Card + Avatar + Badge (status) + Button (edit) |
| "Search/command palette" | `add command` | Command (⌘K) component |
| "Notifications" | `add toast` | Toast system for alerts |
| "Confirmation dialogs" | `add alert-dialog button` | AlertDialog for destructive actions |
| "Create/edit forms" | `add form input textarea select button` | Form with validation + various inputs |
| "Navigation" | `add navigation-menu breadcrumb` | NavigationMenu for main nav |
| "Loading states" | `add skeleton` | Skeleton placeholders |

### Selection Rules

**Always use Form for forms:**
```bash
npx shadcn@latest add form
# Includes react-hook-form + zod validation
```

**For data tables:**
- Simple: `add table`
- With sorting/filtering: `add data-table`

**For user feedback:**
- Success/error: `add toast`
- Confirmations: `add alert-dialog`
- Important info: `add alert`

**For loading:**
- Use `add skeleton` and render during data fetch

---

## Responsive Patterns (Built into shadcn/ui)

shadcn/ui components are mobile-first by default. Just use Tailwind responsive classes:

```tsx
{/* Stack on mobile, row on desktop */}
<div className="flex flex-col md:flex-row gap-4">

{/* 1 column mobile, 2 tablet, 3 desktop */}
<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">

{/* Hide on small screens */}
<div className="hidden md:block">
```

---

## Accessibility (Built into shadcn/ui)

shadcn/ui components are WCAG AA compliant out of the box. Just ensure:

1. **Use semantic HTML** - `<button>` for actions, `<a>` for links
2. **Keep heading hierarchy** - h1 → h2 → h3 (not h1 → h3)
3. **Add aria-label to icon-only buttons**
4. **All forms use FormLabel** (included in Form component)

The components handle the rest (keyboard nav, focus indicators, ARIA attributes).

---

## Quality Checklist

Before creating PR:

- ✅ Used shadcn/ui components (not custom CSS)
- ✅ Test responsive: 375px (mobile), 768px (tablet), 1920px (desktop)
- ✅ Toggle dark mode - everything works
- ✅ TypeScript strict (no `any` types)
- ✅ Build succeeds: `pnpm build`
- ✅ No console errors

---

## Example: User Management Page

**PRD:** "User management with list, search, and add user"

### Step 1: Add components
```bash
npx shadcn@latest add table input button dialog badge form
```

### Step 2: Compose them
```tsx
// app/users/page.tsx - COMPOSING shadcn/ui components
import { Button } from "@/components/ui/button"  // ← These exist in your
import { Input } from "@/components/ui/input"    // ← codebase after Step 1
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Badge } from "@/components/ui/badge"
import { Dialog, DialogTrigger } from "@/components/ui/dialog"

export default function UsersPage() {
  return (
    <div className="container max-w-7xl mx-auto px-4 py-8">
      {/* Header with title + Add button */}
      <div className="flex justify-between items-center mb-8">
        <div>
          <h1 className="text-3xl font-bold">Users</h1>
          <p className="text-muted-foreground">Manage team members</p>
        </div>
        
        <Dialog>
          <DialogTrigger asChild>
            <Button>Add User</Button>
          </DialogTrigger>
          {/* Dialog content with Form... */}
        </Dialog>
      </div>
      
      {/* Search bar */}
      <div className="mb-6 max-w-sm">
        <Input placeholder="Search users..." />
      </div>
      
      {/* User table */}
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Email</TableHead>
            <TableHead>Status</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow>
            <TableCell>John Doe</TableCell>
            <TableCell>john@example.com</TableCell>
            <TableCell><Badge>Active</Badge></TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </div>
  )
}
```

**Result:** Beautiful, accessible, dark-mode-ready UI in minutes.

---

**Key Insight:** Blaze COMPOSES shadcn/ui components, doesn't create them from scratch.



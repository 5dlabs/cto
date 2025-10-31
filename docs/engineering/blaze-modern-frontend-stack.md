# Blaze Modern Frontend Stack

**Date:** 2025-10-31  
**Agent:** Blaze (Frontend Specialist)  
**Status:** Production Configuration  
**Purpose:** Modern, accessible, production-ready UI implementation

---

## Stack Overview

### Core Technologies

| Component | Technology | Version | Purpose |
|-----------|-----------|---------|---------|
| **Framework** | Next.js | 15+ | React framework with App Router |
| **Runtime** | Node.js | 20+ | JavaScript runtime |
| **Language** | TypeScript | 5+ | Type-safe development |
| **Styling** | Tailwind CSS | 4+ | Utility-first CSS |
| **Components** | shadcn/ui | Latest | Copy-paste component system |
| **Package Manager** | pnpm | Latest | Fast, disk-efficient package manager |
| **JavaScript Runtime** | Bun | Latest | Fast all-in-one toolkit |
| **Forms** | React Hook Form + Zod | Latest | Type-safe form validation |
| **Icons** | lucide-react | Latest | Beautiful icon library |
| **Testing** | Vitest + React Testing Library | Latest | Fast unit testing |

---

## Why Modern Tooling?

### pnpm (Package Manager)

**Benefits:**
- ✅ **3x faster** than npm for installations
- ✅ **Up to 90% disk space savings** (content-addressable storage)
- ✅ **Strict dependency resolution** (no phantom dependencies)
- ✅ **Monorepo-friendly** (built-in workspace support)
- ✅ **Production-ready** (used by Vue, Vite, Microsoft)

**Usage:**
```bash
# Install dependencies
pnpm install

# Add package
pnpm add react-hook-form

# Run scripts
pnpm dev
pnpm build
pnpm test
```

### Bun (JavaScript Runtime/Toolkit)

**Benefits:**
- ✅ **3-4x faster** than Node.js for many operations
- ✅ **Built-in bundler, test runner, package manager**
- ✅ **Drop-in replacement** for node/npm/npx
- ✅ **Native TypeScript** support (no ts-node needed)
- ✅ **Fast test runner** (faster than Jest/Vitest)

**Usage:**
```bash
# Run TypeScript directly
bun run src/index.ts

# Run tests
bun test

# Install packages (alternative to pnpm)
bun install
```

**When to Use:**
- Development scripts
- Test execution
- Build tools
- Local development (faster HMR)

**When Not to Use (Yet):**
- Production deployments (still stabilizing)
- Complex Node.js integrations
- Libraries with native bindings

---

## shadcn/ui Component System

### What is shadcn/ui?

**NOT a component library** - It's a code distribution system.

**Key Concept:**
```bash
# This COPIES source code to your project
npx shadcn@latest add button card form

# Result: Files created in YOUR codebase
components/ui/button.tsx
components/ui/card.tsx
components/ui/form.tsx
```

**You own the code** - Customize however you want!

### Why shadcn/ui?

| Benefit | Traditional Library | shadcn/ui |
|---------|-------------------|-----------|
| **Ownership** | Black box in node_modules | Full source code in your repo |
| **Customization** | Limited (props/theming only) | Unlimited (edit source) |
| **Bundle Size** | Import entire library | Only what you use |
| **Accessibility** | Varies | WCAG AA compliant |
| **Dark Mode** | Manual implementation | Built-in |
| **TypeScript** | May lack types | Fully typed |
| **Dependencies** | Many (transitive hell) | Minimal (Radix UI + Tailwind) |

### Available Components

**Forms & Input:**
```bash
npx shadcn@latest add button input textarea select checkbox radio-group switch form label
```
- `button` - Actions, CTAs
- `input` - Text fields
- `form` - React Hook Form + Zod integration
- `select` - Dropdowns
- `checkbox`, `radio-group`, `switch` - Selections

**Layout:**
```bash
npx shadcn@latest add card table separator tabs badge avatar
```
- `card` - Content containers
- `table` - Data display
- `separator` - Visual dividers
- `tabs` - Content organization

**Feedback:**
```bash
npx shadcn@latest add dialog alert-dialog toast alert skeleton
```
- `dialog` - Modals
- `alert-dialog` - Confirmations
- `toast` - Notifications
- `skeleton` - Loading states

**Navigation:**
```bash
npx shadcn@latest add navigation-menu breadcrumb command dropdown-menu
```
- `navigation-menu` - Primary nav
- `command` - ⌘K search palette
- `dropdown-menu` - Action menus

**Advanced:**
```bash
npx shadcn@latest add data-table calendar date-picker chart
```
- `data-table` - Sortable/filterable tables
- `calendar` - Date selection
- `chart` - Data visualization

### shadcn MCP Server Integration

**NEW: MCP Server for Component Management**

shadcn/ui now provides an MCP server that allows AI agents to browse, search, and install components programmatically.

**Configuration:**
```json
{
  "agents": {
    "blaze": {
      "tools": {
        "localServers": {
          "shadcn": {
            "enabled": true,
            "command": "npx",
            "args": ["shadcn@latest", "mcp"],
            "tools": [
              "list_components",
              "search_components", 
              "install_component",
              "init_project"
            ]
          }
        }
      }
    }
  }
}
```

**MCP Tools:**

1. **`list_components`** - Browse all available components
   ```javascript
   // Returns: Array of {name, description, dependencies}
   ```

2. **`search_components`** - Find components by name/functionality
   ```javascript
   // Query: "form validation"
   // Returns: form, input, select, checkbox components
   ```

3. **`install_component`** - Add component to project
   ```javascript
   // Installs: button, dialog, card, etc.
   // Handles: Dependencies, configuration, imports
   ```

4. **`init_project`** - Initialize shadcn/ui in project
   ```javascript
   // Creates: components.json, lib/utils.ts, globals.css
   // Configures: Tailwind, imports, aliases
   ```

**Benefits for Blaze:**
- ✅ **Autonomous component discovery** - No manual documentation lookup
- ✅ **Dependency awareness** - Knows what components need what
- ✅ **Automatic installation** - No copy-paste errors
- ✅ **Registry support** - Works with public and private registries

---

## Documentation Integration

### Local Documentation

**shadcn/ui Documentation:**
```bash
# Clone for offline reference
git clone https://github.com/shadcn-ui/ui.git /workspace/docs/ui

# Component docs location:
/workspace/docs/ui/apps/www/content/docs/components/

# Component examples:
/workspace/docs/ui/apps/www/registry/new-york-v4/
```

### Doc Server Ingestion

**Ingest shadcn/ui docs into doc server for AI retrieval:**

```bash
# Using docs_ingest MCP tool
docs_ingest({
  repository_url: "https://github.com/shadcn-ui/ui",
  doc_type: "shadcn-ui"
})
```

**What gets ingested:**
- Component API documentation
- Usage examples
- Composition patterns
- Theming guide
- Accessibility guidelines

**How Blaze uses it:**
```javascript
// Morgan (docs agent) creates component guide referencing doc server
// Blaze queries doc server for specific component patterns
context7_get_library_docs({
  library: "shadcn-ui",
  query: "form validation with zod"
})
```

**Benefits:**
- ✅ Always up-to-date component information
- ✅ Semantic search (finds relevant patterns)
- ✅ No hallucinated component APIs
- ✅ Example code from official docs

---

## Blaze Implementation Workflow

### 1. Project Initialization

```bash
# Create Next.js project with pnpm
pnpm create next-app@latest frontend \
  --typescript \
  --tailwind \
  --app \
  --no-src-dir \
  --import-alias "@/*" \
  --use-pnpm \
  --yes

cd frontend

# Initialize shadcn/ui
pnpm dlx shadcn@latest init --yes --defaults
```

### 2. Component Selection (Via MCP)

**Blaze uses MCP tools to discover and install components:**

```javascript
// Example: Blaze implementing a dashboard

// 1. Search for relevant components
search_components({ query: "dashboard cards metrics" })
// Returns: card, separator, badge, skeleton

// 2. Install components
install_component({ components: ["card", "separator", "badge", "skeleton"] })

// 3. Build dashboard layout
// Blaze composes components into pages
```

### 3. Development

**Using pnpm:**
```bash
# Install dependencies
pnpm install react-hook-form zod @hookform/resolvers

# Development server
pnpm dev

# Production build
pnpm build

# Run tests
pnpm test
```

**Using Bun (faster):**
```bash
# Run Next.js dev server with Bun
bun --bun next dev

# Run tests with Bun
bun test

# Type checking
bun run typecheck
```

### 4. Quality Checks

**Pre-PR Validation:**
```bash
# Type checking
pnpm tsc --noEmit

# Linting
pnpm lint

# Format check
pnpm prettier --check .

# Build check
pnpm build

# Run all checks
pnpm run validate
```

**package.json scripts:**
```json
{
  "scripts": {
    "dev": "next dev",
    "build": "next build",
    "start": "next start",
    "lint": "next lint",
    "typecheck": "tsc --noEmit",
    "format": "prettier --write .",
    "format:check": "prettier --check .",
    "validate": "pnpm typecheck && pnpm lint && pnpm format:check && pnpm build"
  }
}
```

---

## Component Composition Patterns

### Dashboard Layout

```typescript
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Separator } from "@/components/ui/separator"
import { Badge } from "@/components/ui/badge"

export default function Dashboard() {
  return (
    <div className="container max-w-7xl mx-auto px-4 py-8">
      <div className="mb-8">
        <h1 className="text-3xl font-bold">Dashboard</h1>
        <p className="text-muted-foreground">Welcome back!</p>
      </div>
      
      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader>
            <CardTitle>Total Revenue</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">$45,231</div>
            <Badge className="mt-2">+20.1%</Badge>
          </CardContent>
        </Card>
        {/* More cards... */}
      </div>
    </div>
  )
}
```

### Form with Validation

```typescript
import { Button } from "@/components/ui/button"
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from "@/components/ui/form"
import { Input } from "@/components/ui/input"
import { useForm } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import * as z from "zod"

const formSchema = z.object({
  email: z.string().email("Invalid email address"),
  password: z.string().min(8, "Password must be at least 8 characters"),
})

export default function LoginForm() {
  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
  })

  const onSubmit = async (data: z.infer<typeof formSchema>) => {
    // Handle form submission
  }

  return (
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
  )
}
```

### Data Table

```typescript
import { Button } from "@/components/ui/button"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Badge } from "@/components/ui/badge"

export default function UsersTable() {
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
              <TableHead>Status</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow>
              <TableCell>John Doe</TableCell>
              <TableCell>john@example.com</TableCell>
              <TableCell><Badge>Active</Badge></TableCell>
            </TableRow>
            {/* More rows... */}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
```

---

## Responsive Design

**Mobile-First Approach:**
```typescript
<div className="
  grid 
  grid-cols-1        /* Mobile: 1 column */
  md:grid-cols-2     /* Tablet: 2 columns */
  lg:grid-cols-4     /* Desktop: 4 columns */
  gap-6 
  p-4
">
  {/* Content */}
</div>
```

**Breakpoint Reference:**
| Breakpoint | Width | Modifier |
|------------|-------|----------|
| Mobile | < 768px | (default) |
| Tablet | ≥ 768px | `md:` |
| Desktop | ≥ 1024px | `lg:` |
| Large Desktop | ≥ 1280px | `xl:` |

---

## Accessibility

**Built-in WCAG AA Compliance:**

shadcn/ui components are accessible by default:
- ✅ Keyboard navigation
- ✅ Screen reader support
- ✅ Focus indicators
- ✅ ARIA attributes
- ✅ Semantic HTML

**What Blaze Must Ensure:**
- Use semantic HTML (`<button>` not `<div>`)
- Maintain heading hierarchy (h1 → h2 → h3)
- Add `aria-label` to icon-only buttons
- Ensure sufficient color contrast (handled by Tailwind tokens)

---

## Performance

### Code Splitting

**Next.js 15 handles this automatically:**
```typescript
// Lazy load heavy components
import dynamic from 'next/dynamic'

const HeavyChart = dynamic(() => import('@/components/chart'), {
  loading: () => <Skeleton className="h-64" />,
  ssr: false // Client-only if needed
})
```

### Image Optimization

```typescript
import Image from 'next/image'

<Image
  src="/hero.png"
  alt="Hero image"
  width={1200}
  height={630}
  priority // Above the fold
/>
```

### Font Optimization

```typescript
import { Inter } from 'next/font/google'

const inter = Inter({
  subsets: ['latin'],
  display: 'swap',
})

export default function RootLayout({ children }) {
  return (
    <html lang="en" className={inter.className}>
      {children}
    </html>
  )
}
```

---

## Environment Configuration

**Modern JavaScript Features:**

```json
// package.json
{
  "engines": {
    "node": ">=20.0.0",
    "pnpm": ">=8.0.0"
  },
  "packageManager": "pnpm@9.0.0"
}
```

**TypeScript Configuration:**

```json
// tsconfig.json
{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "paths": {
      "@/*": ["./*"]
    }
  }
}
```

---

## Blaze Agent Instructions

**When implementing frontend tasks:**

1. **Use pnpm** for package management (3x faster than npm)
2. **Use bun** for running scripts/tests when speed matters
3. **Use shadcn MCP** to discover and install components
4. **Reference doc server** for shadcn/ui usage patterns
5. **Compose, don't create** - assemble shadcn components
6. **Validate quality** - run typecheck, lint, build before PR
7. **No mocks** - use real API endpoints (configurable via env)
8. **Mobile-first** - test at 375px, 768px, 1920px
9. **Accessibility** - use semantic HTML, ARIA labels
10. **Document** - add JSDoc to custom components

---

## References

- [shadcn/ui Documentation](https://ui.shadcn.com)
- [shadcn/ui GitHub](https://github.com/shadcn-ui/ui)
- [shadcn/ui MCP Server](https://ui.shadcn.com/docs/mcp-server)
- [Next.js 15 Documentation](https://nextjs.org/docs)
- [pnpm Documentation](https://pnpm.io)
- [Bun Documentation](https://bun.sh)
- [Tailwind CSS Documentation](https://tailwindcss.com)
- [React Hook Form](https://react-hook-form.com)
- [Zod Validation](https://zod.dev)


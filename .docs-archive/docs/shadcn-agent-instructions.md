# shadcn/ui Tools - Agent Instructions for Blaze

## Overview

You have access to **shadcn/ui MCP tools** that provide instant access to production-ready React components with TypeScript support. These tools give you the latest shadcn/ui v4 components, blocks, and demos.

## Available Tools

### Component Tools

1. **`shadcn_list_components`** - List all 54 available shadcn/ui components
2. **`shadcn_get_component`** - Get source code for a specific component
3. **`shadcn_get_component_demo`** - Get demo/example code for a component
4. **`shadcn_get_component_metadata`** - Get metadata about a component

### Block Tools

5. **`shadcn_list_blocks`** - List all pre-built UI blocks (dashboards, login pages, etc.)
6. **`shadcn_get_block`** - Get complete source code for a block

### Utility Tools

7. **`shadcn_get_directory_structure`** - Browse the shadcn/ui repository structure

## When to Use shadcn Tools

✅ **USE shadcn when:**
- Building React/Next.js UI components
- Need production-ready, accessible components
- Want TypeScript-first component examples
- Building forms, dashboards, or authentication UIs
- Need consistent, modern design patterns

❌ **DON'T use shadcn when:**
- Building backend/API code
- Working with non-React frameworks
- Need custom, non-standard UI patterns

## Usage Workflows

### Workflow 1: Discover and Use a Component

**Step 1: List available components**
```javascript
shadcn_list_components()
```
Returns all 54 components: accordion, alert, button, card, dialog, form, etc.

**Step 2: Get component source**
```javascript
shadcn_get_component({ componentName: "button" })
```
Returns the complete TypeScript source code.

**Step 3: Get usage examples**
```javascript
shadcn_get_component_demo({ componentName: "button" })
```
Returns demo code showing how to use the component.

**Step 4: Implement in your project**
Copy the component code and adapt it to your needs.

### Workflow 2: Use Pre-Built Blocks

**Step 1: List available blocks**
```javascript
shadcn_list_blocks()
```
Returns blocks by category: calendar, dashboard, login, sidebar, products.

**Step 2: Get a complete block**
```javascript
shadcn_get_block({ blockName: "dashboard-01" })
```
Returns the complete dashboard block with all components.

**Step 3: Integrate into your project**
Use the block as a starting point for your UI.

### Workflow 3: Build a Form

**Step 1: Get form component**
```javascript
shadcn_get_component({ componentName: "form" })
```

**Step 2: Get input components**
```javascript
shadcn_get_component({ componentName: "input" })
shadcn_get_component({ componentName: "button" })
shadcn_get_component({ componentName: "label" })
```

**Step 3: Get form demo**
```javascript
shadcn_get_component_demo({ componentName: "form" })
```

**Step 4: Build your form**
Combine components based on the demo patterns.

## Component Categories

### Form Components
- `input`, `textarea`, `select`, `checkbox`, `radio-group`, `switch`
- `form`, `label`, `field`, `input-group`, `input-otp`

### Layout Components
- `card`, `separator`, `tabs`, `accordion`, `collapsible`
- `sidebar`, `sheet`, `drawer`, `resizable`

### Feedback Components
- `alert`, `alert-dialog`, `dialog`, `toast`, `sonner`
- `progress`, `spinner`, `skeleton`

### Navigation Components
- `button`, `button-group`, `dropdown-menu`, `navigation-menu`
- `breadcrumb`, `pagination`, `menubar`, `context-menu`

### Data Display
- `table`, `badge`, `avatar`, `calendar`, `chart`
- `hover-card`, `popover`, `tooltip`

## Best Practices

### ✅ DO:

1. **List components first** to see what's available
2. **Get demos** to understand usage patterns
3. **Use TypeScript** - all components are TypeScript-first
4. **Follow shadcn patterns** - components are designed to work together
5. **Check metadata** for dependencies and requirements

### ❌ DON'T:

1. **Don't modify core component logic** - customize via props instead
2. **Don't skip demos** - they show best practices
3. **Don't mix with other UI libraries** - stay consistent with shadcn
4. **Don't forget accessibility** - shadcn components are accessible by default

## Example: Building a Login Page

```javascript
// Step 1: List blocks to see if there's a pre-built login
shadcn_list_blocks({ category: "login" })
// Returns: login-01, login-02, login-03, login-04, login-05

// Step 2: Get a login block
shadcn_get_block({ blockName: "login-01" })
// Returns complete login page with form, validation, etc.

// Step 3: Customize as needed
// Use the block as a starting point and adapt to your requirements
```

## Example: Building a Dashboard

```javascript
// Step 1: Get dashboard block
shadcn_get_block({ blockName: "dashboard-01" })
// Returns complete dashboard with sidebar, charts, metrics

// Step 2: Get additional components
shadcn_get_component({ componentName: "chart" })
shadcn_get_component({ componentName: "card" })

// Step 3: Get demos for complex components
shadcn_get_component_demo({ componentName: "chart" })

// Step 4: Build your dashboard
// Combine block and components
```

## Example: Building a Form

```javascript
// Step 1: Get form components
shadcn_get_component({ componentName: "form" })
shadcn_get_component({ componentName: "input" })
shadcn_get_component({ componentName: "button" })

// Step 2: Get form demo
shadcn_get_component_demo({ componentName: "form" })
// Shows react-hook-form integration

// Step 3: Implement your form
// Follow the demo pattern with your fields
```

## Integration with Context7

Combine shadcn with Context7 for complete documentation:

```javascript
// Use shadcn for component source
shadcn_get_component({ componentName: "dialog" })

// Use Context7 for library documentation
resolve_library_id({ libraryName: "react-hook-form" })
get_library_docs({
  context7CompatibleLibraryID: "/react-hook-form/react-hook-form",
  topic: "form validation with TypeScript"
})
```

## Quick Reference

### Most Common Components

**Forms:**
```javascript
shadcn_get_component({ componentName: "form" })
shadcn_get_component({ componentName: "input" })
shadcn_get_component({ componentName: "button" })
shadcn_get_component({ componentName: "select" })
```

**Layouts:**
```javascript
shadcn_get_component({ componentName: "card" })
shadcn_get_component({ componentName: "tabs" })
shadcn_get_component({ componentName: "sidebar" })
```

**Dialogs:**
```javascript
shadcn_get_component({ componentName: "dialog" })
shadcn_get_component({ componentName: "alert-dialog" })
shadcn_get_component({ componentName: "sheet" })
```

**Data Display:**
```javascript
shadcn_get_component({ componentName: "table" })
shadcn_get_component({ componentName: "chart" })
shadcn_get_component({ componentName: "badge" })
```

### Pre-Built Blocks

**Authentication:**
```javascript
shadcn_list_blocks({ category: "login" })
shadcn_get_block({ blockName: "login-01" })
```

**Dashboards:**
```javascript
shadcn_get_block({ blockName: "dashboard-01" })
```

**Sidebars:**
```javascript
shadcn_list_blocks({ category: "sidebar" })
shadcn_get_block({ blockName: "sidebar-01" })
```

## Component Metadata

Use metadata to understand component requirements:

```javascript
shadcn_get_component_metadata({ componentName: "form" })
```

Returns:
- Dependencies required
- Peer dependencies
- Installation instructions
- Related components

## Tips for Blaze

1. **Start with blocks** - Check if there's a pre-built block for your use case
2. **Get demos** - Always check demos to understand usage patterns
3. **Use TypeScript** - All components have full TypeScript support
4. **Combine components** - shadcn components are designed to work together
5. **Check metadata** - Understand dependencies before implementing
6. **Follow patterns** - shadcn has established patterns for forms, dialogs, etc.

## Summary

shadcn/ui tools provide:
- ✅ 54 production-ready components
- ✅ 66+ pre-built UI blocks
- ✅ Complete source code (not just docs)
- ✅ TypeScript-first examples
- ✅ Accessible by default
- ✅ Customizable via props and CSS

**Always check shadcn first** when building React/Next.js UI!

---

**Quick Start:**
```javascript
// 1. List what's available
shadcn_list_components()

// 2. Get component + demo
shadcn_get_component({ componentName: "button" })
shadcn_get_component_demo({ componentName: "button" })

// 3. Implement in your project
```


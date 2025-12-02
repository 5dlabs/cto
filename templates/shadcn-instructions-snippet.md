# shadcn/ui Tools

You have access to **shadcn/ui MCP tools** for production-ready React components.

## Available Tools

- `shadcn_list_components` - List all 54 components
- `shadcn_get_component` - Get component source code
- `shadcn_get_component_demo` - Get usage examples
- `shadcn_get_component_metadata` - Get component info
- `shadcn_list_blocks` - List pre-built UI blocks (dashboards, login pages, etc.)
- `shadcn_get_block` - Get complete block source code
- `shadcn_get_directory_structure` - Browse repository

## Quick Start Workflow

**For Components:**
```javascript
// 1. List available components
shadcn_list_components()

// 2. Get component source
shadcn_get_component({ componentName: "button" })

// 3. Get usage demo
shadcn_get_component_demo({ componentName: "button" })
```

**For Pre-Built Blocks:**
```javascript
// 1. List blocks by category
shadcn_list_blocks({ category: "login" })

// 2. Get complete block
shadcn_get_block({ blockName: "login-01" })
```

## Common Components

**Forms:** `form`, `input`, `button`, `select`, `checkbox`, `textarea`  
**Layouts:** `card`, `tabs`, `sidebar`, `sheet`, `dialog`  
**Data:** `table`, `chart`, `badge`, `avatar`  
**Navigation:** `dropdown-menu`, `navigation-menu`, `breadcrumb`

## Best Practices

✅ **DO:**
- Check for pre-built blocks first (dashboards, login pages)
- Get demos to understand usage patterns
- Use TypeScript - all components are TypeScript-first
- Combine components - they're designed to work together

❌ **DON'T:**
- Skip the demo step - demos show best practices
- Modify core component logic - customize via props
- Mix with other UI libraries - stay consistent

## Example: Build a Login Page

```javascript
// 1. Check for pre-built login blocks
shadcn_list_blocks({ category: "login" })

// 2. Get a login block
shadcn_get_block({ blockName: "login-01" })

// 3. Customize as needed
```

## Example: Build a Form

```javascript
// 1. Get form components
shadcn_get_component({ componentName: "form" })
shadcn_get_component_demo({ componentName: "form" })

// 2. Get input components
shadcn_get_component({ componentName: "input" })
shadcn_get_component({ componentName: "button" })

// 3. Implement following demo pattern
```

**Remember:** Always get the demo to see usage patterns!


# Blaze Theming & Styling Strategy

## Overview

Blaze uses a flexible, multi-layered theming system that allows customization at multiple levels while providing sensible defaults.

## Theme Hierarchy (Priority Order)

1. **Task-Level Overrides** (highest priority) - Specified in GitHub issue
2. **Repository Design System** - `.blaze/design-system.json` in repo
3. **Organization Defaults** - Configured in Helm values
4. **Blaze Built-in Default** (lowest priority) - shadcn/ui defaults

---

## 1. Built-in Default Theme

Blaze ships with a modern, professional default theme based on shadcn/ui:

```json
{
  "name": "Blaze Default Theme",
  "description": "Modern, accessible theme based on shadcn/ui defaults",
  "version": "1.0",
  
  "colors": {
    "primary": {
      "light": "#3B82F6",
      "dark": "#60A5FA",
      "foreground": "#FFFFFF"
    },
    "secondary": {
      "light": "#64748B",
      "dark": "#94A3B8",
      "foreground": "#FFFFFF"
    },
    "accent": {
      "light": "#10B981",
      "dark": "#34D399",
      "foreground": "#FFFFFF"
    },
    "background": {
      "light": "#FFFFFF",
      "dark": "#0F172A"
    },
    "foreground": {
      "light": "#0F172A",
      "dark": "#F8FAFC"
    },
    "muted": {
      "light": "#F1F5F9",
      "dark": "#1E293B"
    },
    "border": {
      "light": "#E2E8F0",
      "dark": "#334155"
    }
  },
  
  "typography": {
    "fontFamily": {
      "sans": "Inter, system-ui, -apple-system, sans-serif",
      "heading": "Inter, system-ui, -apple-system, sans-serif",
      "mono": "JetBrains Mono, Menlo, monospace"
    },
    "fontSize": {
      "xs": "0.75rem",
      "sm": "0.875rem",
      "base": "1rem",
      "lg": "1.125rem",
      "xl": "1.25rem",
      "2xl": "1.5rem",
      "3xl": "1.875rem",
      "4xl": "2.25rem",
      "5xl": "3rem"
    },
    "fontWeight": {
      "normal": 400,
      "medium": 500,
      "semibold": 600,
      "bold": 700
    },
    "lineHeight": {
      "tight": 1.25,
      "normal": 1.5,
      "relaxed": 1.75
    }
  },
  
  "spacing": {
    "scale": [0, 4, 8, 12, 16, 20, 24, 32, 40, 48, 64, 80, 96, 128],
    "containerMaxWidth": "1280px",
    "containerPadding": "1rem"
  },
  
  "borderRadius": {
    "none": "0",
    "sm": "0.25rem",
    "default": "0.5rem",
    "md": "0.5rem",
    "lg": "0.75rem",
    "xl": "1rem",
    "2xl": "1.5rem",
    "full": "9999px"
  },
  
  "shadows": {
    "sm": "0 1px 2px 0 rgb(0 0 0 / 0.05)",
    "default": "0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)",
    "md": "0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)",
    "lg": "0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)",
    "xl": "0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)"
  },
  
  "animation": {
    "duration": {
      "fast": "150ms",
      "normal": "300ms",
      "slow": "500ms"
    },
    "easing": {
      "default": "cubic-bezier(0.4, 0, 0.2, 1)",
      "in": "cubic-bezier(0.4, 0, 1, 1)",
      "out": "cubic-bezier(0, 0, 0.2, 1)",
      "inOut": "cubic-bezier(0.4, 0, 0.2, 1)"
    }
  }
}
```

---

## 2. Repository Design System

Create `.blaze/design-system.json` in your repository to define project-specific theming.

### Example: SaaS Application

```json
{
  "name": "MyApp Design System",
  "extends": "blaze-default",
  "version": "1.0",
  
  "colors": {
    "primary": {
      "light": "#6366F1",
      "dark": "#818CF8",
      "foreground": "#FFFFFF"
    },
    "brand": {
      "logo": "#6366F1",
      "accent": "#EC4899"
    }
  },
  
  "typography": {
    "fontFamily": {
      "sans": "Outfit, Inter, sans-serif",
      "heading": "Outfit, Inter, sans-serif"
    }
  },
  
  "spacing": {
    "containerMaxWidth": "1400px"
  },
  
  "brandGuidelines": {
    "tone": "Professional, friendly, innovative",
    "personality": "Modern tech company",
    "doNot": [
      "Use rounded corners > 1rem",
      "Use neon or overly bright colors",
      "Use shadows > lg"
    ]
  }
}
```

### Example: E-Commerce Site

```json
{
  "name": "Shop Design System",
  "extends": "blaze-default",
  
  "colors": {
    "primary": {
      "light": "#0F172A",
      "dark": "#1E293B"
    },
    "accent": {
      "light": "#F59E0B",
      "dark": "#FCD34D"
    }
  },
  
  "components": {
    "Button": {
      "defaultVariant": "solid",
      "primaryStyle": {
        "background": "black",
        "color": "white",
        "hover": "scale(1.02)"
      }
    },
    "ProductCard": {
      "imageAspectRatio": "3/4",
      "showQuickView": true,
      "hoverEffect": "lift"
    }
  }
}
```

---

## 3. Task-Level Overrides

Specify design preferences directly in the GitHub issue:

### Example: Minimal Format

```markdown
Title: Create pricing page

Labels: agent-blaze

---

## Requirements
- 3-tier pricing table
- Feature comparison
- FAQ section

## Theme
- Colors: Purple (#8B5CF6) primary, pink accent
- Style: Clean, modern, generous spacing
- Typography: Poppins font
- Buttons: Large, rounded
```

### Example: Detailed Format

```markdown
## Design Preferences

### Color Palette
- Primary: Indigo (#6366F1)
- Secondary: Pink (#EC4899)
- Background: Off-white (#FAFAFA)
- Text: Charcoal (#1F2937)

### Typography
- Font Family: Outfit (headings), Inter (body)
- Heading Sizes: Large (3rem for h1)
- Body: 1.125rem for comfortable reading

### Spacing & Layout
- Container: Max 1200px
- Padding: Generous (2rem desktop, 1rem mobile)
- Section Gaps: 5rem between sections

### Component Style
- Buttons: Rounded (0.75rem), bold text, shadow on hover
- Cards: White background, subtle border, no shadow
- Inputs: Outlined style, focus ring

### Design Language
- Modern, professional SaaS aesthetic
- Similar to: Linear, Vercel, Stripe
- Avoid: Overly colorful, playful, or informal designs
```

### Example: Reference-Based

```markdown
## Design Inspiration

Use design system similar to:
- https://linear.app (navigation + layout)
- https://vercel.com (typography + spacing)
- https://stripe.com (cards + buttons)

Colors: Blue/purple gradient like Linear
Typography: Clean sans-serif like Vercel
Spacing: Generous like Stripe
```

---

## 4. Organization Defaults

Configure organization-wide defaults in Helm values:

```yaml
# infra/charts/controller/values.yaml
agents:
  blaze:
    defaultTheme:
      name: "5DLabs Corporate"
      
      colors:
        primary: "#0066CC"
        secondary: "#6B7280"
        accent: "#10B981"
      
      typography:
        fontFamily: "Inter, system-ui, sans-serif"
        scale: "1.125rem"  # Slightly larger for better readability
      
      brandGuidelines:
        tone: "Professional, innovative"
        style: "Modern enterprise"
```

---

## How Blaze Resolves Themes

### Resolution Algorithm

```javascript
// scripts/blaze/resolve-theme.js

function resolveTheme(taskDescription, repoPath, orgDefaults) {
  // 1. Start with built-in default
  let theme = loadBuiltInTheme();
  
  // 2. Apply organization defaults (if configured)
  if (orgDefaults) {
    theme = mergeTheme(theme, orgDefaults);
  }
  
  // 3. Apply repository design system (if exists)
  const repoTheme = loadRepoTheme(repoPath);
  if (repoTheme) {
    theme = mergeTheme(theme, repoTheme);
  }
  
  // 4. Apply task-level overrides (highest priority)
  const taskOverrides = parseTaskDesignPreferences(taskDescription);
  if (taskOverrides) {
    theme = mergeTheme(theme, taskOverrides);
  }
  
  return theme;
}

function mergeTheme(base, override) {
  // Deep merge with override taking precedence
  return deepMerge(base, override);
}
```

### v0 Prompt Enhancement

```javascript
// scripts/blaze/v0-generate.js

function enhancePromptWithTheme(taskDescription, theme) {
  return `
Create a modern React component with the following requirements:

${taskDescription}

DESIGN SYSTEM:
Colors:
- Primary: ${theme.colors.primary.light} (light mode), ${theme.colors.primary.dark} (dark mode)
- Background: ${theme.colors.background.light}
- Text: ${theme.colors.foreground.light}

Typography:
- Font Family: ${theme.typography.fontFamily.sans}
- Base Size: ${theme.typography.fontSize.base}
- Headings: ${theme.typography.fontFamily.heading}

Spacing:
- Container Max Width: ${theme.spacing.containerMaxWidth}
- Use Tailwind spacing scale: ${theme.spacing.scale.join(', ')}

Border Radius:
- Default: ${theme.borderRadius.default}
- Buttons: ${theme.borderRadius.default}
- Cards: ${theme.borderRadius.lg}

Shadows:
- Cards: ${theme.shadows.default}
- Hover Effects: ${theme.shadows.lg}

IMPORTANT: Use these exact design tokens. All colors should use Tailwind CSS variables.
Generate a tailwind.config.ts that includes these design tokens.
`;
}
```

---

## shadcn/ui Integration

Blaze automatically configures shadcn/ui to use the resolved theme:

### Generated `components.json`

```json
{
  "$schema": "https://ui.shadcn.com/schema.json",
  "style": "default",
  "rsc": false,
  "tsx": true,
  "tailwind": {
    "config": "tailwind.config.ts",
    "css": "app/globals.css",
    "baseColor": "slate",
    "cssVariables": true
  },
  "aliases": {
    "components": "@/components",
    "utils": "@/lib/utils"
  }
}
```

### Generated `tailwind.config.ts`

```typescript
import type { Config } from "tailwindcss"

const config = {
  darkMode: ["class"],
  content: [
    './pages/**/*.{ts,tsx}',
    './components/**/*.{ts,tsx}',
    './app/**/*.{ts,tsx}',
  ],
  theme: {
    container: {
      center: true,
      padding: "1rem",
      screens: {
        "2xl": "1280px",
      },
    },
    extend: {
      colors: {
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        // ... from resolved theme
      },
      fontFamily: {
        sans: ["Inter", "system-ui", "sans-serif"],
        heading: ["Inter", "system-ui", "sans-serif"],
      },
      borderRadius: {
        lg: "0.75rem",
        md: "0.5rem",
        sm: "0.25rem",
      },
    },
  },
  plugins: [require("tailwindcss-animate")],
} satisfies Config

export default config
```

---

## Common Use Cases

### Use Case 1: New Project (No Theme)

**Scenario**: First Blaze task for a new repo

**Result**:
- Uses Blaze built-in default theme
- Creates `.blaze/design-system.json` with defaults for future consistency
- User can edit this file for subsequent tasks

### Use Case 2: Existing Project (Has Design System)

**Scenario**: Repo already has `.blaze/design-system.json`

**Result**:
- Blaze reads existing design system
- All components match existing style
- Consistent look across all Blaze-generated UIs

### Use Case 3: One-Off Experiment

**Scenario**: User wants to try a different color scheme for this specific task

**Task Description**:
```markdown
## Theme Override
Use purple (#8B5CF6) as primary color for this experiment
Keep everything else from the default design system
```

**Result**:
- Applies purple override for this task only
- Doesn't modify `.blaze/design-system.json`
- Future tasks still use default colors

### Use Case 4: Multiple Brands

**Scenario**: Monorepo with multiple sub-brands

**Structure**:
```
apps/
├── consumer-app/
│   └── .blaze/design-system.json (bright, playful)
├── business-app/
│   └── .blaze/design-system.json (professional, muted)
└── admin-app/
    └── .blaze/design-system.json (functional, dense)
```

**Result**: Blaze reads the appropriate design system based on task location

---

## Theme Management Commands

### Initialize Design System

```bash
# Create default design system for repo
blaze init-theme

# Create from template
blaze init-theme --template saas|ecommerce|dashboard|landing
```

### Validate Theme

```bash
# Check theme syntax and completeness
blaze validate-theme .blaze/design-system.json
```

### Preview Theme

```bash
# Generate theme preview page
blaze preview-theme

# Result: Deploys a preview showing all components with current theme
```

---

## Best Practices

### For Repository Maintainers

1. **Create `.blaze/design-system.json` early** - Establish consistency from the start
2. **Document brand guidelines** - Include "doNot" section for common mistakes
3. **Use semantic color names** - "primary", "accent" not "blue", "red"
4. **Keep it DRY** - Use "extends": "blaze-default" and only override what's different
5. **Version your theme** - Increment version when making breaking changes

### For Task Creators

1. **Reference existing designs** - "Similar to Linear" is clearer than describing from scratch
2. **Be specific about colors** - Provide hex codes, not just "blue"
3. **Consider accessibility** - Request WCAG AA compliant colors
4. **Specify breakpoints** - "Desktop: 1200px max-width" helps Blaze understand intent
5. **Use visual references** - Link to examples or inspiration

### For Organization Admins

1. **Set sensible org defaults** - Cover 80% of use cases
2. **Maintain brand consistency** - Align org defaults with company brand
3. **Document theme usage** - Include examples in internal docs
4. **Review generated themes** - Periodically check Blaze output quality
5. **Collect feedback** - Iterate on defaults based on team input

---

## Migration Guide

### Migrating Existing Project to Blaze Themes

**Step 1**: Extract existing styles
```bash
# Analyze existing Tailwind config
cat tailwind.config.js

# Note: colors, fonts, spacing used
```

**Step 2**: Create design system file
```bash
# Create .blaze directory
mkdir -p .blaze

# Create design system
cat > .blaze/design-system.json <<EOF
{
  "name": "MyApp Design System",
  "colors": {
    "primary": { "light": "#YOUR_COLOR" }
    // ... extracted values
  }
}
EOF
```

**Step 3**: Test with Blaze
```bash
# Create test task
gh issue create --label agent-blaze \
  --title "Test: Button component" \
  --body "Create a simple button using our design system"

# Verify output matches existing styles
```

**Step 4**: Refine and iterate
```bash
# Update design system based on output
# Repeat test until consistent
```

---

## Troubleshooting

### Issue: Colors don't match design system

**Cause**: v0 may interpret colors differently  
**Solution**: Be more explicit in prompt, provide HSL values

### Issue: Font not loading

**Cause**: Font needs to be imported  
**Solution**: Add font import to design system:
```json
{
  "typography": {
    "fontImports": [
      "https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap"
    ]
  }
}
```

### Issue: Inconsistent spacing

**Cause**: Multiple spacing scales in use  
**Solution**: Standardize on Tailwind's default scale or define explicit values

---

## Future Enhancements

### Phase 2
- **Theme Preview Generator** - Automated component gallery
- **Theme Validation** - Accessibility contrast checks
- **Theme Library** - Shareable community themes

### Phase 3
- **Figma Integration** - Import design tokens from Figma
- **Design System Linter** - Automated consistency checks
- **Theme Analytics** - Track which themes perform best

---

**Last Updated**: 2025-10-18  
**Version**: 1.0  
**Maintainer**: CTO @ 5D Labs


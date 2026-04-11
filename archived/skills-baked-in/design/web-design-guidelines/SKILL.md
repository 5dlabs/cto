---
name: web-design-guidelines
description: Web Interface Guidelines for reviewing UI code compliance with modern design principles and accessibility standards.
---

# Web Interface Guidelines

Review files for compliance with Web Interface Guidelines - a comprehensive set of rules for building modern, accessible, and performant web interfaces.

## When to Apply

Use these guidelines when:
- Building new UI components
- Reviewing frontend code
- Ensuring accessibility compliance
- Optimizing user experience
- Establishing design system patterns

## Core Principles

### 1. Semantic HTML

- Use appropriate HTML elements for their intended purpose
- Prefer `<button>` over `<div onClick>`
- Use heading hierarchy (`h1` → `h6`) correctly
- Use `<nav>`, `<main>`, `<aside>`, `<footer>` for page structure
- Use `<ul>`/`<ol>` for lists, not divs with CSS

### 2. Accessibility (a11y)

- All interactive elements must be keyboard accessible
- Provide meaningful `alt` text for images
- Use `aria-label` when visual context is insufficient
- Ensure sufficient color contrast (WCAG AA: 4.5:1 for text)
- Don't rely on color alone to convey information
- Support reduced motion preferences
- Focus states must be visible

### 3. Responsive Design

- Mobile-first approach
- Use relative units (`rem`, `em`, `%`) over fixed (`px`)
- Test at multiple breakpoints
- Touch targets minimum 44x44px
- Content should be readable without horizontal scrolling

### 4. Performance

- Lazy load below-the-fold content
- Optimize images (WebP, proper sizing)
- Minimize layout shifts (CLS)
- Use CSS containment where appropriate
- Prefer CSS animations over JavaScript

### 5. Forms

- Always associate labels with inputs
- Provide clear error messages
- Use appropriate input types (`email`, `tel`, `number`)
- Support autofill with correct `autocomplete` attributes
- Show loading/success/error states

## Output Format

When reviewing code, report findings as:

```
file:line - [severity] rule-name: description
```

Severities:
- `[error]` - Must fix, breaks functionality or accessibility
- `[warning]` - Should fix, impacts UX or performance
- `[info]` - Consider fixing, best practice improvement

## Examples

### Good Button Implementation

```tsx
<button
  type="button"
  onClick={handleClick}
  disabled={isLoading}
  aria-busy={isLoading}
  className="px-4 py-2 rounded-md bg-primary text-primary-foreground hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-2"
>
  {isLoading ? <Spinner aria-hidden /> : null}
  {isLoading ? 'Saving...' : 'Save'}
</button>
```

### Good Form Implementation

```tsx
<form onSubmit={handleSubmit}>
  <div className="space-y-4">
    <div>
      <label htmlFor="email" className="block text-sm font-medium">
        Email
      </label>
      <input
        id="email"
        type="email"
        name="email"
        autoComplete="email"
        required
        aria-describedby={error ? 'email-error' : undefined}
        className="mt-1 block w-full rounded-md border"
      />
      {error && (
        <p id="email-error" role="alert" className="mt-1 text-sm text-destructive">
          {error}
        </p>
      )}
    </div>
    <button type="submit">Submit</button>
  </div>
</form>
```

### Good Image Implementation

```tsx
<Image
  src="/hero.webp"
  alt="Dashboard showing analytics overview with charts and metrics"
  width={1200}
  height={600}
  priority // Above the fold
  className="rounded-lg"
/>
```

## Checklist

Before shipping UI code, verify:

- [ ] All interactive elements are keyboard accessible
- [ ] Forms have proper labels and error handling
- [ ] Images have meaningful alt text
- [ ] Color contrast meets WCAG AA
- [ ] Component works at mobile breakpoints
- [ ] Loading and error states are handled
- [ ] Focus states are visible
- [ ] No accessibility violations in DevTools audit

## Attribution

Based on [vercel-labs/agent-skills](https://github.com/vercel-labs/agent-skills) web-design-guidelines - 16K+ installs.

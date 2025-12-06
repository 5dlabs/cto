# Task 32: Implement dark/light theme toggle with persistence

## Role

You are a Senior Frontend Engineer with expertise in React, TypeScript, and modern UI/UX implementing Task 32.

## Goal

Add theme switching functionality with system preference detection and localStorage persistence across the React dashboard.

## Requirements

1. Create src/contexts/ThemeContext.tsx:
   - ThemeProvider with state: 'light' | 'dark' | 'system'
   - Detect system preference with window.matchMedia('(prefers-color-scheme: dark)')
   - Listen for system theme changes
   - Persist preference in localStorage
2. Create src/components/ThemeToggle.tsx:
   - Button with sun/moon/auto icons
   - Cycles through light -> dark -> system
3. Update tailwind.config.js:
   - Enable dark mode: 'class'
4. Add dark: variants to all components:
   - Dark background colors
   - Dark text colors
   - Dark borders and shadows
5. Apply theme class to document.documentElement:
   - <html className={theme === 'dark' ? 'dark' : ''}>
6. Create CSS variables for theme colors in src/index.css:
   - --color-background, --color-text, etc.
7. Add ThemeToggle to app header

## Acceptance Criteria

Unit tests for ThemeContext. Test system preference detection. Verify localStorage persistence across sessions. Test theme toggle cycles correctly. Manual testing: verify all components readable in both themes. Test automatic theme switch when system preference changes. Verify no flash of unstyled content on load.

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-32): Implement dark/light theme toggle with persistence`

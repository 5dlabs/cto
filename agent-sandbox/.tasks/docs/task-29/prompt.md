# Task 29: Implement dark/light theme toggle with persistence

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 29.

## Goal

Add theme switching capability with localStorage persistence and system preference detection

## Requirements

1. Create src/contexts/ThemeContext.tsx:
   - type Theme = 'light' | 'dark' | 'system'
   - useTheme hook: provides theme state and toggle function
   - Detect system preference with window.matchMedia('(prefers-color-scheme: dark)')
2. Store preference in localStorage: theme key
3. Apply theme via Tailwind dark mode class on <html> element
4. Create src/components/ThemeToggle.tsx: button with sun/moon icons
5. Update tailwind.config.js: darkMode: 'class'
6. Define color palette in tailwind.config.js:
   - Light: bg-white, text-gray-900
   - Dark: bg-gray-900, text-gray-100
7. Update all components to use theme-aware Tailwind classes
8. Add theme toggle to navigation bar

## Acceptance Criteria

Test theme persistence across page reloads, verify system preference detection, test all components in both themes, verify localStorage updates

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-29): Implement dark/light theme toggle with persistence`

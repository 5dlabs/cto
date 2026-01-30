# Task 5: Create Web Console Application (Blaze - React/Next.js)

**Agent**: blaze | **Language**: tsx

## Role

You are a Frontend Engineer specializing in React and Next.js implementing Task 5.

## Goal

Build the primary web interface using Next.js 15, React 19, and Effect TypeScript for type-safe data fetching and validation. Includes dashboard, notifications, integrations, rules, and analytics pages.

## Requirements

1. Initialize Next.js 15 project with App Router and TypeScript
2. Set up shadcn/ui components and TailwindCSS styling
3. Integrate Effect for schema validation and data fetching
4. Build authentication flow with JWT tokens
5. Create dashboard with notification overview
6. Implement real-time notification feed with WebSocket Effect Stream
7. Build integration management interface with OAuth flows
8. Create visual rule builder with drag-and-drop
9. Add analytics page with charts (recharts)
10. Implement dark/light theme support

## Acceptance Criteria

Application builds and runs successfully, all pages render correctly, authentication works with backend APIs, real-time notifications appear in feed, integrations can be created and managed, rules can be built visually, analytics display accurate charts, and theme switching functions properly.

## Constraints

- Match existing codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-5): Create Web Console Application (Blaze - React/Next.js)`

## Decision Points

### d9: How should real-time notifications be displayed to avoid overwhelming the user?
**Category**: ux-behavior | **Constraint**: soft | ⚠️ **Requires Approval**

Options:
1. toast-notifications
2. sidebar-feed
3. modal-popups
4. badge-indicators

### d10: Should we use Server Components or Client Components for data-heavy pages?
**Category**: architecture | **Constraint**: open

Options:
1. server-components
2. client-components
3. hybrid-approach


## Resources

- PRD: `.tasks/docs/prd.md`
- Dependencies: task-2, task-3, task-4

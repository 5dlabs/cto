# Task 5: Web Console with Effect Integration (Blaze - React/Next.js)

**Agent**: blaze | **Language**: tsx

## Role

You are a Frontend Engineer specializing in React and Next.js implementing Task 5.

## Goal

Build the web-based dashboard and management interface using Next.js 15 with Effect TypeScript for type-safe data fetching and form validation.

## Requirements

Create Next.js app with pages for dashboard, notifications, integrations, rules, settings, and analytics. Implement Effect Schema for validation, TanStack Query with Effect for data fetching, WebSocket integration with Effect Stream, and shadcn/ui components with TailwindCSS styling.

## Acceptance Criteria

All pages load correctly, WebSocket receives real-time updates, forms validate with Effect Schema, API calls succeed with proper error handling, responsive design works on mobile, and dark/light theme switching functions

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-5): Web Console with Effect Integration (Blaze - React/Next.js)`

## Decision Points

### d9: Real-time notification feed update strategy
**Category**: ux-behavior | **Constraint**: soft

Options:
1. append new notifications to top
2. show notification count with manual refresh
3. auto-scroll to new notifications

### d10: Notification history pagination approach
**Category**: performance | **Constraint**: open

Options:
1. cursor-based pagination
2. offset-based pagination
3. infinite scroll with virtual scrolling


## Resources

- PRD: `.tasks/docs/prd.md`
- Dependencies: task-2, task-3, task-4

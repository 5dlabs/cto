# Implementation Prompt for Task 5

## Context
You are implementing "Web Console with Effect Integration (Blaze - React/Next.js)" for the AlertHub notification platform.

## PRD Reference
See `../../prd.md` for full requirements.

## Task Requirements
Build the web-based dashboard and management interface using Next.js 15 with Effect TypeScript for type-safe data fetching and form validation.

## Implementation Details
Create Next.js app with pages for dashboard, notifications, integrations, rules, settings, and analytics. Implement Effect Schema for validation, TanStack Query with Effect for data fetching, WebSocket integration with Effect Stream, and shadcn/ui components with TailwindCSS styling.

## Dependencies
This task depends on: task-2, task-3, task-4. Ensure those are complete before starting.

## Testing Requirements
All pages load correctly, WebSocket receives real-time updates, forms validate with Effect Schema, API calls succeed with proper error handling, responsive design works on mobile, and dark/light theme switching functions

## Decision Points to Address

The following decisions need to be made during implementation:

### d9: Real-time notification feed update strategy
**Category**: ux-behavior | **Constraint**: soft

Options:
1. append new notifications to top
2. show notification count with manual refresh
3. auto-scroll to new notifications

Document your choice and rationale in the implementation.

### d10: Notification history pagination approach
**Category**: performance | **Constraint**: open

Options:
1. cursor-based pagination
2. offset-based pagination
3. infinite scroll with virtual scrolling

Document your choice and rationale in the implementation.


## Deliverables
1. Source code implementing the requirements
2. Unit tests with >80% coverage
3. Integration tests for external interfaces
4. Documentation updates as needed
5. Decision point resolutions documented

## Notes
- Follow project coding standards
- Use Effect TypeScript patterns where applicable
- Ensure proper error handling and logging

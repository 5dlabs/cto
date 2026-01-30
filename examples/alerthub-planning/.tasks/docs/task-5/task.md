# Task 5: Web Console with Effect Integration (Blaze - React/Next.js)

## Status
pending

## Priority
high

## Dependencies
task-2, task-3, task-4

## Description
Build the web-based dashboard and management interface using Next.js 15 with Effect TypeScript for type-safe data fetching and form validation.

## Details
Create Next.js app with pages for dashboard, notifications, integrations, rules, settings, and analytics. Implement Effect Schema for validation, TanStack Query with Effect for data fetching, WebSocket integration with Effect Stream, and shadcn/ui components with TailwindCSS styling.

## Test Strategy
All pages load correctly, WebSocket receives real-time updates, forms validate with Effect Schema, API calls succeed with proper error handling, responsive design works on mobile, and dark/light theme switching functions

## Decision Points

### d9: Real-time notification feed update strategy
- **Category**: ux-behavior
- **Constraint**: soft
- **Requires Approval**: No
- **Options**:
  - append new notifications to top
  - show notification count with manual refresh
  - auto-scroll to new notifications

### d10: Notification history pagination approach
- **Category**: performance
- **Constraint**: open
- **Requires Approval**: No
- **Options**:
  - cursor-based pagination
  - offset-based pagination
  - infinite scroll with virtual scrolling


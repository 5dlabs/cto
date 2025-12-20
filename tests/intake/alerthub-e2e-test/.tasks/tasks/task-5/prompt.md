# Task 5: Web Console (Next.js + Effect)

## Agent: Blaze
## Priority: High
## Stack: Next.js 15 App Router, React 19, shadcn/ui, TailwindCSS, Effect

## Objective
Build the primary configuration interface for AlertHub.

## Pages
- `/` - Dashboard with notification overview
- `/notifications` - Notification history with filters
- `/integrations` - Manage channel integrations
- `/rules` - Configure notification rules
- `/settings` - Tenant and user settings
- `/analytics` - Delivery metrics and charts

## Core Features
- Dark/light theme support
- Real-time notification feed (WebSocket with Effect Stream)
- Drag-and-drop rule builder
- Integration wizard with OAuth flows
- Responsive design (mobile-friendly)

## Acceptance Criteria
- [ ] All pages render correctly
- [ ] Dark/light theme toggle works
- [ ] Real-time updates work via WebSocket
- [ ] Tests pass with `pnpm test`
- [ ] ESLint passes


# Task 27: Build React dashboard with Kanban board and drag-and-drop

## Role

You are a Senior Frontend Engineer with expertise in React, TypeScript, and modern UI/UX implementing Task 27.

## Goal

Create React 18 TypeScript frontend with Tailwind CSS featuring a drag-and-drop Kanban board for task management

## Requirements

1. Initialize React app: npx create-react-app frontend --template typescript
2. Install dependencies: npm install @dnd-kit/core @dnd-kit/sortable tailwindcss axios react-router-dom zustand
3. Setup Tailwind: npx tailwindcss init, configure tailwind.config.js
4. Create components:
   - src/components/KanbanBoard.tsx: use @dnd-kit for drag-and-drop
   - src/components/TaskCard.tsx: display task with status badge
   - src/components/TaskModal.tsx: create/edit task form
5. Create stores with Zustand:
   - src/stores/taskStore.ts: manage tasks state, WebSocket integration
   - src/stores/authStore.ts: JWT token management
6. Implement WebSocket connection in taskStore:
   - Connect on mount, reconnect on disconnect
   - Update tasks state on WebSocket messages
7. API client in src/api/client.ts: axios instance with JWT interceptor
8. Implement drag-and-drop: on drop, PATCH /api/tasks/:id with new status
9. Add optimistic updates: update UI immediately, rollback on API error

## Acceptance Criteria

Component tests with React Testing Library, E2E tests with Playwright for drag-and-drop, verify WebSocket reconnection, test optimistic updates rollback

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-27): Build React dashboard with Kanban board and drag-and-drop`

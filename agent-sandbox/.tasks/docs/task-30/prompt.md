# Task 30: Build React dashboard with Kanban board and drag-and-drop

## Role

You are a Senior Frontend Engineer with expertise in React, TypeScript, and modern UI/UX implementing Task 30.

## Goal

Create React 18 + TypeScript frontend with Tailwind CSS featuring Kanban board with drag-and-drop task management.

## Requirements

1. Initialize React app in frontend/:
   ```bash
   npx create-react-app frontend --template typescript
   cd frontend && npm install -D tailwindcss postcss autoprefixer
   npm install @dnd-kit/core @dnd-kit/sortable axios react-router-dom
   ```
2. Configure Tailwind in tailwind.config.js with dark mode support
3. Create components:
   - src/components/KanbanBoard.tsx (columns: Todo, In Progress, Done)
   - src/components/TaskCard.tsx (draggable task)
   - src/components/TaskModal.tsx (create/edit task)
   - src/components/TeamHeader.tsx (team info, member count)
4. Implement drag-and-drop with @dnd-kit:
   - DndContext wraps board
   - Droppable columns
   - Draggable task cards
   - onDragEnd updates task status via API
5. Setup WebSocket connection:
   - Connect on board mount
   - Listen for TaskEvent, update local state
   - Reconnect on disconnect
6. Create API client in src/api/client.ts:
   - Axios instance with JWT interceptor
   - Methods for all task/team endpoints
7. Add routing: /teams/:id/board
8. Implement optimistic updates for drag-and-drop

## Acceptance Criteria

Unit tests for components with React Testing Library. Integration tests: render board, verify tasks display. Test drag-and-drop updates task status. Verify WebSocket updates reflected in UI. Test create/edit task modal. Manual testing for responsive design and accessibility.

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-30): Build React dashboard with Kanban board and drag-and-drop`

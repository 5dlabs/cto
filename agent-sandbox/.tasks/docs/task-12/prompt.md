# Task 12: Build React dashboard with Kanban board and drag-and-drop

## Role

You are a Senior Frontend Engineer with expertise in React, TypeScript, and modern UI/UX implementing Task 12.

## Goal

Create React frontend with TypeScript, Tailwind CSS, Kanban board view using react-beautiful-dnd, and dark/light theme support

## Requirements

1. Initialize React app: `npx create-react-app frontend --template typescript`
2. Install dependencies:
   - npm install tailwindcss @headlessui/react @heroicons/react
   - npm install react-beautiful-dnd @types/react-beautiful-dnd
   - npm install axios react-router-dom zustand
3. Configure Tailwind with dark mode: class strategy in tailwind.config.js
4. Create components/:
   - KanbanBoard.tsx: 3 columns (To Do, In Progress, Done)
   - TaskCard.tsx: draggable card with title, assignee, due date
   - DragDropContext with onDragEnd handler
5. Implement state management with Zustand:
   - Store: tasks, teams, currentUser, theme
   - Actions: fetchTasks, updateTaskStatus, toggleTheme
6. API client in services/api.ts with axios interceptors for JWT
7. On drag end, call PATCH /api/tasks/:id with new status

## Acceptance Criteria

Component tests with React Testing Library: render KanbanBoard, verify columns. Test drag-and-drop updates task status. E2E test with Playwright: login, drag task, verify API called. Test theme toggle persists in localStorage

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-12): Build React dashboard with Kanban board and drag-and-drop`

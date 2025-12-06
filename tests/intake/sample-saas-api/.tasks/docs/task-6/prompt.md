# Task 6: Create React dashboard with Kanban board and deployment configuration

## Role

You are a Senior DevOps Engineer with expertise in Kubernetes, GitOps, and CI/CD implementing Task 6.

## Goal

Build responsive React dashboard with drag-and-drop Kanban board, team management UI, and complete Kubernetes deployment setup with observability

## Requirements

1. Setup React 18 + TypeScript + Tailwind CSS project structure
2. Implement Kanban board with react-beautiful-dnd for drag-and-drop
3. Build team activity feed component with real-time WebSocket integration
4. Create member management UI with role assignment
5. Add dark/light theme support using Tailwind CSS
6. Setup Kubernetes manifests with HPA for auto-scaling
7. Configure Prometheus metrics endpoint (/metrics) with custom business metrics
8. Ensure mobile responsiveness for all components

```typescript
// Kanban board component
interface Task {
  id: string;
  title: string;
  status: 'todo' | 'in_progress' | 'done';
  assignee?: User;
  due_date?: string;
}

const KanbanBoard: React.FC = () => {
  const [tasks, setTasks] = useState<Task[]>([]);
  
  const onDragEnd = (result: DropResult) => {
    // Update task status via API
  };
};

// Kubernetes deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: teamsync-api
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: api
        image: teamsync:latest
        resources:
          requests: {cpu: 100m, memory: 128Mi}
          limits: {cpu: 500m, memory: 512Mi}
```

## Acceptance Criteria

React component unit tests with Jest/RTL, drag-and-drop functionality tests, WebSocket integration tests, mobile responsiveness validation, Kubernetes deployment verification, Prometheus metrics endpoint testing

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-6): Create React dashboard with Kanban board and deployment configuration`

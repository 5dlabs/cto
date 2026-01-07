# Task 5: Implement Web Console Frontend (Blaze - React/Next.js + Effect)

**Agent**: blaze | **Language**: tsx

## Role

You are a Senior Frontend Engineer with expertise in React, TypeScript, and modern UI/UX implementing Task 5.

## Goal

Build the primary web interface for AlertHub with Effect for type-safe data fetching, validation, and error handling

## Requirements

1. Initialize Next.js 15 project with App Router:
   npx create-next-app@latest web-console --typescript --tailwind --app
   cd web-console
   bun add effect @effect/schema @tanstack/react-query react-hook-form @hookform/resolvers recharts anime.js
   bunx shadcn@latest init
   bunx shadcn@latest add button card form input select table toast

2. Setup Effect Schemas for API responses:
   const NotificationSchema = Schema.Struct({ id: Schema.String, status: Schema.Literal("pending", "processing", "delivered", "failed"), channel: Schema.Literal("slack", "discord", "email", "push", "webhook"), priority: Schema.Literal("critical", "high", "normal", "low"), payload: Schema.Struct({ title: Schema.String, body: Schema.String }), createdAt: Schema.Date })
   const IntegrationSchema = Schema.Struct({ id: Schema.String, name: Schema.String, channel: Schema.Literal(...), enabled: Schema.Boolean })
   const RuleSchema = Schema.Struct({ id: Schema.String, name: Schema.String, conditions: Schema.Array(...), actions: Schema.Array(...), enabled: Schema.Boolean })

3. Implement Effect-powered API client:
   const fetchNotifications = Effect.tryPromise({ try: () => fetch("/api/notifications").then(r => r.json()), catch: () => new ApiError({ message: "Failed to fetch" }) }).pipe(Effect.flatMap(Schema.decodeUnknown(Schema.Array(NotificationSchema))))
   const createIntegration = (data: CreateIntegration) => Effect.tryPromise({ try: () => fetch("/api/integrations", { method: "POST", body: JSON.stringify(data) }).then(r => r.json()), catch: () => new ApiError({ message: "Failed to create" }) }).pipe(Effect.flatMap(Schema.decodeUnknown(IntegrationSchema)))

4. Create TanStack Query hooks with Effect:
   function useNotifications() { return useQuery({ queryKey: ["notifications"], queryFn: () => Effect.runPromise(fetchNotifications) }) }
   function useCreateIntegration() { return useMutation({ mutationFn: (data) => Effect.runPromise(createIntegration(data)) }) }

5. Build page components:
   app/page.tsx - Dashboard with notification stats, recent activity, quick actions
   app/notifications/page.tsx - Notification history with filters (status, channel, date range), pagination, search
   app/integrations/page.tsx - Integration cards with status, actions (test, edit, delete), add new button
   app/rules/page.tsx - Rule list with drag-and-drop priority ordering, create/edit modal
   app/settings/page.tsx - Tenant settings, user preferences, API keys
   app/analytics/page.tsx - Charts for delivery metrics (success rate, channel breakdown, time series)

6. Implement NotificationFeed component with WebSocket:
   const notificationStream = Stream.async<Notification, WebSocketError>((emit) => {
     const ws = new WebSocket("/api/v1/ws")
     ws.onmessage = (event) => emit.single(Schema.decodeUnknownSync(NotificationSchema)(JSON.parse(event.data)))
     ws.onerror = () => emit.fail(new WebSocketError({ message: "Connection failed" }))
     return Effect.sync(() => ws.close())
   })
   Use Effect.Stream.runForEach to update UI state

7. Implement form validation with Effect Schema:
   const CreateIntegrationSchema = Schema.Struct({ name: Schema.String.pipe(Schema.minLength(1)), channel: Schema.Literal(...), webhookUrl: Schema.optional(Schema.String.pipe(Schema.pattern(/^https?:\/\//))), enabled: Schema.Boolean })
   const form = useForm({ resolver: effectResolver(CreateIntegrationSchema) })

8. Build RuleBuilder component:
   - Drag-and-drop interface for conditions (field, operator, value)
   - Visual action configuration (route to channel, set priority, etc.)
   - Preview rule evaluation with sample data
   - Save/update with Effect error handling

9. Implement AnalyticsChart component:
   - Use recharts for line/bar/pie charts
   - Fetch data with Effect and TanStack Query
   - Transform data with Effect.map
   - Add date range selector, refresh button

10. Add dark/light theme:
    - Use next-themes for theme switching
    - Configure TailwindCSS dark mode
    - Add theme toggle in header

11. Implement toast notifications:
    - Use shadcn toast component
    - Map Effect errors to user-friendly messages
    - Show success toasts for actions

12. Create Dockerfile:
   FROM oven/bun:1.1 AS builder
   WORKDIR /app
   COPY package.json bun.lockb ./
   RUN bun install --frozen-lockfile
   COPY . .
   RUN bun run build
   FROM oven/bun:1.1
   WORKDIR /app
   COPY --from=builder /app/.next ./.next
   COPY --from=builder /app/public ./public
   COPY --from=builder /app/package.json ./
   CMD ["bun", "run", "start"]

## Acceptance Criteria

1. Unit tests for Effect schemas and API client functions
2. Component tests with React Testing Library
3. Test form validation with invalid inputs
4. Test WebSocket connection and real-time updates
5. Test TanStack Query cache behavior
6. Visual regression tests with Playwright
7. Accessibility tests (WCAG AA compliance)
8. Test dark/light theme switching
9. Test responsive design on mobile/tablet
10. E2E tests for critical user flows (create integration, configure rule)

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-5): Implement Web Console Frontend (Blaze - React/Next.js + Effect)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 2, 3, 4

# Subtask 5.2: Implement Auth Integration

## Parent Task
Task 5

## Agent
auth-implementer

## Parallelizable
Yes

## Description
Integrate authentication flow with auth service.

## Details
- Create auth context provider
- Implement login/register forms
- Handle JWT token storage
- Create protected route wrapper
- Implement session refresh

## Deliverables
- `src/context/auth-context.tsx` - Auth state
- `src/components/auth/` - Auth components
- `src/hooks/use-auth.ts` - Auth hook
- `src/middleware.ts` - Route protection

## Acceptance Criteria
- [ ] Login flow works
- [ ] Protected routes redirect
- [ ] Token refreshes automatically

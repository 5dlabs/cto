# Subtask 2.5: Implement Admin User Management

## Parent Task
Task 2

## Agent
admin-implementer

## Parallelizable
Yes

## Description
Build admin API endpoints for user management.

## Details
- List users with pagination and filtering
- Update user roles and permissions
- Disable/enable user accounts
- View user activity logs
- Export user data for GDPR

## Deliverables
- `admin_handler.go` - Admin endpoints
- `admin_middleware.go` - Role-based access control
- `user_exporter.go` - GDPR data export

## Acceptance Criteria
- [ ] Admin can list all users
- [ ] Role changes take effect immediately
- [ ] Disabled users cannot authenticate
- [ ] GDPR export includes all user data

## Testing Strategy
- Test RBAC permissions
- Verify pagination works
- Test GDPR export format

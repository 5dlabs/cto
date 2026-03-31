Implement subtask 3014: Seed database with sample RMS data

## Objective
Create a database seeding script or Go command that populates the RMS database with sample opportunities, projects, inventory items, crew members, bookings, and deliveries for development and testing.

## Steps
1. Create `/cmd/seed/main.go` or a `make seed` target.
2. Seed data:
   - 5 opportunities in various statuses (2 LEAD, 1 QUOTED, 1 WON, 1 LOST)
   - 3 projects linked to WON opportunities (1 ACTIVE, 1 PENDING, 1 COMPLETED)
   - 20 inventory items across categories (generators, lighting, tents, tables, chairs) with barcodes
   - 5 bookings linking inventory to projects with non-overlapping date ranges
   - 8 crew members with different roles
   - 4 crew assignments to active projects
   - 4 deliveries in various statuses
3. Make seeding idempotent (use UPSERT or check-before-insert).
4. Add a Makefile target: `make seed`.

## Validation
Running `make seed` twice produces no errors and no duplicate records. After seeding, querying each table returns the expected number of records. All foreign key relationships are valid.
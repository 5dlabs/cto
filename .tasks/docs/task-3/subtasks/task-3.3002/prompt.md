Implement subtask 3002: Write database migrations for RMS schema

## Objective
Create SQL migration files for all RMS domain tables: opportunities, projects, inventory items, crew members, crew assignments, deliveries, and supporting lookup/junction tables.

## Steps
1. Use golang-migrate or goose for migration tooling. 2. Create migration for opportunities table: id (UUID), customer_id, title, description, status (enum: draft, quoted, accepted, rejected, converted), quoted_amount, currency, valid_until, created_at, updated_at. 3. Create migration for projects table: id, opportunity_id (FK), name, status (enum: planning, active, on_hold, completed, cancelled), start_date, end_date, budget, created_at, updated_at. 4. Create migration for inventory_items: id, barcode, name, description, category, status (enum: available, rented, maintenance, retired), daily_rate, location, created_at, updated_at. 5. Create migration for project_inventory (junction): project_id, inventory_item_id, quantity, checkout_date, return_date. 6. Create migration for crew_members: id, name, email, phone, role, hourly_rate, availability_status, google_calendar_id, created_at. 7. Create migration for crew_assignments: id, project_id, crew_member_id, start_date, end_date, role_on_project. 8. Create migration for deliveries: id, project_id, status (enum: pending, in_transit, delivered, returned), pickup_address, delivery_address, scheduled_date, actual_date, driver_notes, created_at, updated_at. 9. Add appropriate indexes on foreign keys and frequently queried columns (status, barcode).

## Validation
Migrations run successfully against a clean PostgreSQL database; `migrate up` and `migrate down` are idempotent; all tables, columns, indexes, and constraints exist as specified when inspected via psql.
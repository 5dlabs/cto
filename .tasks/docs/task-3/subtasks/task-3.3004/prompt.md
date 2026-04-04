Implement subtask 3004: Database migrations for all RMS schema tables

## Objective
Create golang-migrate migration files for all 9 tables in the rms schema with org_id column, indexes, and foreign key constraints.

## Steps
1. Add `golang-migrate` and `pgx/v5` dependencies to go.mod.
2. Create migration `000001_create_rms_schema.up.sql`: `CREATE SCHEMA IF NOT EXISTS rms;`
3. Create migration `000002_create_opportunities.up.sql`: table `rms.opportunities` with columns: id UUID PK DEFAULT gen_random_uuid(), org_id UUID NOT NULL, customer_id UUID NOT NULL, title TEXT NOT NULL, description TEXT, event_date_start TIMESTAMPTZ, event_date_end TIMESTAMPTZ, status TEXT NOT NULL DEFAULT 'PENDING' CHECK (status IN ('PENDING','QUALIFIED','APPROVED','CONVERTED')), lead_score TEXT, created_at TIMESTAMPTZ DEFAULT now(), updated_at TIMESTAMPTZ DEFAULT now(). Index on (org_id), (org_id, status), (org_id, customer_id).
4. Create migration `000003_create_projects.up.sql`: table `rms.projects` with: id UUID PK, org_id UUID NOT NULL, opportunity_id UUID REFERENCES rms.opportunities(id), customer_id UUID NOT NULL, title TEXT, status TEXT, checkout_date TIMESTAMPTZ, checkin_date TIMESTAMPTZ, created_at, updated_at. Index on (org_id), (opportunity_id).
5. Create migration `000004_create_project_line_items.up.sql`: table `rms.project_line_items` with: id UUID PK, org_id UUID NOT NULL, project_id UUID REFERENCES rms.projects(id), inventory_item_id UUID, quantity INT, unit_price NUMERIC(10,2), created_at.
6. Create migration `000005_create_inventory_items.up.sql`: table `rms.inventory_items` with: id UUID PK, org_id UUID NOT NULL, name TEXT, barcode TEXT UNIQUE, category TEXT, current_location TEXT, status TEXT DEFAULT 'AVAILABLE', quantity_total INT, quantity_available INT, created_at, updated_at. Index on (org_id), (barcode), (org_id, status).
7. Create migration `000006_create_inventory_transactions.up.sql`: table `rms.inventory_transactions` with: id UUID PK, org_id UUID NOT NULL, item_id UUID REFERENCES rms.inventory_items(id), project_id UUID REFERENCES rms.projects(id), type TEXT, quantity INT, timestamp TIMESTAMPTZ DEFAULT now(). Index on (item_id, timestamp), (project_id).
8. Create migration `000007_create_crew_members.up.sql`: table `rms.crew_members` with: id UUID PK, org_id UUID NOT NULL, name TEXT, email TEXT, role TEXT, skills TEXT[]. Index on (org_id).
9. Create migration `000008_create_crew_assignments.up.sql`: table `rms.crew_assignments` with: id UUID PK, org_id UUID NOT NULL, crew_member_id UUID REFERENCES rms.crew_members(id), project_id UUID REFERENCES rms.projects(id), date_start TIMESTAMPTZ, date_end TIMESTAMPTZ, role TEXT. Index on (crew_member_id, date_start, date_end), (project_id).
10. Create migration `000009_create_deliveries.up.sql`: table `rms.deliveries` with: id UUID PK, org_id UUID NOT NULL, project_id UUID REFERENCES rms.projects(id), type TEXT, address JSONB, scheduled_at TIMESTAMPTZ, status TEXT DEFAULT 'SCHEDULED', created_at, updated_at.
11. Create migration `000010_create_delivery_routes.up.sql`: table `rms.delivery_routes` with: id UUID PK, org_id UUID NOT NULL, delivery_ids UUID[], optimized_order INT[], estimated_duration_minutes INT, created_at.
12. Create corresponding `.down.sql` files for all migrations.
13. Write a `db/db.go` package that initializes pgx pool from `DATABASE_URL` env var (PgBouncer from sigma1-infra-endpoints) and runs migrations on startup.

## Validation
Use testcontainers-go to spin up a PostgreSQL container. Run all migrations up, verify all 9 tables exist in rms schema with correct columns via `information_schema.columns` queries. Run all migrations down, verify schema is clean. Run migrations up again to verify idempotency.
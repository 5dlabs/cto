<identity>
You are grizz working on subtask 3003 of task 3.
</identity>

<context>
<scope>
Create numbered SQL migration files in services/rms/migrations/ targeting the rms schema. Cover all seven tables: opportunities, opportunity_line_items, projects, inventory_transactions, crew_members, crew_assignments, deliveries.
</scope>
</context>

<implementation_plan>
Each migration file uses `SET search_path=rms;` at the top. Migration 001: CREATE SCHEMA IF NOT EXISTS rms. Migration 002: opportunities table with all columns, constraints, and CHECK constraints for status and lead_score enums. Migration 003: opportunity_line_items with FK to opportunities. Migration 004: projects with FK to opportunities. Migration 005: inventory_transactions with CHECK constraint on type. Migration 006: crew_members. Migration 007: crew_assignments with FKs to projects and crew_members. Migration 008: deliveries with FK to projects and JSONB route_data column. Add indexes on foreign keys and frequently filtered columns (status, customer_id, inventory_id, project_id). Use golang-migrate UP/DOWN pairs for all migrations.
</implementation_plan>

<validation>
Run `migrate -path migrations -database $DATABASE_URL up` against a fresh rms schema; all 8 migrations apply without error. Run `migrate down` to 0; all tables are dropped cleanly. `psql -c '\dt rms.*'` lists all 7 tables after UP.
</validation>
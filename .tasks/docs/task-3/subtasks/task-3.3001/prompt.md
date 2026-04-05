Implement subtask 3001: Initialize Go project with gRPC, grpc-gateway, and database migrations

## Objective
Set up the Go 1.22+ module structure with gRPC server, grpc-gateway reverse proxy, database connection pooling via POSTGRES_URL and REDIS_URL from ConfigMap, and initial database schema migrations for all RMS domain tables (opportunities, projects, inventory_items, inventory_transactions, crew_members, crew_assignments, deliveries).

## Steps
1. Initialize Go module with `go mod init` for the RMS service.
2. Add dependencies: google.golang.org/grpc, grpc-ecosystem/grpc-gateway/v2, jackc/pgx/v5 for PostgreSQL, redis/go-redis/v9.
3. Create `cmd/server/main.go` with dual listener: gRPC on one port, HTTP/grpc-gateway on another.
4. Load POSTGRES_URL and REDIS_URL from environment (populated via `envFrom` referencing the infra ConfigMap).
5. Set up connection pool for PostgreSQL using pgxpool.
6. Create `migrations/` directory with SQL migration files for all domain tables: opportunities (id, customer_id, title, status, total_amount, created_at, updated_at), projects (id, opportunity_id, start_date, end_date, status, calendar_event_id), inventory_items (id, name, barcode, category, status, location), inventory_transactions (id, item_id, project_id, type [check_out/check_in], timestamp, crew_member_id), crew_members (id, name, role, availability_status), crew_assignments (id, crew_member_id, project_id, start_date, end_date, calendar_event_id), deliveries (id, project_id, type [delivery/pickup], scheduled_at, status, address, notes).
7. Use golang-migrate or similar for migration runner integrated into startup.
8. Create a basic `internal/config/config.go` for centralized configuration loading.

## Validation
Server starts successfully and binds to both gRPC and HTTP ports. Database migrations run without errors. PostgreSQL and Redis connections are established. `go build ./...` succeeds with no errors.
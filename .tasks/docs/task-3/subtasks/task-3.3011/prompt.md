Implement subtask 3011: Implement inter-service auth, GDPR endpoint, and gRPC server bootstrap

## Objective
Build the API key authentication interceptor, GDPR customer deletion endpoint, and the main gRPC + grpc-gateway server wiring.

## Steps
1. Create `internal/auth/apikey.go`:
   - gRPC unary and stream interceptors that extract `Authorization: Bearer <key>` from metadata.
   - Validate key against `sigma1-service-api-keys` secret loaded from env var `SERVICE_API_KEYS` (comma-separated valid keys).
   - Return `codes.Unauthenticated` for missing/invalid keys.
   - Exempt health check endpoints from auth.
2. Create `internal/gdpr/handler.go`:
   - Implement GDPR deletion: given customer_id, delete across all tables in a single transaction.
   - Order: crew_assignments (by project's customer_id) → inventory_transactions (by project) → project_line_items → deliveries → delivery_routes → projects → opportunities.
   - Return structured confirmation: `{deleted: {opportunities: N, projects: N, crew_assignments: N, ...}}`.
   - Register as REST endpoint `DELETE /api/v1/gdpr/customer/{id}` via grpc-gateway or custom HTTP handler on the gateway mux.
3. Create `cmd/server/main.go`:
   - Initialize pgx pool from `DATABASE_URL` env.
   - Run migrations.
   - Create all repo instances.
   - Create all service instances, inject repos and CalendarSyncer.
   - Start gRPC server on port 50051 with auth interceptor.
   - Start grpc-gateway HTTP server on port 8081, registering all services.
   - Mount GDPR handler, health endpoints, and metrics endpoint on the HTTP mux.
   - Graceful shutdown on SIGTERM.

## Validation
1) Auth interceptor test: send gRPC request without auth header → verify Unauthenticated error. Send with invalid key → same. Send with valid key → verify request passes through. 2) GDPR integration test: create opportunity, convert to project, assign crew, create delivery. Call GDPR delete for customer_id. Verify all records deleted and confirmation counts are correct. 3) Server bootstrap test: start server, verify both ports accept connections, verify gRPC reflection lists all 5 services.
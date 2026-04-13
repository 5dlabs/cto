<identity>
You are grizz working on subtask 3004 of task 3.
</identity>

<context>
<scope>
Implement all five OpportunityService methods (CreateOpportunity, GetOpportunity, UpdateOpportunity, ListOpportunities, ScoreLead stub) as gRPC handler structs backed by pgx queries. Wire into grpc-gateway mux on :8080 and gRPC server on :9090.
</scope>
</context>

<implementation_plan>
Create internal/opportunity/handler.go implementing the generated OpportunityServiceServer interface. Use pgx/v5 pool for all DB access. CreateOpportunity: INSERT with uuid.New(), status=pending, return created record. GetOpportunity: SELECT by id, return NotFound if missing. UpdateOpportunity: UPDATE with optimistic updated_at check. ListOpportunities: SELECT with optional status filter via query param. ScoreLead: placeholder returning YELLOW until subtask 3007 is wired in. In cmd/server/main.go: register gRPC server on :9090 with reflection enabled (grpc/reflection package). Register grpc-gateway mux on :8080 with ServeMux backed by grpc.Dial to localhost:9090. Run both listeners in goroutines.
</implementation_plan>

<validation>
POST /api/v1/opportunities returns 201 with id and status=pending. GET /api/v1/opportunities/:id returns the created record. PUT /api/v1/opportunities/:id updates fields. grpcurl -plaintext localhost:9090 list returns sigma1.rms.v1.OpportunityService.
</validation>
<identity>
You are grizz working on subtask 3013 of task 3.
</identity>

<context>
<scope>
Write integration tests using pgx testcontainers covering the full opportunity-to-project flow, inventory conflict detection producing 409, and GDPR anonymization correctness.
</scope>
</context>

<implementation_plan>
Create tests/integration/rms_test.go. Use testcontainers-go to spin up a PostgreSQL container, apply migrations, and start an in-process RMS server. Test cases: (1) Create opportunity → convert to project → verify project row exists with correct opportunity_id. (2) CheckOut inventory → second CheckOut same item same window → assert gRPC AlreadyExists / HTTP 409. (3) ScoreLead with a local httptest vetting server returning score=10 → assert lead_score=YELLOW in DB. (4) GDPR delete → re-query opportunities → customer_id is NULL → audit row exists. Use t.Parallel() across independent test cases. Ensure go test -tags=integration ./tests/integration/... passes and coverage report shows >= 80% for internal packages.
</implementation_plan>

<validation>
go test -tags=integration -v ./tests/integration/... exits 0. All four scenario assertions pass. go test -cover ./... reports >= 80% total coverage. No data races detected with go test -race.
</validation>
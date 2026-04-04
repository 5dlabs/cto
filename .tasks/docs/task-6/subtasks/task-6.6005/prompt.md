Implement subtask 6005: Expose GET /api/validation/report/{run_id} Elysia endpoint

## Objective
Create the Elysia HTTP endpoint that serves the validation report by run_id, returning 200 with the full JSON report or 404 if the run_id is not found.

## Steps
1. Create `src/validation/routes.ts` (or add to existing route file).
2. Define the Elysia route:
   ```typescript
   app.get('/api/validation/report/:run_id', async ({ params }) => {
     const report = await getReport(params.run_id);
     if (!report) {
       return new Response(JSON.stringify({ error: 'Report not found' }), { status: 404 });
     }
     return report;
   });
   ```
3. Add response schema validation using Elysia's built-in schema support (or typebox) to ensure the response matches the ValidationReport type.
4. Register the route in the main app entrypoint.
5. Add error handling for malformed run_id parameters (return 400).

## Validation
Integration test: store a mock report via `storeReport`, then call `GET /api/validation/report/{run_id}` and assert 200 with valid JSON containing all required fields (`run_id`, `total_tasks`, `assigned_tasks`, `pending_tasks`, `issues`, `research_included`, `warnings`). Call with a nonexistent run_id and assert 404. Call with a malformed run_id and assert 400.
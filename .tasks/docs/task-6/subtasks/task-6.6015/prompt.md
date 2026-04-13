<identity>
You are nova working on subtask 6015 of task 6.
</identity>

<context>
<scope>
Write the complete vitest test suite covering upload handler, curation pipeline, caption generation, approval workflow, all four platform Effect services with retry behavior, publish orchestration, and GDPR endpoints.
</scope>
</context>

<implementation_plan>
Use vitest with @vitest/coverage-v8. Mock external APIs with vitest.fn() and msw (or fetch mock). Required test cases: (1) POST /api/v1/social/upload with 3 JPEGs → 202, draft created. (2) Curation pipeline: top photo selection with mocked OpenAI scores. (3) Caption generation: mocked OpenAI returns caption + 10 hashtags. (4) Approval workflow: state machine transitions, 409 on invalid transition. (5) Signal notification failure does not block curation. (6) Effect retry: Instagram mock fails 2 then succeeds → 3 calls total. (7) Publish orchestration: partial platform failure → correct per-platform status in response. (8) GDPR delete → event_id anonymized. (9) /health/ready with all deps healthy → 200. Measure coverage with `vitest --coverage`; enforce >= 80% line coverage in CI.
</implementation_plan>

<validation>
`npx vitest run --coverage` exits 0 with all tests passing. Coverage report shows >= 80% lines covered. CI pipeline enforces coverage threshold as a required gate before merge.
</validation>
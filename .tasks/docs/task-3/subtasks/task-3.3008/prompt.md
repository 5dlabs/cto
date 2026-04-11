<identity>
You are grizz working on subtask 3008 of task 3.
</identity>

<context>
<scope>
Replace the ScoreLead stub with the full weighted scoring algorithm that calls the vetting service HTTP endpoint, computes a composite score from event size, venue history, customer vetting score, and lead age, and returns GREEN/YELLOW/RED.
</scope>
</context>

<implementation_plan>
Create internal/opportunity/score.go. Define a ScoreInput struct populated from the opportunity row (event_date_start/end for size proxy, venue for history lookup, notes for flags, created_at for lead age). Make an HTTP GET to $VETTING_SERVICE_URL/api/v1/vetting/:org_id using net/http with a 2-second timeout; parse the JSON response for a numeric vetting_score field. Weights (to be confirmed via decision point): event_duration_days * 2 + vetting_score * 5 - lead_age_days * 0.1. Thresholds: >= 20 → GREEN, 10-19 → YELLOW, < 10 → RED. On vetting service HTTP error, treat vetting_score=0 and log a warning. Update the ScoreLead handler to call this function and persist the result to lead_score column. Unit tests in score_test.go: table-driven tests covering GREEN, YELLOW, RED cases with mocked HTTP responses using httptest.NewServer.
</implementation_plan>

<validation>
Unit tests: ScoreLead with mocked vetting_score=10 and high event size returns GREEN; vetting_score=5 returns YELLOW; vetting_score=0 and old lead returns RED. Integration: POST /api/v1/opportunities/:id/score with seeded vetting stub returns correct color. GET opportunity after scoring shows updated lead_score.
</validation>
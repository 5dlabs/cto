Implement subtask 6008: Implement approval workflow endpoints and Signal notification

## Objective
Implement POST /api/v1/social/drafts/:id/approve, POST /api/v1/social/drafts/:id/reject, and the Signal notification trigger that sends a preview link to the configured phone number when a draft reaches ready_for_approval.

## Steps
POST /api/v1/social/drafts/:id/approve: check current status; if 'ready_for_approval', transition to 'approved', set approved_by from JWT claims sub field, approved_at=now(). Return 200 {status:'approved'}. If status is not 'ready_for_approval', return 409 with message. POST /api/v1/social/drafts/:id/reject: set status='rejected', store optional reason in a new rejected_reason column (add migration). Return 200. Signal notification: triggered when curation pipeline sets status='ready_for_approval'. Use signal-cli REST API (POST {SIGNAL_CLI_URL}/v2/send, body: {message: 'New draft ready for approval: {APP_URL}/drafts/{draft_id}', recipients:[SIGNAL_PHONE_NUMBER]}). Wrap in try/catch — Signal failure must not block pipeline. Read SIGNAL_CLI_URL, SIGNAL_PHONE_NUMBER, APP_URL from env vars.

## Validation
POST approve on a ready_for_approval draft → 200, status=approved, approved_by=JWT sub, approved_at set. POST approve on an already-approved draft → 409. POST reject on any non-published draft → 200, status=rejected. Signal mock: verify POST to signal-cli is called with correct recipient and draft URL when curation completes. Signal mock returns 500 → pipeline still completes successfully.
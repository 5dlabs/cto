Implement subtask 5005: Implement Stage 2: Online Presence Check (LinkedIn and Website)

## Objective
Build the online presence stage that checks for a company's LinkedIn page existence and follower count, and probes for website existence via DNS/HTTP.

## Steps
1. Create `src/stages/online_presence.rs` module.
2. Define `OnlinePresenceResult` struct: linkedin_exists (bool), linkedin_followers (i32), website_exists (bool), raw_responses (serde_json::Value).
3. Implement LinkedIn check: attempt HTTP GET to `https://www.linkedin.com/company/{company_slug}/` with appropriate User-Agent header. Parse response for indicators of a valid company page (200 status, presence of follower count in meta tags or structured data). If official API is available, prefer that. Fallback: treat 200 as exists with followers=0 if count cannot be parsed.
4. Implement website check: given the company domain, perform DNS lookup (tokio::net::lookup_host) and HTTP HEAD request. Website exists if either returns successfully.
5. Run LinkedIn check and website check concurrently using `tokio::join!`.
6. Use `ResilientClient` with `linkedin` circuit breaker for the LinkedIn request.
7. On circuit open or failure, return linkedin_exists=false, linkedin_followers=0 with Unavailable note.
8. Capture all raw HTTP responses for audit trail.

## Validation
Unit tests with wiremock-rs: mock LinkedIn company page returning 200 with sample HTML containing follower count, verify linkedin_exists=true and followers parsed. Mock 404 for non-existent company. Mock DNS/HTTP for website existence check. Test concurrent execution of both checks. Verify graceful degradation when LinkedIn is unavailable.
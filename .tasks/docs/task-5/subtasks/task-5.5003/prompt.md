Implement subtask 5003: Implement LinkedIn online presence integration

## Objective
Build the integration module for LinkedIn API to assess a company's online presence including company page data, follower count, and activity signals.

## Steps
1. Create a `vetting::integrations::linkedin` module. 2. Define a trait `OnlinePresenceProvider` with method: `assess_presence(company_name: &str, domain: Option<&str>) -> Result<OnlinePresenceResult>`. 3. Implement LinkedIn API integration using OAuth2 client credentials flow; store client_id and client_secret in Kubernetes secrets. 4. Fetch company page data: follower count, post frequency, employee count range, company description completeness. 5. Compute an online_presence_score (0.0-1.0) based on: has_linkedin_page (0.2), follower_count > threshold (0.3), recent_activity within 30 days (0.3), profile_completeness (0.2). 6. Define OnlinePresenceResult struct with score, follower_count, last_activity_date, profile_completeness_pct. 7. Handle LinkedIn API pagination and rate limiting.

## Validation
Unit tests with mocked LinkedIn API responses for: complete profile, sparse profile, no page found, expired OAuth token refresh. Verify scoring algorithm produces expected scores for known input combinations.
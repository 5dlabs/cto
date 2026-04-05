Implement subtask 5003: Implement LinkedIn API integration for online presence analysis

## Objective
Build the LinkedIn API client module to retrieve company profile data including follower count, employee count, and company description for online presence scoring.

## Steps
1. Create a `sources/linkedin.rs` module implementing the `VettingSource` trait. 2. Implement OAuth 2.0 token management for LinkedIn API access (client credentials flow). Store client_id/client_secret in Kubernetes secrets. 3. Call LinkedIn Company API endpoints to retrieve: company name, description, follower_count, employee_count, industry, website_url, logo_url. 4. Parse responses into an `OnlinePresence` struct. 5. Handle LinkedIn-specific errors: expired tokens (auto-refresh), rate limits, private/unavailable profiles. 6. If the LinkedIn API requires specific partnership/approval, implement a fallback stub that returns partial data with a warning flag. 7. Write unit tests with mocked LinkedIn API responses.

## Validation
Unit tests pass for: successful company profile retrieval, token refresh on 401, rate limit handling, private profile graceful degradation. OnlinePresence struct correctly populated from mock data.
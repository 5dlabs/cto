Implement subtask 5004: Implement LinkedIn online presence check integration

## Objective
Build the HTTP client integration for LinkedIn API to assess the online presence and legitimacy of a business entity.

## Steps
1. In `integrations/linkedin.rs`, create a LinkedInClient struct with reqwest::Client and API credentials. 2. Implement methods: `search_company(name: &str) -> Result<LinkedInCompanyResult>`, `get_company_profile(company_id: &str) -> Result<LinkedInProfile>`, `assess_presence(org_name: &str) -> Result<OnlinePresence>`. 3. Define response DTOs for LinkedIn API responses. 4. Map API data into an `OnlinePresence` domain struct with fields: has_linkedin (bool), follower_count, employee_count_range, profile_completeness_score, last_activity_date, presence_score (0-100). 5. Handle OAuth2 token refresh if needed, and API errors/rate limits. 6. Use trait `OnlinePresenceChecker` for mockability.

## Validation
Unit tests with mocked LinkedIn responses verify correct parsing; presence_score calculation is validated against known inputs; error handling for auth failures and rate limits works correctly.
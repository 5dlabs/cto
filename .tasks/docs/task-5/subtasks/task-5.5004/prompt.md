Implement subtask 5004: Implement LinkedIn online presence integration module

## Objective
Build a standalone Rust module that integrates with the LinkedIn API to assess a company's online presence, including company page existence, follower count, employee count, and recent activity signals.

## Steps
1. Create src/integrations/linkedin.rs.
2. Define a trait `OnlinePresenceChecker` with async method `check_presence(company_name: &str, domain: Option<&str>) -> Result<OnlinePresence, VettingError>`.
3. OnlinePresence struct: found (bool), profile_url (Option<String>), follower_count (Option<i64>), employee_count_range (Option<String>), description (Option<String>), presence_score (f64 0.0-1.0).
4. Implement `LinkedInClient` struct with reqwest::Client and API credentials.
5. Use LinkedIn Marketing/Company API to search for company profiles and retrieve organization details.
6. Calculate a presence_score based on: profile exists (0.3), has description (0.1), follower_count tiers (0.3), employee_count tiers (0.3).
7. Handle OAuth2 token refresh, rate limits, and API errors.
8. Implement `MockOnlinePresenceChecker` for testing.

## Validation
Unit tests with mocked LinkedIn API responses verify correct presence scoring algorithm; edge cases (no profile found, incomplete data) produce appropriate scores; mock implementation returns configurable test data.
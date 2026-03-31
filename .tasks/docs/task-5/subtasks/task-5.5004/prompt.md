Implement subtask 5004: Implement LinkedIn online presence pipeline stage

## Objective
Build the HTTP client integration for checking a company's LinkedIn presence and extracting relevant signals (employee count, company age, activity) as the second vetting pipeline stage.

## Steps
1. Create a `clients/linkedin.rs` module.
2. Define a `LinkedInClient` struct wrapping reqwest::Client and API credentials.
3. Implement `check_presence(&self, company_name: &str, website: Option<&str>) -> Result<OnlinePresence>` that:
   - Calls the LinkedIn company search/lookup endpoint.
   - Extracts: company page exists (bool), employee count range, years active, recent activity score.
   - Populates the `OnlinePresence` struct with `linkedin_found: bool`, `employee_count: Option<u32>`, `activity_score: f64`.
4. Implement the `VettingStage` trait for `LinkedInStage`.
5. Handle OAuth token management if required by the LinkedIn API.
6. Handle graceful degradation: if LinkedIn API is unavailable, return a partial result with `confidence: low` rather than failing the entire pipeline.
7. Include mock implementation for testing.

## Validation
Unit tests with mocked HTTP responses verify correct parsing. Graceful degradation is tested: when the API returns errors, the stage returns a partial/low-confidence result instead of failing. The VettingStage trait contract is satisfied.
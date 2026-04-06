Implement subtask 5003: Implement LinkedIn API integration client

## Objective
Build an async HTTP client module for querying LinkedIn data to retrieve company profile information and employee signals for vetting purposes.

## Steps
1. Create `src/integrations/linkedin.rs` module.
2. Define request/response types for LinkedIn company profile lookup (via LinkedIn Marketing/Company API or a proxy service).
3. Use reqwest with OAuth2 bearer token authentication. Read LINKEDIN_ACCESS_TOKEN from Kubernetes secrets.
4. Implement `fetch_company_profile(company_name: &str) -> Result<LinkedInProfile, VettingError>` returning employee count, industry, founding year, specialties.
5. Handle OAuth token refresh if applicable, rate limiting, and API errors.
6. Map API responses into domain-specific structs for scoring consumption.
7. Add unit tests with mocked responses.

## Validation
Unit tests pass with mocked LinkedIn responses covering: successful profile fetch, company not found, authentication failure, rate limit exceeded. Response struct fields correctly populated from mock data.
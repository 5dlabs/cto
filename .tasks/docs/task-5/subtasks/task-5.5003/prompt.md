Implement subtask 5003: Implement LinkedIn data integration module

## Objective
Build a Rust module for retrieving company online presence and profile data from LinkedIn (via selected API or enrichment provider) for organization vetting.

## Steps
1. Create `src/integrations/linkedin.rs` module.
2. Define an `OnlinePresenceProvider` trait with async methods: `get_company_profile(org_name: &str) -> Result<LinkedInProfile>`, `get_employee_count(org_name: &str) -> Result<Option<u32>>`.
3. Implement the trait for the chosen LinkedIn data source (official API or third-party enrichment like Proxycurl). Use the API key from secrets.
4. Define internal types: LinkedInProfile { company_name, description, industry, employee_count, headquarters, website, specialties, followers_count, founded_year }.
5. Implement HTTP client calls with proper authentication headers.
6. Handle error cases: company not found, partial data, API errors.
7. Add structured logging and request tracing.
8. Design the trait so the implementation can be swapped once dp-5-2 is resolved.

## Validation
Write unit tests with mocked HTTP responses verifying correct parsing, graceful handling of missing fields, and error scenarios. Verify the trait can be implemented with different backends.
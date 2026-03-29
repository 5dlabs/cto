Implement subtask 5002: Implement customer vetting API endpoints

## Objective
Develop API endpoints for triggering the vetting pipeline and retrieving vetting results and credit signals for an organization.

## Steps
1. Implement handlers for `POST /api/v1/vetting/run` to initiate a vetting process.2. Implement handlers for `GET /api/v1/vetting/:org_id` to retrieve full vetting results.3. Implement handlers for `GET /api/v1/vetting/credit/:org_id` to retrieve specific credit signals.

## Validation
1. Use `curl` to call `POST /api/v1/vetting/run` with sample data and verify a 202 Accepted response.2. Use `curl` to call `GET /api/v1/vetting/:org_id` and `GET /api/v1/vetting/credit/:org_id` and verify structured (even if empty/mock) responses.
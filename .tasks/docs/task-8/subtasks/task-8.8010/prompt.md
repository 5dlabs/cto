Implement subtask 8010: Write unit and integration tests for all pages and components

## Objective
Develop comprehensive unit tests for individual components and integration tests for page-level data fetching, form submission, and widget interactions, targeting 80%+ code coverage.

## Steps
1. Set up testing infrastructure: Vitest (or Jest), React Testing Library, MSW (Mock Service Worker) for API mocking.
2. Unit tests for shared components:
   - shadcn/ui wrappers and custom components.
   - Effect Schema validators.
   - Utility functions.
3. Integration tests for each page:
   - Homepage: renders hero, fetches and displays featured equipment.
   - Equipment listing: search, filter, sort, pagination with mocked API.
   - Equipment detail: renders correct data, availability check, quote CTA navigation.
   - Quote builder: multi-step form navigation, validation, submission.
   - Portfolio: gallery rendering, filtering, lazy loading.
4. Integration tests for chat widget:
   - WebSocket connection mock, message send/receive, session persistence.
5. API integration tests:
   - TanStack Query hooks return correct data with Effect layers.
   - Error states are handled correctly.
   - Effect Schema validation rejects malformed responses.
6. Configure coverage reporting and verify ≥80% code path coverage.
7. Add tests to CI pipeline configuration.

## Validation
All tests pass; code coverage report shows ≥80% of code paths covered; MSW correctly mocks all API endpoints; each page has at least one integration test verifying data flow; form validation tests cover valid and invalid inputs; chat widget tests verify connection and messaging lifecycle.
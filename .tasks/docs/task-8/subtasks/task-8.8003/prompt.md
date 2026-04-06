Implement subtask 8003: Build self-service quote builder page (/quote) with Effect form validation

## Objective
Implement the quote builder allowing customers to select equipment, specify rental dates and location, and submit a quote request with full form validation using Effect.

## Steps
1. Create the /quote page with the chosen interaction pattern (multi-step wizard or single-page form, per dp-12 decision).
2. Implement equipment selection: allow users to search/browse and add equipment items to the quote, with quantity selectors.
3. Implement date range selection: rental start date, end date, with calendar UI component.
4. Implement location/delivery details: address input, delivery preferences.
5. Implement customer contact information fields: name, email, phone, company.
6. Build form validation using Effect schemas: validate all fields with typed errors, display inline validation messages.
7. Implement form submission: POST the quote request to the backend (Catalog or Finance API), show a loading state, and display confirmation or error.
8. Support pre-filling from /equipment/:id 'Add to Quote' navigation (read equipment from URL params or state).
9. Add a quote summary sidebar/section showing selected items, estimated pricing, and rental duration.

## Validation
Quote form renders all fields; validation prevents submission with missing/invalid data and shows inline errors; pre-filling from equipment page works; submission sends correct payload to API; confirmation displays on success; error state displays on failure.
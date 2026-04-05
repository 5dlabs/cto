Implement subtask 8005: Implement quote builder page (/quote) with form handling and API integration

## Objective
Build the quote request page with a multi-step form for selecting equipment, specifying rental details, customer information, and submitting the quote request to the backend.

## Steps
1. Create app/quote/page.tsx.
2. Implement a multi-step form:
   - Step 1: Equipment selection (search/browse, add to quote, specify quantities and date ranges).
   - Step 2: Delivery/pickup details (address, dates, special requirements).
   - Step 3: Customer information (name, company, email, phone).
   - Step 4: Review and submit.
3. Use Effect Schema for form validation at each step.
4. Pre-populate equipment if navigated from /equipment/:id with query params.
5. Submit quote request to the Finance/Quote API endpoint.
6. Display confirmation with quote reference number on success.
7. Handle submission errors with user-friendly messages.
8. Implement form state persistence (e.g., sessionStorage) so users don't lose progress on navigation.
9. Ensure all form fields have proper labels, error messages, and ARIA attributes.

## Validation
Multi-step form navigates correctly between steps; Effect Schema validation catches invalid inputs and displays error messages; equipment pre-population from query params works; form submits successfully and shows confirmation; form state persists across page navigation; all form fields are accessible with screen readers.
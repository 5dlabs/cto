Implement subtask 8005: Implement self-service quote builder page (/quote) with Effect.Schema validation

## Objective
Build the interactive quote builder page where customers can select equipment, specify rental dates, enter contact information, and submit a quote request with full Effect.Schema validation.

## Steps
1. Create `app/quote/page.tsx` for the quote builder.
2. Define the quote request schema using Effect.Schema: equipment selections (array of { equipmentId, quantity, startDate, endDate }), customer contact info (name, email, phone, company), optional notes.
3. Build a multi-step form or single-page form: Step 1 - Equipment Selection (search/browse and add items with quantities and dates), Step 2 - Contact Information, Step 3 - Review & Submit.
4. Implement equipment search within the builder using the catalog API (reuse TanStack Query hooks from 8003).
5. Implement real-time validation using Effect.Schema on each field and the overall form.
6. Submit the quote request to the backend API (sigma1_generate_quote endpoint or equivalent REST endpoint).
7. Display a confirmation view with quote reference number on successful submission.
8. Handle submission errors gracefully with retry options.

## Validation
Quote builder renders at `/quote`; users can search and add equipment items; date pickers work and validate date ranges; contact form validates all required fields via Effect.Schema; form submission sends correct payload to the API; confirmation view displays on success; validation errors display inline; submission errors show retry options.
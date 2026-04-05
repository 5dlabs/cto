Implement subtask 8005: Implement self-service quote builder page (/quote) with Effect form validation

## Objective
Build the quote builder page allowing users to select equipment, specify rental details, and submit a quote request, using Effect Schema for form validation.

## Steps
1. Create `app/quote/page.tsx` for the quote builder.
2. Define the quote form schema using Effect Schema: equipment selections (array of {equipmentId, quantity, duration}), contact info (name, email, phone, company), project details (location, start date, end date, project description).
3. Build a multi-step or single-page form:
   - Step 1: Equipment selection (search and add equipment items, set quantity and duration for each).
   - Step 2: Project details (location, dates, description).
   - Step 3: Contact information.
   - Step 4: Review and submit.
4. Validate each step using Effect Schema with real-time error messages.
5. On submit, call the quote creation API endpoint. Display success confirmation with quote reference number.
6. Handle pre-population from /equipment/:id CTA (read query params and pre-fill equipment).
7. Persist form state in session storage so users don't lose progress on navigation.

## Validation
Form renders at /quote. Equipment can be searched and added. Validation errors display for invalid inputs (empty required fields, invalid email, past dates). Valid submission calls the API and shows confirmation. Pre-population from equipment detail page works. Form state persists across page refreshes.
Implement subtask 8006: Build quote builder page (/quote) with multi-step form and Effect.Schema validation

## Objective
Implement the self-service quote builder as a multi-step form with equipment selection, rental details, contact information, and review/submit steps, all validated with Effect.Schema.

## Steps
1. Create `app/quote/page.tsx` with metadata.
2. Create `app/quote/quote-builder.tsx` client component with multi-step form state management.
3. Step 1 - Equipment Selection: searchable equipment selector (combobox), allow multiple items, show selected items with quantities. Pre-populate if `?equipmentId=` query param is present.
4. Step 2 - Rental Details: date range picker for rental period, delivery address input, special requirements textarea.
5. Step 3 - Contact Information: name, email, phone, company name fields.
6. Step 4 - Review & Submit: summary of all selections, editable (back buttons), submit button.
7. Define Effect.Schema validators for each step: `QuoteStep1Schema`, `QuoteStep2Schema`, `QuoteStep3Schema`, and a combined `QuoteRequestSchema`.
8. Validate each step on 'Next' click using Effect.Schema.decodeUnknown, display inline field errors using shadcn/ui form error styling.
9. On submit, call `useSubmitQuote()` mutation. Show loading state during submission, success confirmation with quote reference number, and error state with retry.
10. Implement step indicator/progress bar showing current step.
11. Persist form state to sessionStorage so refreshing doesn't lose progress.

## Validation
Multi-step form navigates forward and backward between steps. Each step validates on Next and shows inline errors for invalid fields. Pre-population from query param works. Submit sends correct payload to backend API. Success state shows quote reference. Session storage persists form state across page refresh. Effect.Schema rejects malformed input with descriptive error messages.
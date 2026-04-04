Implement subtask 8008: Build Quote Builder multi-step form

## Objective
Implement the `/quote` route with a multi-step form: Step 1 — select products with availability check, Step 2 — event details (date range, venue, contact info), Step 3 — review and submit. Form state managed with Effect, submission to backend API.

## Steps
1. Create `app/quote/page.tsx` — Client Component (heavy interactivity).
2. Step indicator: visual stepper (1-2-3) showing current step, using shadcn Tabs or custom stepper.
3. Step 1 — Product Selection:
   - Display products already in quote store (from 'Add to Quote' on product pages).
   - Allow adding more products via search/browse inline (mini catalog view).
   - Each QuoteLineItem component: product image, name, quantity selector, date range (from AvailabilityCalendar), day rate, line total.
   - Real-time availability check for each product + date range combination.
   - Remove product button.
4. Step 2 — Event Details:
   - Form fields: event name, date range (pre-filled from product dates), venue/location, contact name, email, phone.
   - Effect Schema validation on all fields: email format, phone format, required fields.
   - shadcn Form components with proper labels, error messages.
5. Step 3 — Review & Submit:
   - Summary: product list with quantities, dates, rates, subtotal. Event details summary. Estimated total.
   - Edit buttons to go back to step 1 or 2.
   - Submit button: calls `useSubmitQuote()` mutation.
   - Loading state during submission.
6. Confirmation:
   - After successful submit, show confirmation page with opportunity reference number.
   - 'Chat with Morgan about this quote' CTA.
7. Form state: manage entire multi-step form state in React state (or Effect Ref), persisted to localStorage so user doesn't lose progress on navigation.
8. URL state: optionally update URL hash (#step-1, #step-2, #step-3) for browser back/forward support.

## Validation
Integration test full flow: pre-populate quote store with 2 products, render `/quote`. Verify Step 1 shows both products with correct info. Fill Step 2 with valid event details, advance to Step 3. Verify review shows all data correctly. Mock submit API, click submit, verify API called with correct payload shape. Verify confirmation page shows reference number. Test validation: leave required fields empty in Step 2, verify error messages shown and cannot advance.
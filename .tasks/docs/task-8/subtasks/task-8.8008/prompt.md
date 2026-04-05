Implement subtask 8008: Build Quote Builder multi-step form with React Hook Form, dynamic line items, and real-time pricing

## Objective
Implement the `/quote` page with a 4-step form using React Hook Form v7: Step 1 (event details with date range picker), Step 2 (equipment selection with useFieldArray for dynamic line items), Step 3 (review with pricing and availability conflict highlighting), Step 4 (contact info and submission). Includes real-time price calculation and Effect Schema validation.

## Steps
1. Create `app/quote/page.tsx` as a client component.
2. Set up React Hook Form with a single form instance spanning all 4 steps. Use `@hookform/resolvers` with Effect Schema (or Zod) for per-step validation.
3. Step navigation component: horizontal stepper UI showing current step, completed steps, and upcoming steps. Validate current step before allowing navigation to next.
4. **Step 1 — Event Details**:
   - Fields: event name, date range (start/end date pickers), venue name, venue address, event type (dropdown: concert, corporate, wedding, festival, etc.).
   - Date range picker: use a shadcn/ui calendar-based date range component.
5. **Step 2 — Equipment Selection**:
   - `useFieldArray` for `items[]` where each item = { equipmentId, name, quantity, dayRate, subtotal }.
   - Equipment search/browse: inline search field that queries Equipment Catalog API, results shown as a dropdown/popover.
   - Click to add item → appends to useFieldArray.
   - Each line item: product name, quantity input (number stepper), day rate, subtotal (quantity × dayRate × numberOfDays), remove button.
   - Running total at bottom, recalculated reactively on any change.
   - Pre-populated items from QuoteProvider context (items added via 'Add to Quote' on product pages).
6. **Step 3 — Review**:
   - Display all line items in a read-only table.
   - For each item, call availability API for the selected date range. If conflict, highlight row in red/warning with message.
   - Show total price, number of rental days, item count.
   - Allow user to go back to Step 2 to modify.
7. **Step 4 — Contact Info + Submit**:
   - Fields: full name, email, phone, company (optional), additional notes (textarea).
   - Submit button: POST to quotes API endpoint with full form payload.
   - On success: show confirmation message with quote reference number.
   - On error: show error toast, keep form data.
8. Effect Schema definitions for each step's validation in `lib/schemas/quote.ts`.
9. Form state persisted to sessionStorage via QuoteProvider so refreshing doesn't lose data.

## Validation
Component test: render quote builder, fill Step 1 fields, advance to Step 2, add 3 items via useFieldArray, verify line items array has 3 entries with correct subtotals and total price. Test availability conflict: mock one item as unavailable for selected dates, verify Step 3 highlights it. Test form validation: try to advance from Step 1 with empty required fields, verify validation errors shown. Test submission: fill all steps, submit, verify POST request payload matches form data. Test sessionStorage persistence: fill Step 1, refresh page, verify data restored.
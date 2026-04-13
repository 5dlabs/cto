<identity>
You are blaze working on subtask 8009 of task 8.
</identity>

<context>
<scope>
Implement the self-service quote builder as a client component with pre-populated items from localStorage, customer info form with Effect.Schema validation, and RMS opportunity creation on submit.
</scope>
</context>

<implementation_plan>
1. Create app/quote/page.tsx — thin server wrapper. Main logic in components/quote/QuoteBuilder.tsx as 'use client'.
2. State: quoteItems (read from localStorage sigma1_quote_items on mount), eventDates ({from, to}), customerInfo ({name, email, org, phone}).
3. UI sections:
   - Quote line items table: Table component from shadcn. Columns: Product Name, Quantity (Input), Days (derived from dates), Day Rate, Subtotal. Allow removing items.
   - Event date picker: DayPicker range selector.
   - Customer info: Input fields for name, email, org, phone.
4. Validation using Effect.Schema before submit:
   - Define CustomerInfoSchema with S.String (non-empty), email pattern, phone optional.
   - On submit: S.decode(CustomerInfoSchema)(formData) — on ParseError, show field-level error messages.
5. Submit handler: calls runApiEffect(createOpportunity({ customer_id: email, line_items, event_date: from })).
6. Success state: hide form, show confirmation Card with opportunity_id and message 'Your quote has been submitted. Morgan will contact you shortly.'.
7. Error state: show Dialog with error message and retry button.
8. Empty state: if no items, show prompt 'Browse equipment to add items to your quote' with Button linking to /equipment.
</implementation_plan>

<validation>
Playwright: navigate to /quote with localStorage sigma1_quote_items pre-set — items appear in table. Submit form with invalid email — email field shows validation error, form does not submit. Submit with valid data and mocked API (MSW returning 201) — confirmation panel appears with opportunity ID. GET /quote with empty localStorage — empty state prompt is visible.
</validation>
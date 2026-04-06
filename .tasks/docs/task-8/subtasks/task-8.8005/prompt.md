Implement subtask 8005: Implement self-service quote builder with Effect form validation

## Objective
Build the interactive quote builder page at `/quote` where users can select equipment, specify rental dates, enter delivery details, and submit a quote request. Use Effect Schema for robust form validation.

## Steps
1. Create `app/quote/page.tsx` with a multi-step or single-page quote form.
2. Implement equipment selection: search/select from catalog (fetch from API), add multiple items with quantities.
3. Implement date range picker for rental period (start date, end date).
4. Implement delivery details fields: delivery address, site contact, special instructions.
5. Implement customer information fields: name, company, email, phone.
6. Use Effect `Schema` for form validation: define schemas for each section, validate on submit.
7. Display inline validation errors with clear messages.
8. On valid submission, POST to the Quote Engine API; display confirmation with quote number and summary.
9. Handle submission errors gracefully (API down, validation failures from backend).
10. Show a quote summary section that updates in real-time as items and dates are selected (calculate estimated pricing client-side if rate data is available).

## Validation
Quote builder renders at `/quote`; equipment can be searched and added; date picker selects valid ranges; form validation prevents submission with missing/invalid fields and shows inline errors; valid submission calls the Quote Engine API and displays confirmation; estimated pricing updates as items are added; submission error from API displays user-friendly message.
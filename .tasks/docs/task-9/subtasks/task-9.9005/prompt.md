Implement subtask 9005: Build Quote Builder tab with step-by-step wizard and submission

## Objective
Implement the Quote tab as a multi-step wizard: equipment selection (searchable list), date range picker, venue input, review screen, and API submission to create an opportunity.

## Steps
1. Create wizard state management using React context or zustand: track current step, selected equipment (array of {productId, quantity}), date range, venue, and contact info.
2. **Step 1 - Equipment Selector**: Searchable equipment list. Each item shows ProductCard with quantity stepper. Selected items shown as chips/badges at top. 'Next' button enabled when ≥1 item selected.
3. **Step 2 - Date Range**: Use `@react-native-community/datetimepicker` or Expo's native date picker. Select event start and end dates. Validate end > start.
4. **Step 3 - Venue Input**: Text input for venue name/address. Optional map integration or just free-text. Add any special instructions textarea.
5. **Step 4 - Review**: Summary screen showing all `QuoteLineItem` components, dates, venue. Edit buttons to jump back to specific steps. Total estimate if pricing is available.
6. **Submit**: POST to quotes API with full payload (products, dates, venue). Show loading state during submission. On success, navigate to confirmation screen with quote ID. On failure, show error with retry.
7. Implement progress indicator (step dots or progress bar) at top of wizard.
8. Support pre-population: if user tapped 'Add to Quote' from Equipment tab product detail, the product should appear pre-selected in Step 1.

## Validation
Wizard flow test: complete all steps with mock data, verify final submission payload contains correct product IDs, quantities, dates, and venue. Step validation: verify 'Next' is disabled when no equipment selected. Date validation: verify end-before-start shows error. Pre-population test: navigate from product detail with product context, verify Step 1 has product pre-selected. Review screen test: verify all QuoteLineItems render correctly.
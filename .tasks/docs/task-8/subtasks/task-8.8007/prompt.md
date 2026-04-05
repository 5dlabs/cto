Implement subtask 8007: Build AvailabilityCalendar component with date range picker

## Objective
Create the AvailabilityCalendar custom component that displays available/unavailable dates for a product and integrates a date range picker. Fetches availability data from the API with Effect Schema validation for date inputs.

## Steps
1. Create `components/custom/AvailabilityCalendar.tsx` — Client Component.
2. Props: `productId: string`, `onDateRangeSelect: (start: Date, end: Date) => void`.
3. Calendar display:
   - Use shadcn Calendar (Radix-based) or extend with react-day-picker for range selection.
   - Fetch availability via `useProductAvailability(productId, { month, year })` TanStack Query hook → `GET /api/v1/catalog/products/:id/availability?start=YYYY-MM-DD&end=YYYY-MM-DD`.
   - Color-code dates: green (available), red (unavailable/booked), gray (past dates).
   - Disable selection of unavailable dates.
4. Date range selection:
   - User clicks start date, then end date to form a range.
   - Validate with Effect Schema: start < end, start >= today, range within reasonable bounds (e.g., max 90 days).
   - Display validation errors inline.
5. Loading state: skeleton overlay on calendar while fetching availability.
6. Month navigation: prev/next month buttons trigger new availability fetch.
7. Integrate into Product Detail page (`/equipment/:id`): place below product info, selected date range feeds into 'Add to Quote' action.

## Validation
Unit test AvailabilityCalendar: mock availability API returning mix of available/unavailable dates. Verify available dates are selectable and unavailable dates are disabled. Test date range selection: click start and end date, verify `onDateRangeSelect` callback fires with correct dates. Test validation: select end date before start date, verify error message shown. Test month navigation: click next month, verify new API call made with updated date range.
Implement subtask 8007: Build Availability Calendar component with 90-day booking view

## Objective
Create a reusable availability calendar component that displays available/booked dates for the next 90 days for a given equipment item. Fetches availability data from the API and visually distinguishes available, booked, and partially-available dates.

## Steps
1. Create `components/sigma1/availability-calendar.tsx` as a client component.
2. Props: `equipmentId: string`, `onDateRangeSelect?: (start: Date, end: Date) => void`.
3. UI: 3-month calendar grid (current month + 2 next months), navigable.
   - Each day cell color-coded: green/default = available, red/muted = booked, yellow/warning = limited.
   - Legend explaining color coding.
4. Data fetching: `useEquipmentAvailability(equipmentId)` hook.
   - Calls availability API endpoint.
   - TanStack Query with stale time: 0 (no cache — availability is real-time critical).
   - Effect Schema validation on response (array of { date: string, status: 'available' | 'booked' | 'limited' }).
5. Interaction: users can select a date range (click start date, click end date) which highlights the range and calls `onDateRangeSelect`. This feeds into the quote builder.
6. Responsive: on mobile, show one month at a time with swipe/arrow navigation.
7. Accessible: each date cell has aria-label like "January 15, 2025 - Available" or "January 16, 2025 - Booked". Keyboard navigable with arrow keys.

## Validation
Component test: render calendar with mock availability data (mix of available/booked dates), verify correct color coding on cells. Test date range selection: click two dates, verify onDateRangeSelect callback fires with correct start/end. Test accessibility: each cell has correct aria-label. Test mobile: at 375px width, verify only one month is visible with navigation controls.
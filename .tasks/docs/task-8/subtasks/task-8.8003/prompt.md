Implement subtask 8003: Implement self-service quote builder with Effect form validation

## Objective
Build the /quote page with a multi-step quote builder form that allows customers to select equipment, specify event details, and submit a quote request, using Effect 3.x for typed form validation and submission.

## Steps
Step 1: Create the /quote route and design the multi-step form flow: Step 1 — Event details (date, location, event type, guest count); Step 2 — Equipment selection (search/browse catalog, add items with quantities); Step 3 — Review & submit. Step 2: Implement the event details form with Effect-based validation schemas: date must be future, location is required, guest count is positive integer. Step 3: Implement the equipment selection step: inline catalog search, item cards with quantity selectors, running total calculation, and item removal. Step 4: Implement the review step: summary of all selections, event details, estimated total, and editable sections. Step 5: Implement form submission via TanStack Query mutation + Effect — call the quote generation API endpoint, handle success (confirmation with quote ID), and error states (validation errors, server errors). Step 6: Implement form state persistence — save draft to localStorage so users don't lose progress on page refresh. Step 7: Add URL parameter support so equipment detail pages can deep-link to the quote builder with pre-selected items.

## Validation
Complete the full quote builder flow from event details → equipment selection → review → submit; validation errors display for invalid inputs; submission calls the API and displays a confirmation with quote ID; form state persists across page refresh; deep-link from equipment detail pre-selects the item.
Implement subtask 8005: Implement portfolio gallery with event photos and testimonials

## Objective
Build the /portfolio page displaying a gallery of past event photos and customer testimonials, with filtering by event type and a lightbox view for images.

## Steps
Step 1: Create the /portfolio route with a server component that fetches portfolio data (event photos and testimonials) from the backend API or static data source. Step 2: Implement the photo gallery UI: masonry or grid layout with responsive columns, lazy-loaded images via Next.js Image component (optimized from object storage). Step 3: Implement event type filtering (weddings, corporate, festivals, etc.) with animated transitions between filter states. Step 4: Implement a lightbox/modal view for full-size image viewing with next/previous navigation. Step 5: Implement the testimonials section: customer quotes with name, event type, and star rating, displayed as cards or a carousel. Step 6: Add Schema.org Review structured data for testimonials. Step 7: Implement loading skeletons for the gallery while images load.

## Validation
Portfolio page loads and displays event photos in a responsive grid; filters correctly show/hide photos by event type; lightbox opens on image click with navigation; testimonials display with customer info; images are lazy-loaded and optimized via Next.js Image; Schema.org Review data is present.
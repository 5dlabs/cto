Implement subtask 8007: Implement AI-native features: llms.txt, llms-full, and Schema.org structured data

## Objective
Create the /llms.txt and /llms-full routes serving machine-readable site descriptions for AI crawlers, and ensure comprehensive Schema.org structured data is present across all pages.

## Steps
Step 1: Create the /llms.txt route as a plain text response describing the Sigma-1 site's purpose, equipment catalog structure, and key capabilities in the llms.txt format. Step 2: Create the /llms-full route with a comprehensive machine-readable description including all equipment categories, pricing structures, service areas, and contact methods. Step 3: Audit and enhance Schema.org structured data across pages: Organization (home page), Product (equipment detail), ItemList (equipment listing), Review (portfolio testimonials), LocalBusiness (contact info). Step 4: Add JSON-LD script tags to the head of relevant pages via Next.js metadata API. Step 5: Validate structured data using Google's Rich Results Test or Schema.org validator.

## Validation
/llms.txt returns plain text with accurate site description; /llms-full returns comprehensive machine-readable content; Schema.org validation passes for all page types with no errors; JSON-LD is present in page source for equipment, portfolio, and home pages.
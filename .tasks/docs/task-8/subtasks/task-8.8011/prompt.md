<identity>
You are blaze working on subtask 8011 of task 8.
</identity>

<context>
<scope>
Create the /llms.txt and /llms-full route handlers for AI crawler discoverability, and inject Schema.org Organization/LocalBusiness JSON-LD into the root layout.
</scope>
</context>

<implementation_plan>
1. Create app/llms.txt/route.ts as a Next.js Route Handler:
   - Returns Response with Content-Type: text/plain.
   - Body: multi-line plain text including: company name (Sigma-1 / Perception Events), description of services (event lighting, visual production, equipment rental), contact email, website URL, and a list of equipment categories.
   - Keep under 2000 characters.
2. Create app/llms-full/route.ts as a Route Handler:
   - Returns Response with Content-Type: application/json.
   - Fetches categories and products server-side (or uses static cached data).
   - Returns JSON: { company, services[], equipment_categories[], product_count, contact }.
3. In app/layout.tsx, add a <script type='application/ld+json'> tag in <head> with Schema.org JSON-LD:
   ```json
   {
     "@context": "https://schema.org",
     "@type": ["Organization", "LocalBusiness"],
     "name": "Sigma-1 / Perception Events",
     "description": "Event lighting and visual production",
     "url": "https://sigma1.com",
     "contactPoint": { "@type": "ContactPoint", "contactType": "sales" }
   }
   ```
4. Add <link rel='ai-content' href='/llms.txt' /> in layout <head>.
</implementation_plan>

<validation>
GET /llms.txt returns HTTP 200 with Content-Type: text/plain and body contains 'Sigma-1'. GET /llms-full returns HTTP 200 with Content-Type: application/json and parseable JSON containing company and equipment_categories fields. View page source of / — JSON-LD script tag is present with @type containing 'Organization'.
</validation>
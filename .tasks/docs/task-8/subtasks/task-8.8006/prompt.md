<identity>
You are blaze working on subtask 8006 of task 8.
</identity>

<context>
<scope>
Implement the homepage with a hero section, Sigma-1 value proposition copy, and CTA buttons linking to /equipment and /quote.
</scope>
</context>

<implementation_plan>
1. Create app/page.tsx as a Server Component.
2. Hero section: full-width dark background, centered content. H1: 'Illuminate Every Event' (or approved brand copy). Subheadline: 2-line description of Sigma-1's lighting and visual production services.
3. CTA buttons: primary Button linking to /equipment ('Browse Equipment'), secondary outline Button linking to /quote ('Get a Quote').
4. Value props section below hero: 3-column grid of Card components, each with an icon (lucide-react), title, and 1-sentence description. Topics: Equipment Range, Real-Time Availability, Instant Quotes.
5. Contact anchor section (#contact): email address, phone, and location. Keep minimal for v1.
6. Apply sigma1 brand colors from design tokens (background, accent electric blue for CTA).
7. Add og:image meta tag referencing a static placeholder image for social sharing.
</implementation_plan>

<validation>
Playwright: GET / on 1280px viewport — h1 is visible, two CTA buttons present with correct href attributes. GET / on 375px viewport — h1 and CTAs are visible without horizontal scroll. `next build` produces no TypeScript errors for this file.
</validation>
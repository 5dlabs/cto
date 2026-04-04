Implement subtask 8004: Build Home page with hero section, CTAs, and Schema.org Organization markup

## Objective
Implement the `/` (Home) route as a statically generated page with a video/image hero background showcasing lighting production, value proposition text, CTA buttons ('Browse Equipment', 'Get a Quote', 'Chat with Morgan'), and Schema.org Organization JSON-LD.

## Steps
1. Create `app/page.tsx` as a Server Component (static generation).
2. Hero section:
   - Full-viewport height with video background (or high-quality image fallback) showing lighting/production work.
   - Overlay gradient for text readability.
   - H1 with company tagline, supporting paragraph.
   - CTA buttons using shadcn Button: 'Browse Equipment' → `/equipment`, 'Get a Quote' → `/quote`, 'Chat with Morgan' → triggers chat widget open.
3. Below hero: brief sections for Services overview, Featured Equipment (static or fetched at build), Testimonials/social proof.
4. SEO metadata in `generateMetadata`: title, description, OpenGraph (og:image, og:title, og:description, og:url), Twitter Card.
5. Schema.org JSON-LD script in head: Organization type with name, url, logo, description, contactPoint.
6. Use Next.js `<Image>` component for all images with proper alt text, sizes, priority for hero image.
7. Responsive layout: hero text and CTAs stack vertically on mobile, side-by-side on desktop.

## Validation
Integration test: render Home page, verify H1 text present, all 3 CTA buttons rendered with correct hrefs. Verify Schema.org JSON-LD script tag contains Organization type with required fields. Verify OpenGraph meta tags present in document head. Responsive test at 375px and 1440px widths.
Implement subtask 8004: Implement Home page with hero section, featured equipment carousel, and CTAs

## Objective
Build the `/` home page with hero section (video/image background showcasing lighting setups), value proposition messaging, CTA buttons (Browse Equipment, Get a Quote, Chat with Morgan), featured equipment carousel, and testimonial section. Server-rendered static content.

## Steps
1. Create `app/page.tsx` as a Server Component (static rendering).
2. Hero section:
   - Full-viewport-height section with background media (image or video — use a placeholder optimized image initially).
   - Overlay with headline: tagline like "One platform, one conversation".
   - Three CTA buttons: Browse Equipment (→ /equipment), Get a Quote (→ /quote), Chat with Morgan (triggers chat widget open).
   - The 'Chat with Morgan' CTA needs a client component wrapper to dispatch a custom event or call a context method to open the chat widget.
3. Featured equipment carousel:
   - Horizontal scrollable card carousel (use CSS scroll-snap or a lightweight approach — no heavy carousel library).
   - Each card: product image, name, day rate, category badge.
   - Data: fetch from Equipment Catalog API at build time (or SSR), limit to ~8 featured/popular items.
4. Testimonial section:
   - 2-3 static testimonial cards with quote, author name, company.
   - Simple grid layout.
5. Add SEO metadata via Next.js `metadata` export: title, description, Open Graph tags, Schema.org Organization JSON-LD.
6. Optimize images with `next/image` component, proper sizing, and priority loading for hero image.

## Validation
Render the home page and verify: hero section is full viewport height, all 3 CTA buttons are present and link to correct routes, featured equipment carousel renders cards with images and prices, Schema.org JSON-LD is present in page source. Lighthouse performance score >= 90.
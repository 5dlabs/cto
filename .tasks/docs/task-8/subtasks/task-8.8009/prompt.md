Implement subtask 8009: Implement accessibility (WCAG 2.1 AA), performance optimization (LCP < 2s), and Schema.org SEO

## Objective
Audit and enhance the entire site for WCAG 2.1 AA accessibility compliance, optimize performance to achieve LCP under 2 seconds, and verify Schema.org structured data across all pages.

## Steps
1. Accessibility audit:
   - Run axe-core or similar tool on all pages.
   - Verify color contrast ratios meet AA standards (4.5:1 for text, 3:1 for large text).
   - Ensure all interactive elements are keyboard navigable with visible focus indicators.
   - Verify all images have meaningful alt text.
   - Ensure form fields have associated labels and error messages are announced.
   - Test with screen reader (NVDA/VoiceOver) on key flows.
   - Add skip navigation link.
   - Verify ARIA attributes are used correctly.
2. Performance optimization:
   - Analyze with Lighthouse and Web Vitals.
   - Optimize images: use next/image with proper sizing, formats (WebP/AVIF), and lazy loading.
   - Implement code splitting and dynamic imports for heavy components (chat widget, gallery).
   - Optimize fonts: use next/font with preloading.
   - Review and optimize TanStack Query caching strategies.
   - Ensure LCP element loads within 2 seconds on 3G simulation.
3. SEO verification:
   - Validate Schema.org structured data on all pages using Google's Rich Results Test.
   - Verify sitemap.xml and robots.txt are correctly generated.
   - Confirm canonical URLs and Open Graph tags on all pages.

## Validation
axe-core reports zero critical or serious accessibility violations on all pages; Lighthouse accessibility score ≥ 90; LCP < 2s on desktop and < 2.5s on mobile (simulated 3G); Schema.org validation passes with no errors on all pages; keyboard navigation works through all interactive flows; screen reader testing confirms all content is announced correctly.
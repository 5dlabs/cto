Implement subtask 8007: Accessibility audit, responsive design QA, and Lighthouse performance optimization

## Objective
Ensure the entire site passes accessibility checks, is fully responsive across devices, and achieves a Lighthouse score above 90 in all categories.

## Steps
1. Run axe-core or similar automated accessibility tool across all pages; fix all critical and serious violations.
2. Verify keyboard navigation works on all interactive elements (chat widget, forms, navigation, filters).
3. Ensure proper ARIA labels, roles, and landmarks on all custom components.
4. Verify color contrast ratios meet WCAG AA standards.
5. Test responsive design on mobile (375px), tablet (768px), and desktop (1280px+) breakpoints.
6. Run Lighthouse CI on all pages; identify and fix performance bottlenecks (image optimization, bundle splitting, font loading, unused CSS).
7. Implement lazy loading for below-fold content and optimize LCP (Largest Contentful Paint).
8. Minimize CLS (Cumulative Layout Shift) by setting explicit dimensions on images and dynamic content.
9. Ensure all pages achieve Lighthouse scores >90 for Performance, Accessibility, Best Practices, and SEO.
10. Document any known accessibility limitations.

## Validation
axe-core reports zero critical/serious violations; keyboard-only navigation completes all major user flows; Lighthouse CI scores >90 in all categories on all pages; responsive layout renders correctly at 375px, 768px, and 1280px+ breakpoints; no horizontal scrolling on mobile.
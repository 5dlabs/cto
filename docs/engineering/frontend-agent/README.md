# Blaze Frontend Agent - User Guide

> 🎨 **Automated UI/UX development with live previews and comprehensive testing**

## Overview

Blaze is your AI-powered Frontend Agent that automatically generates production-ready React applications from text descriptions. Every implementation includes:

- ✅ **React 19 + TypeScript + Next.js 15** codebase
- ✅ **shadcn/ui** component library
- ✅ **Live preview URL** (hosted on Kubernetes + Ngrok)
- ✅ **Multi-viewport screenshots** (mobile, tablet, desktop)
- ✅ **E2E tests** with Playwright
- ✅ **Accessibility audit** (WCAG AA)
- ✅ **Performance testing** (Core Web Vitals)

---

## Quick Start

### 1. Create a Frontend Task

Create a new GitHub issue with the `agent-blaze` label:

**Example Issue**:
```markdown
Title: Create dashboard landing page

Labels: agent-blaze

---

## Requirements
- Responsive dashboard layout
- Header with logo and navigation
- Hero section with CTA button
- Stats cards (4 columns showing metrics)
- Footer with social links

## Design Preferences
- Modern, clean aesthetic
- Blue primary color (#3B82F6)
- Mobile-first responsive design
- Card-based layout with subtle shadows

## Acceptance Criteria
- WCAG AA accessible
- Lighthouse performance >90
- Works seamlessly on mobile, tablet, desktop
- All interactive elements keyboard-navigable
```

### 2. Blaze Gets to Work

Within **~5 minutes**, Blaze will:

1. 🎨 Generate UI design via v0 AI
2. ⚙️ Initialize Next.js project with shadcn/ui
3. 🚀 Deploy to Kubernetes staging
4. 🌐 Create live preview URL (Ngrok)
5. 🎭 Run Playwright tests + capture screenshots
6. 📝 Create GitHub PR with everything

### 3. Review the PR

Your PR will include:

#### 🌐 Live Preview URL
Click the live demo link to interact with the implementation in real-time:
```
https://blaze-task-123-xyz.ngrok.app
```

- ✅ **Fully functional** - not just mockups
- ✅ **Responsive** - test on any device
- ✅ **Live updates** - every push updates the preview
- ✅ **Available until merge** - auto-cleanup after

#### 📸 Screenshots
Visual documentation at three viewports:
- Desktop (1920×1080)
- Tablet (768×1024)
- Mobile (375×667)

#### ✅ Test Results
- Playwright E2E tests
- Accessibility audit (0 violations)
- Performance metrics (Core Web Vitals)
- Visual regression results

### 4. Automated Quality Gates

**Cleo** (Code Quality Agent) will:
- ✅ ESLint + TypeScript + Prettier
- ✅ jsx-a11y accessibility checks
- ✅ Unit test coverage (>80%)
- ✅ React hooks compliance

**Tess** (QA Agent) will:
- ✅ Extended E2E tests
- ✅ Cross-browser testing
- ✅ Visual regression comparison
- ✅ Performance benchmarking

### 5. Final Approval & Merge

Once Cleo + Tess approve (or after you make requested changes), you can:
- 👀 Review the live preview one last time
- ✅ Approve and merge the PR
- 🎉 Staging namespace auto-cleans up

---

## Task Description Best Practices

### ✅ DO: Be Specific

```markdown
## Requirements
- Dashboard with 4 metric cards showing:
  - Total Users (with trend indicator)
  - Monthly Revenue (formatted as currency)
  - Conversion Rate (as percentage)
  - Active Sessions (real-time counter)
- Each card should have an icon, value, label, and trend arrow
- Cards should be in a 2×2 grid on mobile, 1×4 on desktop
```

### ❌ DON'T: Be Vague

```markdown
Make a dashboard with some cards
```

### ✅ DO: Include Design Preferences

```markdown
## Design Preferences
- Minimalist aesthetic inspired by Linear
- Neutral color palette (slate + blue accents)
- Generous whitespace (1.5rem padding)
- Subtle animations on hover (scale 1.02)
- Inter font family
```

### ❌ DON'T: Skip Design Details

```markdown
Make it look nice
```

### ✅ DO: Specify Interactions

```markdown
## Interactions
- CTA button should show loading spinner on click
- Form inputs should show inline validation errors
- Mobile menu should slide in from right with overlay
- Cards should be clickable and navigate to detail pages
```

### ❌ DON'T: Assume Blaze Knows

```markdown
Add some buttons
```

---

## Understanding the Live Preview

### How It Works

1. **Kubernetes Deployment**: Each task gets its own isolated namespace
   ```
   Namespace: blaze-staging-task-123
   Pod: frontend-preview
   Service: frontend-preview:80
   ```

2. **Ngrok Ingress**: Exposes the deployment with a public URL
   ```
   Domain: blaze-task-123-xyz.ngrok.app
   Protocol: HTTPS (automatic TLS)
   ```

3. **Auto-Updates**: Every push to the PR branch updates the deployment
   ```
   Push commit → Argo detects → Redeploys → Same URL, new code
   ```

### Sharing the Preview

The live preview URL can be shared with:
- ✅ Team members for feedback
- ✅ Product managers for approval
- ✅ Designers for design review
- ✅ Stakeholders for demos

**Note**: URLs are publicly accessible but:
- 🔒 Unique, hard-to-guess subdomain
- ⏰ Auto-expire after PR merge (cleanup)
- 🚫 No production data or secrets

---

## Reviewing Screenshots

### What to Look For

#### Layout & Spacing
- Consistent padding and margins
- Proper alignment of elements
- Responsive breakpoints working correctly

#### Typography
- Readable font sizes (min 16px body text)
- Proper heading hierarchy (h1, h2, h3)
- Consistent line heights

#### Colors & Contrast
- WCAG AA compliant (4.5:1 minimum)
- Hover states clearly visible
- Focus indicators present

#### Components
- Buttons have proper states (default, hover, active, disabled)
- Forms show validation states
- Loading states implemented
- Error states designed

### Mobile-First Checklist

When reviewing mobile screenshots:
- [ ] Text is readable without zooming
- [ ] Touch targets are at least 44×44px
- [ ] No horizontal scrolling
- [ ] Navigation is accessible (hamburger menu works)
- [ ] Forms are easy to complete on mobile

---

## Testing the Live Preview

### Manual Testing Checklist

#### Functionality
- [ ] All links work correctly
- [ ] Forms submit successfully
- [ ] Buttons trigger expected actions
- [ ] Navigation works across all pages

#### Responsiveness
- [ ] Resize browser window (375px → 1920px)
- [ ] Test on actual mobile device
- [ ] Test on tablet (landscape + portrait)
- [ ] No layout breaks at any viewport

#### Accessibility
- [ ] Tab through all interactive elements
- [ ] Screen reader announces content correctly
- [ ] Skip links work
- [ ] ARIA labels present where needed

#### Performance
- [ ] Page loads quickly (<3 seconds)
- [ ] No layout shifts during load (CLS)
- [ ] Images load progressively
- [ ] Smooth interactions (60fps)

### Testing on Mobile Devices

**Option 1: Direct Access**
```
Open browser on mobile → Enter ngrok URL
```

**Option 2: QR Code**
```bash
# Generate QR code for easy mobile access
qrencode -t UTF8 "https://blaze-task-123-xyz.ngrok.app"
```

Scan with phone camera to open instantly.

---

## Common Scenarios

### Scenario 1: Simple Landing Page

**Task Description**:
```markdown
Create a SaaS landing page with:
- Hero section with headline, subheadline, CTA
- Features section (3 columns with icons)
- Pricing table (3 tiers)
- Footer with links

Design: Clean, modern, blue/white color scheme
```

**Expected Timeline**:
- Blaze implementation: ~5 minutes
- Cleo review: ~3 minutes
- Tess QA: ~10 minutes
- **Total: ~18 minutes**

### Scenario 2: Dashboard UI

**Task Description**:
```markdown
Create an admin dashboard with:
- Sidebar navigation (collapsible)
- Top bar with search and profile menu
- Main content area with:
  - 4 metric cards
  - Line chart (placeholder data)
  - Recent activity table
  
Design: Dark mode, purple/pink gradient accents
```

**Expected Timeline**:
- Blaze implementation: ~6 minutes (more complex)
- Cleo review: ~4 minutes
- Tess QA: ~12 minutes
- **Total: ~22 minutes**

### Scenario 3: Multi-Page Application

**Task Description**:
```markdown
Create a blog application with:
- Home page (article grid)
- Article detail page (with comments UI)
- About page (team grid)
- Contact page (form)
- Shared header and footer

Design: Minimalist, typography-focused
```

**Expected Timeline**:
- Blaze implementation: ~8 minutes (multiple pages)
- Cleo review: ~5 minutes
- Tess QA: ~15 minutes
- **Total: ~28 minutes**

---

## Troubleshooting

### Issue: No PR Created After 10 Minutes

**Possible Causes**:
1. Missing `agent-blaze` label on issue
2. v0 API rate limit hit
3. Kubernetes namespace creation failure

**Resolution**:
```bash
# Check workflow status
kubectl get workflows -n agent-platform | grep blaze

# View workflow logs
kubectl logs -n agent-platform <workflow-pod-name>

# Check Argo Events sensor
kubectl describe sensor -n agent-platform blaze-task-sensor
```

### Issue: Live Preview URL Not Loading

**Possible Causes**:
1. Next.js build failure
2. Ngrok tunnel not established
3. Pod not ready

**Resolution**:
```bash
# Check deployment status
kubectl get pods -n blaze-staging-task-123

# Check pod logs
kubectl logs -n blaze-staging-task-123 deployment/frontend-preview

# Check Ngrok ingress
kubectl get ingress -n blaze-staging-task-123

# Describe domain status
kubectl describe domain -n blaze-staging-task-123
```

### Issue: Screenshots Missing from PR

**Possible Causes**:
1. Playwright tests failed
2. GitHub API rate limit
3. Screenshot upload failure

**Resolution**:
```bash
# Check Playwright test results
# (Available in PR as artifacts)

# Check workflow logs for screenshot step
kubectl logs -n agent-platform <workflow-pod> -c blaze-agent

# Verify test-results/ directory in PVC
kubectl exec -it -n agent-platform <workflow-pod> -- ls -la /workspace/screenshots/
```

### Issue: Cleo Requests Changes

**Common Issues**:
- TypeScript errors
- ESLint violations
- Missing accessibility attributes
- Low test coverage

**Resolution**:
Blaze will automatically enter a remediation loop:
1. Parse Cleo's feedback
2. Fix identified issues
3. Push updated commit
4. Request re-review

**Manual Override** (if needed):
You can push fixes manually to the PR branch, and Cleo will re-review.

### Issue: Tess QA Fails

**Common Issues**:
- Visual regression diffs
- Broken interactions
- Accessibility violations
- Performance below targets

**Resolution**:
1. Review Tess's test results (in PR comments)
2. Check specific failure screenshots
3. Test locally against live preview URL
4. Push fixes to PR branch

---

## Advanced Usage

### Custom Design System

If you want Blaze to use your existing design system:

```markdown
## Design System
Use the design tokens from `/design-system.json`:
- Colors: From `colors.primary` and `colors.neutral`
- Typography: Font family from `typography.fontFamily`
- Spacing: Use `spacing.scale` values
- Border radius: `borderRadius.default`

shadcn/ui components should be customized to match these tokens.
```

### Third-Party Integrations

Request specific integrations in your task:

```markdown
## Integrations
- Analytics: Add Plausible snippet to layout
- Maps: Use Mapbox GL JS for location map
- Forms: Integrate with Resend for contact form submission
```

### Performance Budget

Set specific performance targets:

```markdown
## Performance Requirements
- First Contentful Paint: <1.5s
- Largest Contentful Paint: <2.0s
- Total Blocking Time: <200ms
- Cumulative Layout Shift: <0.1
- Bundle size: <250KB (gzipped)
```

---

## Best Practices

### 1. Start Simple, Iterate

**Phase 1**: Basic layout and structure
```markdown
Create a dashboard landing page with header, hero, and footer
```

**Phase 2**: Add interactivity
```markdown
Add a modal dialog for the CTA button with a signup form
```

**Phase 3**: Enhance with data
```markdown
Connect the stats cards to the `/api/metrics` endpoint
```

### 2. Provide Visual References

Include links to design inspiration:
```markdown
## Design Inspiration
- Similar to: https://linear.app
- Color scheme: https://coolors.co/palette-link
- Typography: Use Inter font like https://vercel.com
```

### 3. Specify Components Upfront

If you know you need specific shadcn/ui components:
```markdown
## shadcn/ui Components Needed
- Dialog (for modal)
- Form + Input (for search)
- Table (for data grid)
- Tabs (for navigation)
```

This helps Blaze install everything in one pass.

### 4. Define Data Structure

If your UI displays data, define the structure:
```markdown
## Data Structure
```typescript
interface DashboardMetrics {
  totalUsers: number;
  revenue: number;
  conversionRate: number;
  activeSessions: number;
}
```

Use placeholder data for the preview.
```

---

## FAQ

### How long does Blaze take?

Typically **4-6 minutes** from issue creation to PR with live preview.

### Can I request changes?

Yes! Comment on the PR with specific feedback, and Blaze will update the implementation.

### How many revisions can I request?

Unlimited. Blaze will iterate until the implementation meets requirements.

### Can I use Blaze for non-React projects?

Currently, Blaze only supports React (Next.js 15). Other frameworks may be added in future phases.

### What happens after PR merge?

The staging namespace is automatically deleted, and the Ngrok URL becomes inactive. The code is merged to main.

### Can I extend the live preview lifetime?

By default, previews are cleaned up after merge. To extend, comment on the PR:
```
@blaze keep-preview 7d
```

This will keep the preview active for 7 days post-merge.

### How do I report a bug with Blaze?

Create an issue with the `blaze-bug` label:
```markdown
Title: Blaze generated incorrect layout

Labels: blaze-bug

## Issue
Blaze created a 3-column grid instead of requested 4-column.

## Expected
4 columns on desktop, 2 on tablet, 1 on mobile.

## Task ID
task-123

## PR Link
https://github.com/org/repo/pull/456
```

---

## Examples

### Example 1: Portfolio Site

**Issue**:
```markdown
Title: Create portfolio landing page

Labels: agent-blaze

---

## Requirements
- Hero section with name, title, brief intro
- Projects section (6 project cards in grid)
- Skills section (tech stack with icons)
- Contact section (email, GitHub, LinkedIn links)

## Design
- Dark theme (slate-900 background)
- Accent color: Emerald-500
- Modern, portfolio-style layout
- Smooth scroll navigation

## Acceptance Criteria
- Mobile-first responsive
- Lighthouse performance >95
- WCAG AA compliant
- Smooth animations on scroll
```

**Result**: https://blaze-task-789-abc.ngrok.app

### Example 2: E-commerce Product Page

**Issue**:
```markdown
Title: Create product detail page

Labels: agent-blaze

---

## Requirements
- Product image gallery (4 images, main + thumbnails)
- Product info section:
  - Title, price, description
  - Size selector (S, M, L, XL)
  - Color selector (visual swatches)
  - Quantity selector
  - Add to Cart button
- Product tabs (Description, Specs, Reviews)
- Related products carousel (4 items)

## Design
- Clean, e-commerce style
- Primary color: Blue-600
- Card-based layout
- Product images should be large and prominent

## Acceptance Criteria
- Image gallery with zoom on hover
- Size/color selections update URL
- Add to Cart shows success toast
- Mobile-optimized layout
```

**Result**: https://blaze-task-790-def.ngrok.app

---

## Getting Help

### Documentation
- [Architecture Overview](./architecture.md)
- [PRD (Product Requirements)](./prd.md)
- [Kubernetes Deployment Guide](../../../infra/README.md)

### Support Channels
- GitHub Issues: `blaze-bug` or `blaze-question` labels
- Slack: `#agent-platform` channel
- Email: dev@5dlabs.io

### Monitoring
- Grafana: [Blaze Dashboard](http://grafana.local/d/blaze)
- Argo Workflows: [Workflow UI](http://argo-workflows.local)
- Kubernetes: `kubectl get pods -n agent-platform -l agent=blaze`

---

**Last Updated**: 2025-10-18  
**Version**: 1.0  
**Maintainer**: CTO @ 5D Labs


/**
 * Feature flags for the marketing site.
 * Toggle these to show/hide features before public launch.
 *
 * To re-enable features, simply set the flag to `true`.
 */

export const featureFlags = {
  /**
   * Show the "Start Now" button that links to app.5dlabs.ai
   * Set to true when ready for public app access
   */
  showStartNowButton: false,

  /**
   * Show the pricing page link in navigation
   * Set to true when pricing is finalized for public viewing
   */
  showPricingLink: false,
} as const;

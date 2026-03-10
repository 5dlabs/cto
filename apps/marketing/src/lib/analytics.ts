"use client";

declare global {
  interface Window {
    dataLayer?: unknown[];
    gtag?: (...args: unknown[]) => void;
    posthog?: { capture?: (eventName: string, properties?: Record<string, unknown>) => void };
    plausible?: (eventName: string, options?: { props?: Record<string, unknown> }) => void;
    umami?:
      | ((eventName: string, properties?: Record<string, unknown>) => void)
      | { track?: (eventName: string, properties?: Record<string, unknown>) => void };
  }
}

export function trackEvent(eventName: string, properties: Record<string, unknown> = {}) {
  if (typeof window === "undefined") {
    return;
  }

  if (typeof window.gtag === "function") {
    window.gtag("event", eventName, properties);
  }

  if (typeof window.posthog?.capture === "function") {
    window.posthog.capture(eventName, properties);
  }

  if (typeof window.plausible === "function") {
    window.plausible(eventName, { props: properties });
  }

  if (typeof window.umami === "function") {
    window.umami(eventName, properties);
  } else if (typeof window.umami?.track === "function") {
    window.umami.track(eventName, properties);
  }
}

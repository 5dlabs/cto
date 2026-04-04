"use client";

import type { MouseEvent } from "react";
import { useEffect, useState } from "react";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { featureFlags } from "@/config/feature-flags";
import { cn } from "@/lib/utils";

const homeHref =
  process.env.NODE_ENV === "development"
    ? "http://localhost:3000"
    : "https://5dlabs.ai";

const navLinks = [
  { name: "Agents", href: "/cto#agents" },
  { name: "Platform", href: "/cto/services" },
  { name: "Bare Metal", href: "/cto#infrastructure" },
];

const socials = [
  {
    name: "GitHub",
    href: "https://github.com/5dlabs",
    icon: (
      <svg className="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 24 24">
        <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12" />
      </svg>
    ),
  },
];

export function Header() {
  const pathname = usePathname();
  const router = useRouter();
  const [currentHash, setCurrentHash] = useState("");
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [isSubdomain] = useState(() =>
    typeof window !== "undefined" && window.location.hostname.startsWith("cto.")
  );

  useEffect(() => {
    const syncHash = () => setCurrentHash(window.location.hash);
    syncHash();
    window.addEventListener("hashchange", syncHash);
    return () => window.removeEventListener("hashchange", syncHash);
  }, [pathname]);

  const resolveHref = (href: string): string => {
    if (!isSubdomain) return href;
    if (href.startsWith("/cto")) {
      if (href.startsWith("/cto#")) {
        return href;
      }
      // Some subdomain deployments are mounted at /cto while others are mounted at /.
      // Preserve /cto paths when already within that namespace; otherwise strip it.
      if (pathname === "/cto" || pathname.startsWith("/cto/")) {
        return href;
      }
      return href.replace(/^\/cto/, "") || "/";
    }
    return href;
  };

  const isNavLinkActive = (href: string) => {
    if (href.startsWith("http")) {
      return false;
    }
    const resolved = resolveHref(href);
    const [targetPath, rawHash] = resolved.split("#");
    const normalizedTargetPath = (targetPath || "/").replace(/\/+$/, "") || "/";
    const normalizedPathname = pathname.replace(/\/+$/, "") || "/";

    if (normalizedPathname !== normalizedTargetPath) {
      return false;
    }
    if (!rawHash) {
      return true;
    }

    return currentHash === `#${rawHash}`;
  };

  const getNavLinkClassName = (href: string) =>
    [
      "px-3 py-1.5 rounded-full text-xs whitespace-nowrap font-medium",
      "transition-all duration-200 ease-out",
      isNavLinkActive(href)
        ? "premium-chip text-foreground shadow-[0_0_16px_rgba(99,102,241,0.2)]"
        : "text-muted-foreground hover:text-foreground hover:bg-white/[0.08]",
    ].join(" ");

  const handleNavClick = (event: MouseEvent<HTMLAnchorElement>, href: string) => {
    const resolved = resolveHref(href);
    const [targetPath, rawHash] = resolved.split("#");
    const hash = rawHash ? `#${rawHash}` : "";
    const normalizedTargetPath = (targetPath || "/").replace(/\/+$/, "") || "/";
    const normalizedPathname = pathname.replace(/\/+$/, "") || "/";

    if (hash) {
      event.preventDefault();

      // Same-page hash navigation can no-op under router transitions.
      // Force a deterministic in-page scroll so clicks always respond.
      if (normalizedPathname === normalizedTargetPath) {
        const targetId = rawHash;
        if (!targetId) {
          return;
        }
        const target = document.getElementById(targetId);
        if (!target) {
          return;
        }

        target.scrollIntoView({ behavior: "smooth", block: "start" });
        window.history.replaceState(null, "", hash);
        setCurrentHash(hash);
        return;
      }

      // Cross-page hash navigation can occasionally miss target anchors.
      // Route explicitly so the destination hash is preserved every time.
      router.push(resolved);
      return;
    }

    // If user taps the page they're already on, scroll to top.
    if (!hash && normalizedPathname === normalizedTargetPath) {
      event.preventDefault();
      window.scrollTo({ top: 0, behavior: "smooth" });
    }
  };

  return (
    <header
      className="fixed left-0 right-0 z-50 flex justify-center px-4"
      style={{ top: "max(1rem, env(safe-area-inset-top, 1rem))" }}
    >
      <nav className="premium-shell flex items-center gap-1.5 px-2.5 py-1.5 rounded-full backdrop-blur-xl">
        {/* Brand switcher */}
        <a
          href={homeHref}
          className="flex items-center justify-center h-8 px-3 rounded-full bg-white/[0.04] text-[11px] font-semibold text-muted-foreground hover:text-foreground hover:bg-white/[0.1] transition-colors"
          aria-label="Back to 5D Labs"
        >
          5D Labs
        </a>

        <div className="w-px h-4 bg-white/[0.14] mx-1" />

        <Link
          href={resolveHref("/cto")}
          className={`flex items-center justify-center h-8 px-3 rounded-full transition-all duration-200 ease-out ${
            pathname === "/cto" || pathname === "/" || pathname === ""
              ? "premium-chip"
              : "bg-white/[0.07] hover:bg-white/[0.12]"
          }`}
          aria-label="CTO Home"
        >
          <span className="text-xs font-bold text-cyan">CTO</span>
        </Link>

        {/* Morgan - prominent CTA (desktop only; mobile uses hamburger) */}
        <Link
          href={resolveHref("/cto/morgan")}
          className="hidden sm:flex shrink-0 items-center justify-center h-8 px-4 rounded-full bg-gradient-to-r from-violet-500 via-indigo-500 to-cyan-500 text-white text-xs font-semibold transition-all shadow-[0_0_24px_rgba(99,102,241,0.4)] hover:brightness-110"
          aria-label="Talk to Morgan"
        >
          <span className="whitespace-nowrap">Talk to Morgan</span>
        </Link>

        {/* Divider (desktop only when Morgan is visible) */}
        <div className="hidden sm:block w-px h-4 bg-white/[0.14] mx-1" />

        {/* Mobile: hamburger opens sidebar */}
        <div className="sm:hidden">
          <button
            type="button"
            onClick={() => setMobileMenuOpen((o) => !o)}
            className="flex h-8 w-8 items-center justify-center rounded-full text-muted-foreground hover:text-foreground hover:bg-white/[0.1]"
            aria-label={mobileMenuOpen ? "Close menu" : "Open menu"}
            aria-expanded={mobileMenuOpen}
          >
            {mobileMenuOpen ? (
              <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            ) : (
              <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
              </svg>
            )}
          </button>
          {/* Sidebar overlay + panel */}
          <div
            className={cn(
              "fixed inset-0 z-40 transition-opacity duration-300",
              mobileMenuOpen ? "opacity-100" : "opacity-0 pointer-events-none"
            )}
            aria-hidden={!mobileMenuOpen}
          >
            <div
              className="absolute inset-0 bg-black/60 backdrop-blur-sm"
              onClick={() => setMobileMenuOpen(false)}
            />
            <aside
              className={cn(
                "fixed left-0 top-0 bottom-0 z-50 w-[min(280px,85vw)] flex flex-col",
                "border-r border-white/[0.08] bg-card/98 backdrop-blur-xl shadow-2xl",
                "transition-transform duration-300 ease-out",
                mobileMenuOpen ? "translate-x-0" : "-translate-x-full"
              )}
            >
              <div className="flex items-center justify-between p-4 border-b border-white/[0.06]">
                <span className="text-sm font-semibold text-muted-foreground">Navigation</span>
                <button
                  type="button"
                  onClick={() => setMobileMenuOpen(false)}
                  className="p-2 rounded-full text-muted-foreground hover:text-foreground hover:bg-white/[0.06]"
                  aria-label="Close menu"
                >
                  <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>
              <nav className="flex flex-col p-3 gap-0.5 overflow-y-auto">
                <Link
                  href={resolveHref("/cto/morgan")}
                  onClick={() => setMobileMenuOpen(false)}
                  className={cn(
                    "flex items-center gap-2 px-4 py-3 rounded-lg text-sm font-semibold transition-colors",
                    "bg-gradient-to-r from-cyan-500/20 to-blue-500/20 text-cyan"
                  )}
                >
                  <span className="w-2 h-2 rounded-full bg-cyan animate-pulse" />
                  Talk to Morgan
                </Link>
                {navLinks.map((link) => (
                  <Link
                    key={link.name}
                    href={resolveHref(link.href)}
                    onClick={(event) => {
                      handleNavClick(event, link.href);
                      setMobileMenuOpen(false);
                    }}
                    className={cn(
                      "block px-4 py-3 rounded-lg text-sm transition-colors",
                      isNavLinkActive(link.href)
                        ? "text-foreground bg-white/[0.08]"
                        : "text-muted-foreground hover:text-foreground hover:bg-white/[0.04]"
                    )}
                  >
                    {link.name}
                  </Link>
                ))}
                {featureFlags.showPricingLink && (
                  <Link
                    href={resolveHref("/cto/pricing")}
                    onClick={() => setMobileMenuOpen(false)}
                    className={cn(
                      "block px-4 py-3 rounded-lg text-sm transition-colors",
                      isNavLinkActive("/cto/pricing")
                        ? "text-foreground bg-white/[0.08]"
                        : "text-muted-foreground hover:text-foreground hover:bg-white/[0.04]"
                    )}
                  >
                    Pricing
                  </Link>
                )}
                <div className="mt-4 pt-4 border-t border-white/[0.06]">
                  <a
                    href={socials[0].href}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="flex items-center gap-2 px-4 py-2 text-sm text-muted-foreground hover:text-foreground"
                  >
                    <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                      <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12" />
                    </svg>
                    GitHub
                  </a>
                </div>
              </nav>
            </aside>
          </div>
        </div>

        {/* Nav links (desktop) */}
        <div className="hidden sm:flex items-center gap-0.5">
          {navLinks.map((link) => (
            <Link
              key={link.name}
              href={resolveHref(link.href)}
              onClick={(event) => handleNavClick(event, link.href)}
              className={getNavLinkClassName(link.href)}
            >
              {link.name}
            </Link>
          ))}
          {featureFlags.showPricingLink && (
            <Link
              href={resolveHref("/cto/pricing")}
              className={getNavLinkClassName("/cto/pricing")}
            >
              Pricing
            </Link>
          )}
        </div>

        {/* Divider */}
        <div className="hidden sm:block w-px h-4 bg-white/[0.14] mx-1" />

        {/* Social icons + 5D Labs link */}
        <div className="flex items-center gap-0.5">
          {socials.map((s) => (
            <a
              key={s.name}
              href={s.href}
              target={s.href.startsWith("http") ? "_blank" : undefined}
              rel={s.href.startsWith("http") ? "noopener noreferrer" : undefined}
              className="flex items-center justify-center w-7 h-7 rounded-full text-muted-foreground/60 hover:text-foreground hover:bg-white/[0.1] transition-all"
              aria-label={s.name}
            >
              {s.icon}
            </a>
          ))}
        </div>

        {/* Start Now button */}
        {featureFlags.showStartNowButton && (
          <>
            <div className="w-px h-4 bg-white/[0.08] mx-1" />
            <a
              href="https://app.5dlabs.ai"
              className="px-3 py-1.5 rounded-full text-xs font-semibold bg-gradient-to-r from-cyan-500 to-blue-500 text-white hover:from-cyan-600 hover:to-blue-600 transition-all"
            >
              Start Now
            </a>
          </>
        )}
      </nav>
    </header>
  );
}

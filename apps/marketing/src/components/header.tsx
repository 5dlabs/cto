"use client";

import type { MouseEvent } from "react";
import { useEffect, useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { featureFlags } from "@/config/feature-flags";
import { cn } from "@/lib/utils";

const homeHref =
  process.env.NODE_ENV === "development"
    ? "http://localhost:3001"
    : "https://5dlabs.ai";

const navLinks = [
  { name: "Stack", href: "/#stack" },
  { name: "Services", href: "/services" },
  { name: "Infrastructure", href: "/#infrastructure" },
  { name: "Platform", href: "/#platform" },
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
  const [currentHash, setCurrentHash] = useState("");
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  useEffect(() => {
    const syncHash = () => setCurrentHash(window.location.hash);
    syncHash();
    window.addEventListener("hashchange", syncHash);
    return () => window.removeEventListener("hashchange", syncHash);
  }, []);

  const isNavLinkActive = (href: string) => {
    if (href.startsWith("http")) {
      return false;
    }
    const [targetPath, rawHash] = href.split("#");
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
      "px-3 py-1.5 rounded-full text-xs whitespace-nowrap",
      "transition-all duration-200 ease-out",
      isNavLinkActive(href)
        ? "bg-white/[0.14] text-foreground shadow-[inset_0_0_0_1px_rgba(255,255,255,0.18)]"
        : "text-muted-foreground hover:text-foreground hover:bg-white/[0.06]",
    ].join(" ");

  const handleNavClick = (event: MouseEvent<HTMLAnchorElement>, href: string) => {
    const [targetPath, rawHash] = href.split("#");
    const hash = rawHash ? `#${rawHash}` : "";
    const normalizedTargetPath = (targetPath || "/").replace(/\/+$/, "") || "/";
    const normalizedPathname = pathname.replace(/\/+$/, "") || "/";

    // Same-page hash navigation can no-op under router transitions.
    // Force a deterministic in-page scroll so clicks always respond.
    if (hash && normalizedPathname === normalizedTargetPath) {
      const targetId = rawHash;
      if (!targetId) {
        return;
      }
      const target = document.getElementById(targetId);
      if (!target) {
        return;
      }

      event.preventDefault();
      target.scrollIntoView({ behavior: "smooth", block: "start" });
      window.history.replaceState(null, "", hash);
      setCurrentHash(hash);
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
      <nav className="flex items-center gap-1 px-2 py-1.5 rounded-full border border-white/[0.06] bg-white/[0.03] backdrop-blur-xl shadow-[0_0_30px_rgba(0,0,0,0.3)]">
        {/* Brand switcher */}
        <a
          href={homeHref}
          className="flex items-center justify-center h-8 px-3 rounded-full bg-white/[0.03] text-[11px] font-semibold text-muted-foreground hover:text-foreground hover:bg-white/[0.08] transition-colors"
          aria-label="Back to 5D Labs"
        >
          5D Labs
        </a>

        <div className="w-px h-4 bg-white/[0.08] mx-1" />

        <Link
          href="/"
          className={`flex items-center justify-center h-8 px-3 rounded-full transition-all duration-200 ease-out ${
            pathname === "/" || pathname === ""
              ? "bg-white/[0.14] shadow-[inset_0_0_0_1px_rgba(255,255,255,0.18)]"
              : "bg-white/[0.06] hover:bg-white/[0.1]"
          }`}
          aria-label="CTO Home"
        >
          <span className="text-xs font-bold text-cyan">CTO</span>
        </Link>

        {/* Morgan - prominent CTA (desktop only; mobile uses hamburger) */}
        <Link
          href="/morgan"
          className="hidden sm:flex items-center justify-center h-8 px-3 rounded-full bg-gradient-to-r from-cyan-500/90 to-blue-500/90 text-white text-xs font-semibold hover:from-cyan-500 hover:to-blue-500 transition-all shadow-[0_0_12px_rgba(34,211,238,0.3)]"
          aria-label="Talk to Morgan"
        >
          Talk to Morgan
        </Link>

        {/* Divider (desktop only when Morgan is visible) */}
        <div className="hidden sm:block w-px h-4 bg-white/[0.08] mx-1" />

        {/* Mobile menu button */}
        <div className="relative sm:hidden">
          <button
            type="button"
            onClick={() => setMobileMenuOpen((o) => !o)}
            className="flex h-8 w-8 items-center justify-center rounded-full text-muted-foreground hover:text-foreground hover:bg-white/[0.06]"
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
          {mobileMenuOpen && (
            <>
              <div
                className="fixed inset-0 z-40"
                aria-hidden
                onClick={() => setMobileMenuOpen(false)}
              />
              <div className="absolute right-0 top-full mt-2 z-50 min-w-[160px] rounded-xl border border-white/[0.08] bg-card/95 py-2 shadow-xl backdrop-blur-xl">
                <Link
                  href="/morgan"
                  onClick={() => setMobileMenuOpen(false)}
                  className="flex items-center gap-2 px-4 py-2.5 text-sm font-semibold bg-gradient-to-r from-cyan-500/20 to-blue-500/20 text-cyan border-b border-white/[0.06] mb-2 -mx-1 -mt-1 rounded-t-xl"
                >
                  <span className="w-2 h-2 rounded-full bg-cyan animate-pulse" />
                  Talk to Morgan
                </Link>
                {navLinks.map((link) => (
                  <Link
                    key={link.name}
                    href={link.href}
                    onClick={(event) => {
                      handleNavClick(event, link.href);
                      setMobileMenuOpen(false);
                    }}
                    className={cn(
                      "block px-4 py-2 text-sm transition-colors",
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
                    href="/pricing"
                    onClick={() => setMobileMenuOpen(false)}
                    className={cn(
                      "block px-4 py-2 text-sm transition-colors",
                      isNavLinkActive("/pricing")
                        ? "text-foreground bg-white/[0.08]"
                        : "text-muted-foreground hover:text-foreground hover:bg-white/[0.04]"
                    )}
                  >
                    Pricing
                  </Link>
                )}
                <Link
                  href="/team"
                  onClick={() => setMobileMenuOpen(false)}
                  className={cn(
                    "block px-4 py-2 text-sm transition-colors",
                    isNavLinkActive("/team")
                      ? "text-foreground bg-white/[0.08]"
                      : "text-muted-foreground hover:text-foreground hover:bg-white/[0.04]"
                  )}
                >
                  Team
                </Link>
              </div>
            </>
          )}
        </div>

        {/* Nav links (desktop) */}
        <div className="hidden sm:flex items-center gap-0.5">
          {navLinks.map((link) => (
            <Link
              key={link.name}
              href={link.href}
              onClick={(event) => handleNavClick(event, link.href)}
              className={getNavLinkClassName(link.href)}
            >
              {link.name}
            </Link>
          ))}
          {featureFlags.showPricingLink && (
            <Link
              href="/pricing"
              className={getNavLinkClassName("/pricing")}
            >
              Pricing
            </Link>
          )}
          <Link href="/team" className={getNavLinkClassName("/team")}>
            Team
          </Link>
        </div>

        {/* Divider */}
        <div className="hidden sm:block w-px h-4 bg-white/[0.08] mx-1" />

        {/* Social icons + 5D Labs link */}
        <div className="flex items-center gap-0.5">
          {socials.map((s) => (
            <a
              key={s.name}
              href={s.href}
              target={s.href.startsWith("http") ? "_blank" : undefined}
              rel={s.href.startsWith("http") ? "noopener noreferrer" : undefined}
              className="flex items-center justify-center w-7 h-7 rounded-full text-muted-foreground/60 hover:text-foreground hover:bg-white/[0.06] transition-all"
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

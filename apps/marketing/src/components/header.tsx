import Link from "next/link";
import { featureFlags } from "@/config/feature-flags";

const navLinks = [
  { name: "Team", href: "/#agents" },
  { name: "Stack", href: "/#stack" },
  { name: "Infrastructure", href: "/#infrastructure" },
  { name: "Platform", href: "/#platform" },
];

const socials = [
  {
    name: "5D Labs",
    href: "https://5dlabs.ai",
    icon: (
      <span className="text-[10px] font-bold">5D</span>
    ),
  },
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
  return (
    <header className="fixed top-4 left-0 right-0 z-50 flex justify-center px-4 fade-in-down">
      <nav className="flex items-center gap-1 px-2 py-1.5 rounded-full border border-white/[0.06] bg-white/[0.03] backdrop-blur-xl shadow-[0_0_30px_rgba(0,0,0,0.3)]">
        {/* Logo */}
        <Link
          href="/"
          className="flex items-center justify-center h-8 px-3 rounded-full bg-white/[0.06] hover:bg-white/[0.1] transition-colors"
          aria-label="CTO Home"
        >
          <span className="text-xs font-bold text-cyan">CTO</span>
        </Link>

        {/* Divider */}
        <div className="w-px h-4 bg-white/[0.08] mx-1" />

        {/* Nav links */}
        <div className="hidden sm:flex items-center gap-0.5">
          {navLinks.map((link) => (
            <Link
              key={link.name}
              href={link.href}
              className="px-3 py-1.5 rounded-full text-xs text-muted-foreground hover:text-foreground hover:bg-white/[0.06] transition-all"
            >
              {link.name}
            </Link>
          ))}
          {featureFlags.showPricingLink && (
            <Link
              href="/pricing"
              className="px-3 py-1.5 rounded-full text-xs text-muted-foreground hover:text-foreground hover:bg-white/[0.06] transition-all"
            >
              Pricing
            </Link>
          )}
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

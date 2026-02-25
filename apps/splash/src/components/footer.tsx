"use client";

import Image from "next/image";

const footerLinks = [
  {
    title: "Ventures",
    links: [
      { name: "CTO Platform", href: "https://cto.5dlabs.ai" },
      { name: "Agentic Trading", href: "/#ventures" },
      { name: "OpenClaw Platform", href: "https://github.com/5dlabs/openclaw-platform" },
      { name: "Sanctuary", href: "/#ventures" },
    ],
  },
  {
    title: "Company",
    links: [
      { name: "Consulting", href: "/consulting" },
      { name: "Founder", href: "/founder" },
      { name: "Team", href: "/team" },
      { name: "Investors", href: "/investors" },
    ],
  },
  {
    title: "Connect",
    links: [
      { name: "GitHub", href: "https://github.com/5dlabs" },
      { name: "Discord", href: "https://discord.gg/r334tFP87Y" },
      { name: "X / Twitter", href: "https://x.com/5dlabs" },
      { name: "YouTube", href: "https://youtube.com/@5dlabs" },
    ],
  },
];

export function Footer() {
  return (
    <footer className="border-t border-border/30 bg-background/50 backdrop-blur-sm">
      <div className="max-w-6xl mx-auto px-6 py-16">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-8">
          {/* Brand column */}
          <div className="col-span-2 md:col-span-1">
            <Image
              src="/5dlabs-logo-header-v2.png"
              alt="5D Labs — OpenClaw-first AI startup studio logo"
              width={120}
              height={120}
              className="opacity-90 mb-4"
            />
            <p className="text-sm text-muted-foreground max-w-xs">
              OpenClaw-first, crypto-first, AI-first startup studio.
            </p>
          </div>

          {/* Link columns */}
          {footerLinks.map((group) => (
            <div key={group.title}>
              <h3 className="text-sm font-semibold text-foreground mb-4">
                {group.title}
              </h3>
              <ul className="space-y-3">
                {group.links.map((link) => (
                  <li key={link.name}>
                    <a
                      href={link.href}
                      target={link.href.startsWith("http") ? "_blank" : undefined}
                      rel={link.href.startsWith("http") ? "noopener noreferrer" : undefined}
                      className="text-sm text-muted-foreground hover:text-foreground transition-colors"
                      data-umami-event={`footer-${group.title.toLowerCase()}-${link.name.toLowerCase().replace(/[\s\/]/g, "-")}`}
                    >
                      {link.name}
                    </a>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        {/* Bottom bar */}
        <div className="mt-12 pt-8 border-t border-border/30 flex flex-col sm:flex-row items-center justify-between gap-4">
          <p className="text-sm text-muted-foreground">
            &copy; {new Date().getFullYear()} 5D Labs. Building the future in parallel.
          </p>
          <a
            href="https://cal.com/jonathon-fritz-2uhdqe/discovery"
            target="_blank"
            rel="noopener noreferrer"
            className="text-sm text-cyan hover:text-cyan/80 transition-colors"
            data-umami-event="footer-schedule-call"
          >
            Schedule a Call
          </a>
        </div>
      </div>
    </footer>
  );
}

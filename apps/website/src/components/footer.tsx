"use client";

import Image from "next/image";
import Link from "next/link";

const teamHref =
  process.env.NODE_ENV === "development"
    ? "http://localhost:3000/cto/team"
    : "/cto/team";

const footerLinks = [
  {
    title: "Products",
    links: [
      { name: "CTO Platform", href: "/cto" },
      { name: "Trading Engine", href: "/trading" },
    ],
  },
  {
    title: "Company",
    links: [
      { name: "Opportunities", href: "/opportunities" },
      { name: "Founder", href: "/founder" },
      { name: "Team", href: teamHref },
      { name: "Investors", href: "/investors" },
    ],
  },
  {
    title: "Connect",
    links: [
      { name: "GitHub", href: "https://github.com/5dlabs" },
      { name: "OpenClaw Platform (Coming Soon)", href: "/#openclaw" },
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
            <Link href="/" aria-label="5D Labs home" className="inline-flex">
              <Image
                src="/5dlabs-logo-3d.jpg"
                alt="5D Labs"
                width={220}
                height={56}
                className="h-12 w-auto opacity-95 mb-4"
              />
            </Link>
            <p className="text-sm text-muted-foreground max-w-xs">
              AI-native venture studio powered by an internal operating stack.
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
        </div>
      </div>
    </footer>
  );
}

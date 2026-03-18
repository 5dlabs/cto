import type { Metadata } from "next";
import Script from "next/script";
import { Space_Grotesk, JetBrains_Mono } from "next/font/google";
import { GridPulse } from "@/components/grid-pulse";
import { MagneticFilingsBackgroundSwitch } from "@/components/magnetic-filings-background";
import "./globals.css";

const spaceGrotesk = Space_Grotesk({
  variable: "--font-geist-sans",
  subsets: ["latin"],
  display: "swap",
});

const jetbrainsMono = JetBrains_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
  display: "swap",
});

export const metadata: Metadata = {
  title: "5D Labs | AI-Native Venture Studio",
  description:
    "5D Labs is an AI-native venture studio powered by CTO, our build engine, an internal trading engine, and OpenClaw, our orchestration layer. We use this stack to discover, finance, and ship new ventures.",
  keywords: [
    "5D Labs",
    "OpenClaw",
    "AI venture studio",
    "venture studio",
    "crypto",
    "Solana",
    "Base",
    "Near",
    "agentic trading",
    "CTO platform",
    "multi-agent",
    "AI agents",
    "AI engineering",
    "autonomous agents",
    "DeFi",
    "web3",
    "blockchain",
    "machine learning",
    "artificial intelligence",
    "AI consulting",
    "blockchain consulting",
    "Solana development",
    "Kubernetes consulting",
    "DevOps consulting",
    "infrastructure consulting",
    "Rust development",
    "platform engineering",
  ],
  authors: [{ name: "5D Labs", url: "https://5dlabs.ai" }],
  creator: "5D Labs",
  publisher: "5D Labs",
  manifest: "/site.webmanifest",
  metadataBase: new URL("https://5dlabs.ai"),
  alternates: {
    canonical: "/",
  },
  openGraph: {
    title: "5D Labs | AI-Native Venture Studio",
    description:
      "AI-native venture studio powered by CTO, an internal trading engine, and OpenClaw.",
    url: "https://5dlabs.ai",
    siteName: "5D Labs",
    locale: "en_US",
    type: "website",
    images: [
      {
        url: "/og-image-v2.jpg",
        width: 1200,
        height: 630,
        alt: "5D Labs - AI-Native Venture Studio",
        type: "image/jpeg",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "5D Labs | AI-Native Venture Studio",
    description:
      "AI-native venture studio powered by CTO, an internal trading engine, and OpenClaw.",
    images: ["/og-image-v2.jpg"],
    creator: "@5dlabs",
  },
  robots: {
    index: true,
    follow: true,
    googleBot: {
      index: true,
      follow: true,
      "max-video-preview": -1,
      "max-image-preview": "large",
      "max-snippet": -1,
    },
  },
  other: {
    "ai:description":
      "5D Labs is an AI-native venture studio powered by CTO, our commercial build engine, an in-house trading engine, and OpenClaw, the orchestration layer beneath both. The stack exists to discover, finance, and ship new ventures.",
    "ai:capabilities":
      "venture-studio, multi-agent-orchestration, internal-trading-infrastructure, AI-engineering, crypto, DeFi, open-source, kubernetes, gitops, technical-co-founder, fractional-cto, infrastructure-lead",
    "ai:ventures": "CTO, Internal Trading Engine, OpenClaw Platform, Venture Pipeline",
    "ai:opportunities": "https://5dlabs.ai/opportunities/",
  },
};

const jsonLd = {
  "@context": "https://schema.org",
  "@graph": [
    {
      "@type": "Organization",
      "@id": "https://5dlabs.ai/#organization",
      name: "5D Labs",
      url: "https://5dlabs.ai",
      logo: {
        "@type": "ImageObject",
        url: "https://5dlabs.ai/og-image-v2.jpg",
        width: 1200,
        height: 630,
      },
      description:
        "AI-native venture studio powered by CTO, an internal trading engine, and OpenClaw.",
      foundingDate: "2024",
      founder: {
        "@type": "Person",
        name: "Jonathon Fritz",
        jobTitle: "Founder & CEO",
        url: "https://5dlabs.ai/founder",
      },
      sameAs: [
        "https://github.com/5dlabs",
        "https://x.com/5dlabs",
        "https://discord.gg/r334tFP87Y",
        "https://youtube.com/@5dlabs",
      ],
      knowsAbout: [
        "AI Agent Orchestration",
        "Multi-Agent Systems",
        "Blockchain Development",
        "Solana",
        "On-Chain Market Infrastructure",
        "Kubernetes",
        "DevOps",
        "OpenClaw",
      ],
    },
    {
      "@type": "WebSite",
      "@id": "https://5dlabs.ai/#website",
      url: "https://5dlabs.ai",
      name: "5D Labs",
      description:
        "AI-native venture studio powered by CTO, an internal trading engine, and OpenClaw.",
      publisher: {
        "@id": "https://5dlabs.ai/#organization",
      },
    },
    {
      "@type": "SoftwareApplication",
      name: "CTO",
      applicationCategory: "DeveloperApplication",
      description:
        "Multi-agent AI engineering platform with 13 specialized agents for autonomous software development on self-healing bare metal infrastructure.",
      url: "https://cto.5dlabs.ai",
      operatingSystem: "Kubernetes",
      author: {
        "@id": "https://5dlabs.ai/#organization",
      },
    },
    {
      "@type": "SoftwareApplication",
      name: "OpenClaw Platform",
      applicationCategory: "DeveloperApplication",
      description:
        "Open-source Kubernetes-native platform for deploying and managing AI agent fleets. One-command TUI installer supporting KinD and EKS with GitOps, NATS messaging, and observability.",
      url: "https://github.com/5dlabs/openclaw-platform",
      operatingSystem: "Kubernetes",
      isAccessibleForFree: true,
      license: "https://opensource.org/licenses/AGPL-3.0",
      author: {
        "@id": "https://5dlabs.ai/#organization",
      },
    },
    {
      "@type": "ProfessionalService",
      "@id": "https://5dlabs.ai/consulting/#service",
      name: "5D Labs Consulting",
      description:
        "Expert consulting for AI agent systems, blockchain and Solana development, Kubernetes infrastructure, and DevOps.",
      provider: {
        "@id": "https://5dlabs.ai/#organization",
      },
      serviceType: [
        "AI Agent Consulting",
        "Blockchain Consulting",
        "Solana Development",
        "Kubernetes Consulting",
        "DevOps Consulting",
        "Trading Infrastructure Architecture",
        "Platform Engineering",
      ],
      areaServed: {
        "@type": "Country",
        name: "US",
      },
      url: "https://5dlabs.ai/consulting/",
      priceRange: "$225-$3500/mo",
    },
    {
      "@type": "BreadcrumbList",
      itemListElement: [
        {
          "@type": "ListItem",
          position: 1,
          name: "Home",
          item: "https://5dlabs.ai",
        },
        {
          "@type": "ListItem",
          position: 2,
          name: "Consulting",
          item: "https://5dlabs.ai/consulting",
        },
        {
          "@type": "ListItem",
          position: 3,
          name: "Investors",
          item: "https://5dlabs.ai/investors",
        },
        {
          "@type": "ListItem",
          position: 4,
          name: "Founder",
          item: "https://5dlabs.ai/founder",
        },
        {
          "@type": "ListItem",
          position: 5,
          name: "Team",
          item: "https://5dlabs.ai/team",
        },
      ],
    },
    {
      "@type": "FAQPage",
      mainEntity: [
        {
          "@type": "Question",
          name: "What is 5D Labs?",
          acceptedAnswer: {
            "@type": "Answer",
            text: "5D Labs is an AI-native venture studio powered by an internal operating stack. CTO is the commercial build engine, the internal trading engine helps finance experimentation and provide market intelligence, and OpenClaw is the orchestration layer beneath both. The purpose of the stack is to help 5D Labs discover, validate, and launch new ventures.",
          },
        },
        {
          "@type": "Question",
          name: "What is OpenClaw?",
          acceptedAnswer: {
            "@type": "Answer",
            text: "OpenClaw is an open-source agent orchestration platform that coordinates AI agents across multiple CLIs (Claude Code, Cursor, Codex, Factory, Gemini), self-healing bare metal Kubernetes infrastructure, and MCP server ecosystems. It powers all 5D Labs ventures.",
          },
        },
        {
          "@type": "Question",
          name: "Does 5D Labs offer consulting services?",
          acceptedAnswer: {
            "@type": "Answer",
            text: "Yes. 5D Labs offers expert consulting in AI agent systems, blockchain development (Solana, Base, Polygon, Near, Sui), Kubernetes infrastructure, and DevOps. Rates start at $225/hr with monthly retainers available at $3,500/mo. Book a free discovery call at https://cal.com/jonathon-fritz-2uhdqe/discovery.",
          },
        },
        {
          "@type": "Question",
          name: "What blockchains does 5D Labs support?",
          acceptedAnswer: {
            "@type": "Answer",
            text: "5D Labs runs its internal trading systems across Solana (low-latency execution), Base (Ethereum composability), Polygon (Ethereum scaling and institutional adoption), Near (AI-native smart contracts), and Sui (Move, object-centric DeFi).",
          },
        },
      ],
    },
  ],
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark">
      <head>
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd) }}
        />
        <link rel="icon" href="/favicon.ico" sizes="32x32" />
        <link rel="icon" href="/icon.svg" type="image/svg+xml" />
        <link rel="apple-touch-icon" href="/apple-touch-icon.png" />
        <meta name="theme-color" content="#06b6d4" />
        <link rel="llms" href="/llms.txt" />
        <link rel="llms-full" href="/llms-full.txt" />
        <meta name="llms-txt" content="/llms.txt" />
        {/* Keep content declarative: avoid scripts that hide/show nodes post-render. */}
      </head>
      <body
        className={`${spaceGrotesk.variable} ${jetbrainsMono.variable} antialiased`}
      >
        <Script
          defer
          src="https://cloud.umami.is/script.js"
          data-website-id="da83813e-ccdc-4b94-944a-66f476db85ef"
          strategy="afterInteractive"
        />
        <MagneticFilingsBackgroundSwitch />
        <GridPulse />
        {children}
      </body>
    </html>
  );
}

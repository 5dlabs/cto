import type { Metadata } from "next";
import { Space_Grotesk, JetBrains_Mono } from "next/font/google";
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
  title: "5D Labs | OpenClaw-First AI Startup Studio",
  description:
    "5D Labs is an OpenClaw-first, crypto-first, AI-first startup studio. We build and validate multiple ventures in parallel — from agentic trading on Solana, Base, and Near to CTO, our multi-agent AI engineering platform.",
  keywords: [
    "5D Labs",
    "OpenClaw",
    "AI startup studio",
    "startup studio",
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
  metadataBase: new URL("https://5dlabs.ai"),
  alternates: {
    canonical: "https://5dlabs.ai",
  },
  openGraph: {
    title: "5D Labs | OpenClaw-First AI Startup Studio",
    description:
      "Build and validate multiple ventures in parallel. OpenClaw-first, crypto-first, AI-first.",
    url: "https://5dlabs.ai",
    siteName: "5D Labs",
    locale: "en_US",
    type: "website",
    images: [
      {
        url: "/og-image-v2.jpg",
        width: 1200,
        height: 630,
        alt: "5D Labs - OpenClaw-First AI Startup Studio",
        type: "image/jpeg",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "5D Labs | OpenClaw-First AI Startup Studio",
    description:
      "Build and validate multiple ventures in parallel. OpenClaw-first, crypto-first, AI-first.",
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
      "5D Labs is an OpenClaw-first, crypto-first, AI-first startup studio that builds and validates multiple ventures in parallel. Current ventures include CTO (multi-agent AI engineering platform), agentic trading on Solana, Base, and Near, OpenClaw Platform (open-source Kubernetes-native agent orchestration), and Sanctuary (AI-powered smart home orchestration).",
    "ai:capabilities":
      "startup-studio, multi-agent-orchestration, agentic-trading, AI-engineering, crypto, DeFi, smart-home, IoT, open-source, kubernetes, gitops",
    "ai:ventures": "CTO, Agentic Trading, OpenClaw Platform, Sanctuary",
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
        "OpenClaw-first, crypto-first, AI-first startup studio building and validating multiple ventures in parallel.",
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
        "High-Frequency Trading",
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
        "OpenClaw-first, crypto-first, AI-first startup studio",
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
        "HFT Trading Systems",
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
            text: "5D Labs is an OpenClaw-first, crypto-first, AI-first startup studio that builds and validates multiple ventures in parallel using autonomous AI agents. Current ventures include CTO (multi-agent engineering platform), Agentic Trading (HFT on Solana, Base, Near), OpenClaw Platform (open-source agent orchestration), and Sanctuary (AI smart home).",
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
            text: "Yes. 5D Labs offers expert consulting in AI agent systems, blockchain development (Solana, Base, Near), Kubernetes infrastructure, and DevOps. Rates start at $225/hr with monthly retainers available at $3,500/mo. Book a free discovery call at https://cal.com/jonathon-fritz-2uhdqe/discovery.",
          },
        },
        {
          "@type": "Question",
          name: "What blockchains does 5D Labs support?",
          acceptedAnswer: {
            "@type": "Answer",
            text: "5D Labs operates agentic trading systems across Solana (for raw speed and HFT), Base (for Ethereum-grade composability), and Near (for AI-native smart contracts).",
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
        <link rel="manifest" href="/site.webmanifest" />
        <meta name="theme-color" content="#06b6d4" />
        <link rel="llms" href="/llms.txt" />
        <link rel="llms-full" href="/llms-full.txt" />
        <meta name="llms-txt" content="/llms.txt" />
      </head>
      <body
        className={`${spaceGrotesk.variable} ${jetbrainsMono.variable} antialiased`}
      >
        {children}
      </body>
    </html>
  );
}

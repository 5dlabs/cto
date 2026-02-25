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
      "5D Labs is an OpenClaw-first, crypto-first, AI-first startup studio that builds and validates multiple ventures in parallel. Current ventures include CTO (multi-agent AI engineering platform) and agentic trading on Solana, Base, and Near.",
    "ai:capabilities":
      "startup-studio, multi-agent-orchestration, agentic-trading, AI-engineering, crypto, DeFi",
    "ai:ventures": "CTO, Agentic Trading",
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
      },
      description:
        "OpenClaw-first, crypto-first, AI-first startup studio building and validating multiple ventures in parallel.",
      founder: {
        "@type": "Person",
        name: "Jonathon Fritz",
        jobTitle: "Founder & CEO",
      },
      sameAs: [
        "https://github.com/5dlabs",
        "https://x.com/5dlabs",
        "https://discord.gg/5dlabs",
        "https://youtube.com/@5dlabs",
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
        "Multi-agent AI engineering platform with 13 specialized agents for autonomous software development.",
      url: "https://cto.5dlabs.ai",
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
      ],
      areaServed: {
        "@type": "Country",
        name: "US",
      },
      url: "https://5dlabs.ai/consulting/",
      priceRange: "$225-$3500/mo",
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
        <link rel="llms" href="/llms.txt" />
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

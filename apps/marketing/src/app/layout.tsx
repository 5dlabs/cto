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
  title: "CTO by 5D Labs | AI Engineering Collective",
  description:
    "Thirteen AI specialists from the Fifth Dimension. Multi-agent orchestration on self-healing bare metal. 60-80% less than cloud. Bring your own CLI and keys.",
  keywords: [
    "AI engineering team",
    "AI agents",
    "multi-agent orchestration",
    "bare metal infrastructure",
    "Claude Code",
    "Cursor",
    "Factory",
    "Codex",
    "AI coding assistant",
    "software development automation",
    "GitOps",
    "self-healing infrastructure",
    "BYOK",
    "bring your own keys",
    "5D Labs",
    "CTO platform",
  ],
  authors: [{ name: "5D Labs", url: "https://github.com/5dlabs" }],
  creator: "5D Labs",
  publisher: "5D Labs",
  metadataBase: new URL("https://cto.5dlabs.ai"),
  alternates: {
    canonical: "https://cto.5dlabs.ai",
  },
  openGraph: {
    title: "CTO by 5D Labs | AI Engineering Collective",
    description:
      "Thirteen AI specialists. Multi-agent orchestration on self-healing bare metal. 60-80% less than cloud.",
    url: "https://cto.5dlabs.ai",
    siteName: "CTO by 5D Labs",
    locale: "en_US",
    type: "website",
    images: [
      {
        url: "/og-image.png",
        width: 1200,
        height: 630,
        alt: "CTO by 5D Labs - AI Engineering Collective",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "CTO by 5D Labs | AI Engineering Collective",
    description:
      "Thirteen AI specialists. Multi-agent orchestration on self-healing bare metal. 60-80% less than cloud.",
    images: ["/twitter-image.png"],
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
  // For AI agents/crawlers
  other: {
    "ai:description":
      "CTO is a multi-agent AI engineering platform with 13 specialized agents (Rex, Blaze, Nova, Grizz, etc.) that deploy on bare metal infrastructure. Supports Claude Code, Cursor, Factory, Codex CLIs with BYOK model.",
    "ai:capabilities":
      "code-generation, code-review, security-analysis, testing, infrastructure-setup, project-management",
    "ai:agents":
      "Morgan, Rex, Grizz, Nova, Blaze, Tap, Spark, Vex, Cleo, Cipher, Tess, Atlas, Bolt",
  },
};

// JSON-LD Structured Data
const jsonLd = {
  "@context": "https://schema.org",
  "@graph": [
    {
      "@type": "Organization",
      "@id": "https://cto.5dlabs.ai/#organization",
      name: "5D Labs",
      url: "https://cto.5dlabs.ai",
      logo: {
        "@type": "ImageObject",
        url: "https://cto.5dlabs.ai/5dlabs-logo-header-v2.png",
      },
      sameAs: ["https://github.com/5dlabs"],
    },
    {
      "@type": "WebSite",
      "@id": "https://cto.5dlabs.ai/#website",
      url: "https://cto.5dlabs.ai",
      name: "CTO by 5D Labs",
      description:
        "AI Engineering Collective - Thirteen AI specialists on self-healing bare metal infrastructure",
      publisher: {
        "@id": "https://cto.5dlabs.ai/#organization",
      },
    },
    {
      "@type": "SoftwareApplication",
      "@id": "https://cto.5dlabs.ai/#software",
      name: "CTO",
      applicationCategory: "DeveloperApplication",
      operatingSystem: "Cloud, On-Premises",
      description:
        "Multi-agent AI engineering platform with 13 specialized agents for software development, deployment, and maintenance on bare metal infrastructure.",
      offers: {
        "@type": "Offer",
        price: "0",
        priceCurrency: "USD",
        description: "Join waitlist for early access",
      },
      featureList: [
        "Multi-agent orchestration",
        "CLI agnostic (Claude Code, Cursor, Factory, Codex)",
        "Bare metal infrastructure",
        "Self-healing deployments",
        "BYOK (Bring Your Own Keys)",
        "GitHub-driven deployments",
        "Linear integration",
        "MCP server ecosystem",
      ],
      author: {
        "@id": "https://cto.5dlabs.ai/#organization",
      },
    },
    {
      "@type": "FAQPage",
      "@id": "https://cto.5dlabs.ai/#faq",
      mainEntity: [
        {
          "@type": "Question",
          name: "What is CTO?",
          acceptedAnswer: {
            "@type": "Answer",
            text: "CTO is an AI-powered engineering collective with 13 specialized AI agents that work together to build, ship, and maintain software on self-healing bare metal infrastructure.",
          },
        },
        {
          "@type": "Question",
          name: "What CLIs does CTO support?",
          acceptedAnswer: {
            "@type": "Answer",
            text: "CTO is CLI agnostic and supports Claude Code, Cursor, Factory, Codex, OpenCode, Gemini CLI, Ollama, vLLM, and more. Bring your own keys and preferred tools.",
          },
        },
        {
          "@type": "Question",
          name: "How much does CTO cost compared to cloud?",
          acceptedAnswer: {
            "@type": "Answer",
            text: "CTO deploys on bare metal infrastructure, typically resulting in 60-80% cost savings compared to traditional cloud providers like AWS, GCP, or Azure.",
          },
        },
        {
          "@type": "Question",
          name: "What AI agents are included?",
          acceptedAnswer: {
            "@type": "Answer",
            text: "CTO includes 13 specialized agents: Morgan (project management), Rex (Rust), Grizz (Go), Nova (Node.js), Blaze (React), Tap (Mobile), Spark (Desktop), Vex (XR), Cleo (code review), Cipher (security), Tess (testing), Atlas (integration), and Bolt (infrastructure).",
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
        {/* JSON-LD Structured Data */}
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd) }}
        />
        {/* Additional AI/Agent hints */}
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

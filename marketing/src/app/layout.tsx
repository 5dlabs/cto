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
  title: "CTO Platform | AI Agents That Ship Code",
  description:
    "Meet your new engineering team. AI agents that understand your codebase, execute complex tasks, and ship production-ready code. Zero infrastructure required.",
  keywords: [
    "AI coding",
    "AI agents",
    "code generation",
    "software development",
    "automation",
    "DevOps",
    "engineering platform",
  ],
  authors: [{ name: "5D Labs" }],
  openGraph: {
    title: "CTO Platform | AI Agents That Ship Code",
    description:
      "Meet your new engineering team. AI agents that understand your codebase and ship production-ready code.",
    url: "https://cto.5dlabs.ai",
    siteName: "CTO Platform",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "CTO Platform | AI Agents That Ship Code",
    description:
      "Meet your new engineering team. AI agents that understand your codebase and ship production-ready code.",
  },
  robots: {
    index: true,
    follow: true,
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark">
      <body
        className={`${spaceGrotesk.variable} ${jetbrainsMono.variable} antialiased`}
      >
        {children}
      </body>
    </html>
  );
}

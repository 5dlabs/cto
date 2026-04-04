import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Pitch Deck (staging) | 5D Labs — AI-Native Venture Studio",
  description:
    "5D Labs pitch deck (staging). CTO turns a product spec into production-ready software with 22 AI agents on owned infrastructure.",
  alternates: {
    canonical: "/investors/staging/",
  },
  openGraph: {
    title: "5D Labs — Pitch Deck (staging)",
    description:
      "Spec in, software out. 22 AI agents. Research-backed infra economics. Pre-seed $750K.",
  },
  robots: {
    index: false,
    follow: false,
    googleBot: {
      index: false,
      follow: false,
    },
  },
};

export default function InvestorsStagingLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return children;
}

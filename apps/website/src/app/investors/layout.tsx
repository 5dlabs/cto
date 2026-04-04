import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Pitch Deck | 5D Labs — AI-Native Venture Studio",
  description:
    "5D Labs pitch deck. CTO turns a product spec into production-ready software with 22 AI agents on owned infrastructure — economics benchmarked to public case studies (Stanford AI Index, Flexera, 37signals).",
  robots: {
    index: false,
    follow: false,
  },
  alternates: {
    canonical: "/investors/",
  },
  openGraph: {
    title: "5D Labs — Pitch Deck",
    description:
      "Spec in, software out. 22 AI agents. Research-backed infra economics. Pre-seed $750K.",
  },
};

export default function InvestorsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return children;
}

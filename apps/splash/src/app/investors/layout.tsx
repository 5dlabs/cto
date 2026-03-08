import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Investor Relations | 5D Labs — AI-Native Venture Studio",
  description:
    "5D Labs is building an AI-native venture studio powered by CTO, an internal trading engine, and OpenClaw orchestration. See the thesis, operating model, and investment narrative.",
  alternates: {
    canonical: "/investors/",
  },
  openGraph: {
    title: "Investor Relations | 5D Labs",
    description:
      "Invest in an AI-native venture studio powered by CTO, an internal trading engine, and OpenClaw.",
  },
};

export default function InvestorsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return children;
}

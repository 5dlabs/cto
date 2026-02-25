import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Investor Relations | 5D Labs — AI-Native Startup Studio",
  description:
    "5D Labs is raising its first round. 5+ active ventures powered by 13 AI agents, 60-80% infrastructure cost savings, and OpenClaw orchestration. See our thesis, portfolio, and key metrics.",
  openGraph: {
    title: "Investor Relations | 5D Labs",
    description:
      "Invest in the future of AI-native startups. 5+ ventures in flight, 13 autonomous agents, open-source infrastructure.",
  },
};

export default function InvestorsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return children;
}

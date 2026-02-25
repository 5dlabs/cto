import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Jonathon Fritz — Founder & CEO | 5D Labs",
  description:
    "Meet Jonathon Fritz, founder of 5D Labs. 15+ years building production systems across AI, blockchain, Kubernetes, and trading. Building the future of AI-native startups with OpenClaw.",
  alternates: {
    canonical: "/founder/",
  },
  openGraph: {
    title: "Jonathon Fritz — Founder & CEO | 5D Labs",
    description:
      "15+ years shipping production systems across AI, blockchain, and infrastructure. Now building 5D Labs — an AI-native startup studio.",
  },
};

export default function FounderLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return children;
}

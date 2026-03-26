import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "AI & Infrastructure Consulting | 5D Labs",
  description:
    "Expert consulting for AI agent systems, blockchain and Solana development, Kubernetes infrastructure, and DevOps. Rates from $225/hr with monthly retainers available.",
  alternates: {
    canonical: "/consulting/",
  },
  openGraph: {
    title: "AI & Infrastructure Consulting | 5D Labs",
    description:
      "Expert consulting in AI agents, blockchain, Kubernetes, and DevOps from the team behind the OpenClaw platform.",
  },
};

export default function ConsultingLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return children;
}

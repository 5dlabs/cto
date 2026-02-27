import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Work With Me | Jonathon Fritz — Technical Co-Founder & Infrastructure Lead",
  description:
    "Open to technical co-founder, fractional CTO, and senior infrastructure roles. 20+ years shipping production systems. 10,600+ GitHub contributions. Builder of multi-agent AI platforms on bare-metal Kubernetes.",
  alternates: {
    canonical: "/opportunities/",
  },
  openGraph: {
    title: "Work With Me | Jonathon Fritz",
    description:
      "Technical co-founder, fractional CTO, or infrastructure lead. 20+ years, 10,600+ contributions, ships AI agent platforms on bare-metal Kubernetes.",
  },
};

export default function OpportunitiesLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return children;
}

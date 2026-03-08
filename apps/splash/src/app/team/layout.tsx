import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "AI Agent Team | 5D Labs — Specialized Autonomous Agents",
  description:
    "Meet the AI agents powering 5D Labs: Morgan (PM), Rex (Rust), Grizz (Go), Blaze (React), Cipher (Security), Tess (Testing), and more. Each agent has a unique identity, skills, and personality.",
  alternates: {
    canonical: "/team/",
  },
  openGraph: {
    title: "AI Agent Team | 5D Labs",
    description:
      "Specialized AI agents — from program management to Rust engineering to security auditing — working autonomously across all 5D Labs ventures.",
  },
};

export default function TeamLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return children;
}

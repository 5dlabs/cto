import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "AI Agent Team | 5D Labs — 13 Specialized Autonomous Agents",
  description:
    "Meet the 13 AI agents powering 5D Labs: Morgan (PM), Rex (Rust), Grizz (Go), Blaze (React), Cipher (Security), Tess (Testing), and more. Each agent has a unique identity, skills, and personality.",
  openGraph: {
    title: "AI Agent Team | 5D Labs",
    description:
      "13 specialized AI agents — from program management to Rust engineering to security auditing — working autonomously across all 5D Labs ventures.",
  },
};

export default function TeamLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return children;
}

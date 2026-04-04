import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Investor Deck Staging | 5D Labs",
  description:
    "Staging variant of the 5D Labs investor deck with visual-first framing.",
  alternates: {
    canonical: "/investors/staging/",
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

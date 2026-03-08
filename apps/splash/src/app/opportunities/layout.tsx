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
  other: {
    "ai:description":
      "Jonathon Fritz is open to technical co-founder (equity), fractional CTO (negotiable), and full-time remote infrastructure roles: VP/Director of Infrastructure, Head of Platform Engineering, AI Systems Lead, Principal Engineer. 20+ years, 10,600+ GitHub contributions, 1B+ requests at Pocket Network. Case studies and engagement types on this page.",
    "ai:capabilities":
      "technical-co-founder, fractional-cto, infrastructure-lead, vp-infrastructure, head-of-platform-engineering, ai-systems-lead, principal-engineer, remote",
    "ai:open_to":
      "technical co-founder (equity), fractional CTO (negotiable), full-time VP/Director of Infrastructure, Head of Platform Engineering, Infrastructure Engineering Manager, AI Systems/Agent Platform Lead, Principal Engineer Infrastructure",
  },
};

const opportunitiesJsonLd = {
  "@context": "https://schema.org",
  "@type": "Person",
  name: "Jonathon Fritz",
  jobTitle: [
    "Technical Co-Founder",
    "Fractional CTO",
    "VP/Director of Infrastructure",
    "Head of Platform Engineering",
    "AI Systems Lead",
    "Principal Engineer",
  ],
  description:
    "Open to technical co-founder (equity), fractional CTO (negotiable), and full-time remote infrastructure roles. 20+ years, 10,600+ GitHub contributions, 1B+ requests at Pocket Network.",
  url: "https://5dlabs.ai/opportunities/",
  sameAs: [
    "https://www.linkedin.com/in/jonathonfritz",
    "https://github.com/kaseonedge",
    "https://resume.jonathonfritz.com",
  ],
  contactPoint: {
    "@type": "ContactPoint",
    email: "j@jonathonfritz.com",
    contactType: "hiring",
    areaServed: "Worldwide",
  },
  worksFor: { "@type": "Organization", name: "5D Labs", url: "https://5dlabs.ai" },
};

export default function OpportunitiesLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <>
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(opportunitiesJsonLd) }}
      />
      {children}
    </>
  );
}

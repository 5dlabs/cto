import type { Metadata } from "next";
import { LegalPage } from "@/components/legal-page";
import { loadLegalMarkdown } from "@/lib/load-legal-doc";

export const metadata: Metadata = {
  title: "Terms of Service | 5D Labs",
  description: "Terms of service for the 5D Labs investor pitch deck website.",
};

export default function TermsOfServicePage() {
  const markdown = loadLegalMarkdown("terms-of-service.md");
  return <LegalPage markdown={markdown} />;
}

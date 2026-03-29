import type { Metadata } from "next";
import { LegalPage } from "@/components/legal-page";
import { loadLegalMarkdown } from "@/lib/load-legal-doc";

export const metadata: Metadata = {
  title: "Privacy Policy | 5D Labs",
  description:
    "Privacy policy for the 5D Labs investor pitch deck website and optional Google Drive export.",
};

export default function PrivacyPolicyPage() {
  const markdown = loadLegalMarkdown("privacy-policy.md");
  return <LegalPage markdown={markdown} />;
}

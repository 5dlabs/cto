"use client";

import Image from "next/image";
import { motion } from "framer-motion";
import { Header } from "@/components/header";

const homeHref =
  process.env.NODE_ENV === "development"
    ? "http://localhost:3001"
    : "https://5dlabs.ai";

const tiers = [
  {
    name: "Free",
    price: "$0",
    period: "forever",
    description: "Perfect for trying out the platform",
    highlight: false,
    badge: null,
    infrastructure: "Fully Managed",
    features: [
      { label: "CodeRuns included", value: "50/month" },
      { label: "Overage rate", value: "$3.00/run" },
      { label: "Users", value: "1" },
      { label: "Repositories", value: "5" },
      { label: "AI API keys", value: "BYOK only" },
      { label: "Support", value: "Community (Discord)" },
    ],
    cta: "Start Free",
    ctaHref: "https://app.5dlabs.ai",
    color: "cyan",
  },
  {
    name: "Team",
    price: "$199",
    period: "/month",
    description: "For small teams shipping faster",
    highlight: false,
    badge: null,
    infrastructure: "Fully Managed",
    features: [
      { label: "CodeRuns included", value: "200/month" },
      { label: "Overage rate", value: "$1.50/run" },
      { label: "Users", value: "10" },
      { label: "Repositories", value: "Unlimited" },
      { label: "AI API keys", value: "BYOK or Managed (+15%)" },
      { label: "Support", value: "Email (48h)" },
      { label: "Features", value: "Team management, SSO" },
    ],
    cta: "Start Trial",
    ctaHref: "https://app.5dlabs.ai",
    color: "blue",
  },
  {
    name: "Growth",
    price: "$499",
    period: "/month",
    description: "For scaling engineering teams",
    highlight: true,
    badge: "Most Popular",
    infrastructure: "Fully Managed",
    features: [
      { label: "CodeRuns included", value: "1,000/month" },
      { label: "Overage rate", value: "$0.75/run" },
      { label: "Users", value: "Unlimited" },
      { label: "Repositories", value: "Unlimited" },
      { label: "AI API keys", value: "BYOK or Managed (+10%)" },
      { label: "Support", value: "Slack (24h)" },
      { label: "Features", value: "Full SSO/SAML, audit logs, SCIM, self-healing" },
    ],
    cta: "Start Trial",
    ctaHref: "https://app.5dlabs.ai",
    color: "magenta",
  },
  {
    name: "Enterprise",
    price: "Custom",
    period: "",
    description: "For large organizations with compliance needs",
    highlight: false,
    badge: null,
    infrastructure: "Managed or Self-Hosted",
    features: [
      { label: "CodeRuns included", value: "Custom allotment" },
      { label: "Overage rate", value: "$0.50/run (negotiable)" },
      { label: "Users", value: "Unlimited" },
      { label: "Repositories", value: "Unlimited" },
      { label: "AI API keys", value: "BYOK, Managed, or Volume" },
      { label: "Support", value: "Dedicated CSM, 4h SLA" },
      { label: "Features", value: "Custom integrations, source access (NDA)" },
    ],
    cta: "Contact Sales",
    ctaHref: "mailto:sales@5dlabs.ai",
    color: "purple",
  },
];

const comparisonFeatures = [
  {
    category: "Platform",
    features: [
      { name: "CodeRuns/month", free: "50", team: "200", growth: "1,000", enterprise: "Custom" },
      { name: "Users", free: "1", team: "10", growth: "Unlimited", enterprise: "Unlimited" },
      { name: "Repositories", free: "5", team: "Unlimited", growth: "Unlimited", enterprise: "Unlimited" },
      { name: "All 13 AI Agents", free: true, team: true, growth: true, enterprise: true },
      { name: "GitHub Integration", free: true, team: true, growth: true, enterprise: true },
    ],
  },
  {
    category: "Infrastructure",
    features: [
      { name: "Fully Managed", free: true, team: true, growth: true, enterprise: true },
      { name: "Self-Hosted Option", free: false, team: false, growth: false, enterprise: true },
      { name: "Dedicated Namespace", free: false, team: false, growth: true, enterprise: true },
      { name: "Self-Healing Infrastructure", free: false, team: false, growth: true, enterprise: true },
    ],
  },
  {
    category: "Security & Compliance",
    features: [
      { name: "SSO (Google/Microsoft)", free: false, team: true, growth: true, enterprise: true },
      { name: "SAML/OIDC", free: false, team: false, growth: true, enterprise: true },
      { name: "Audit Logs", free: false, team: false, growth: true, enterprise: true },
      { name: "SCIM Provisioning", free: false, team: false, growth: true, enterprise: true },
      { name: "SOC 2 Compliance", free: false, team: false, growth: false, enterprise: true },
      { name: "Source Code Access (NDA)", free: false, team: false, growth: false, enterprise: true },
    ],
  },
  {
    category: "Support",
    features: [
      { name: "Community (Discord)", free: true, team: true, growth: true, enterprise: true },
      { name: "Email Support", free: false, team: "48h", growth: "24h", enterprise: "4h SLA" },
      { name: "Slack Connect", free: false, team: false, growth: true, enterprise: true },
      { name: "Dedicated CSM", free: false, team: false, growth: false, enterprise: true },
      { name: "Custom SLA", free: false, team: false, growth: false, enterprise: true },
    ],
  },
];

const faqs = [
  {
    question: "What is a CodeRun?",
    answer: "A CodeRun is a single execution of an AI agent task—from the moment it starts to completion. This includes code generation, testing, security audits, or any agent action. Each run is metered by execution time.",
  },
  {
    question: "Can I use my own API keys?",
    answer: "Yes. BYOK (Bring Your Own Keys) is supported on all tiers. Use your Anthropic, OpenAI, or Google API keys directly. We never see your keys. They stay encrypted behind 5D Vault, our managed secrets layer.",
  },
  {
    question: "What does 'Fully Managed' mean?",
    answer: "You don't touch any infrastructure. No Kubernetes knowledge required. We handle everything—server provisioning, deployments, scaling, monitoring, and self-healing. You just connect GitHub and go.",
  },
  {
    question: "Can Enterprise customers self-host?",
    answer: "Yes. Enterprise tier includes the option to deploy on your own infrastructure for compliance or security requirements. We provide Talos Linux bootstrapping and ongoing support. All other tiers are fully managed only.",
  },
  {
    question: "What happens if I exceed my CodeRun limit?",
    answer: "You'll be charged the overage rate for your tier. Free: $3/run, Team: $1.50/run, Growth: $0.75/run, Enterprise: $0.50/run (negotiable). You can also upgrade to a higher tier anytime.",
  },
  {
    question: "Do annual plans save money?",
    answer: "Yes! Annual commitments receive 2 months free (pay for 10, get 12). Team annual is $1,990/year, Growth annual is $4,990/year. Enterprise pricing is custom.",
  },
];

export default function PricingPage() {
  return (
    <div className="relative min-h-screen overflow-hidden">
      {/* Background layers */}
      <div className="fixed inset-0 bg-gradient-to-b from-background via-background to-[oklch(0.06_0.03_260)] z-0" />
      <div className="fixed inset-0 circuit-bg z-0" />
      <div className="fixed inset-0 noise-overlay z-0" />

      {/* Header */}
      <Header />

      {/* Content */}
      <main className="relative z-10 pt-24">
        {/* Hero Section */}
        <section className="py-16 px-6">
          <div className="max-w-6xl mx-auto text-center">
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.8 }}
            >
              <motion.div
                initial={{ opacity: 0, scale: 0.95 }}
                animate={{ opacity: 1, scale: 1 }}
                transition={{ delay: 0.2, duration: 0.8, ease: "easeOut" }}
                className="inline-flex items-center gap-2 px-4 py-2 rounded-full border border-cyan/30 bg-cyan/5 mb-8"
              >
                <span className="w-2 h-2 rounded-full bg-cyan animate-[pulse_3s_ease-in-out_infinite]" />
                <span className="text-sm text-cyan font-medium">
                  Zero Infrastructure. Zero Cloud Tax.
                </span>
              </motion.div>

              <h1 className="text-4xl sm:text-5xl md:text-6xl font-bold tracking-tight mb-6">
                <span className="gradient-text glow-text-cyan">Simple Pricing</span>
                <br />
                <span className="text-foreground">For Every Team</span>
              </h1>

              <p className="text-xl text-muted-foreground max-w-2xl mx-auto mb-8">
                Start free, scale as you grow. All tiers include access to our complete 13-agent engineering collective on fully managed bare metal.
              </p>
            </motion.div>
          </div>
        </section>

        {/* Pricing Cards */}
        <section className="py-8 px-6">
          <div className="max-w-7xl mx-auto">
            <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
              {tiers.map((tier, index) => (
                <motion.div
                  key={tier.name}
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.6, delay: index * 0.1 }}
                  className={`relative p-6 rounded-2xl border backdrop-blur-sm flex flex-col ${
                    tier.highlight
                      ? "border-[oklch(0.7_0.25_320)] bg-[oklch(0.7_0.25_320)]/10"
                      : "border-border/50 bg-card/30"
                  }`}
                >
                  {tier.badge && (
                    <div className="absolute -top-3 left-1/2 -translate-x-1/2">
                      <span className="px-3 py-1 rounded-full text-xs font-semibold bg-gradient-to-r from-cyan-500 to-[oklch(0.7_0.25_320)] text-white shadow-lg">
                        {tier.badge}
                      </span>
                    </div>
                  )}

                  <div className="mb-6">
                    <h3 className="text-xl font-bold mb-1">{tier.name}</h3>
                    <p className="text-sm text-muted-foreground mb-4">{tier.description}</p>
                    <div className="flex items-baseline gap-1">
                      <span className="text-4xl font-bold gradient-text">{tier.price}</span>
                      <span className="text-muted-foreground">{tier.period}</span>
                    </div>
                  </div>

                  <div className="mb-4 px-3 py-2 rounded-lg bg-cyan/5 border border-cyan/20">
                    <span className="text-xs text-cyan font-medium">{tier.infrastructure}</span>
                  </div>

                  <ul className="space-y-3 mb-6 flex-grow">
                    {tier.features.map((feature, i) => (
                      <li key={i} className="flex justify-between text-sm">
                        <span className="text-muted-foreground">{feature.label}</span>
                        <span className="font-medium text-foreground">{feature.value}</span>
                      </li>
                    ))}
                  </ul>

                  <a
                    href={tier.ctaHref}
                    className={`w-full py-3 rounded-lg font-semibold text-center transition-all ${
                      tier.highlight
                        ? "bg-gradient-to-r from-cyan-500 to-[oklch(0.7_0.25_320)] text-white shadow-lg shadow-[oklch(0.7_0.25_320)]/30 hover:shadow-[oklch(0.7_0.25_320)]/50 hover:scale-105"
                        : "bg-secondary text-foreground hover:bg-secondary/80"
                    }`}
                  >
                    {tier.cta}
                  </a>
                </motion.div>
              ))}
            </div>
          </div>
        </section>

        {/* Value Props */}
        <section className="py-16 px-6 border-t border-border/30 mt-12">
          <div className="max-w-6xl mx-auto">
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.8 }}
              className="text-center mb-12"
            >
              <h2 className="text-3xl font-bold mb-4">
                Every Plan <span className="gradient-text">Includes</span>
              </h2>
            </motion.div>

            <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-6">
              {[
                {
                  icon: (
                    <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                    </svg>
                  ),
                  title: "All 13 Agents",
                  description: "Morgan, Rex, Grizz, Nova, Blaze, Tap, Spark, Cleo, Cipher, Tess, Stitch, Atlas, Bolt",
                  color: "cyan",
                },
                {
                  icon: (
                    <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2" />
                    </svg>
                  ),
                  title: "Bare Metal Infrastructure",
                  description: "60-80% cheaper than AWS/GCP. Zero cloud tax, maximum performance.",
                  color: "green",
                },
                {
                  icon: (
                    <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                    </svg>
                  ),
                  title: "CLI Agnostic",
                  description: "Claude Code, Cursor, Factory, Codex, Gemini—use what you love.",
                  color: "magenta",
                },
                {
                  icon: (
                    <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
                    </svg>
                  ),
                  title: "BYOK Support",
                  description: "Bring your own Anthropic, OpenAI, or Google keys. Zero vendor lock-in.",
                  color: "yellow",
                },
              ].map((item, index) => (
                <motion.div
                  key={item.title}
                  initial={{ opacity: 0, y: 20 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true }}
                  transition={{ duration: 0.5, delay: index * 0.1 }}
                  className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm text-center"
                >
                  <div className={`w-12 h-12 rounded-lg bg-${item.color}/10 flex items-center justify-center mx-auto mb-4 text-${item.color === "magenta" ? "[oklch(0.7_0.25_320)]" : item.color === "cyan" ? "cyan" : item.color === "green" ? "green-500" : "yellow-500"}`}>
                    {item.icon}
                  </div>
                  <h3 className="font-semibold mb-2">{item.title}</h3>
                  <p className="text-sm text-muted-foreground">{item.description}</p>
                </motion.div>
              ))}
            </div>
          </div>
        </section>

        {/* Comparison Table */}
        <section className="py-16 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto">
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.8 }}
              className="text-center mb-12"
            >
              <h2 className="text-3xl font-bold mb-4">
                Compare <span className="gradient-text">Plans</span>
              </h2>
              <p className="text-muted-foreground">
                See exactly what&apos;s included in each tier
              </p>
            </motion.div>

            <motion.div
              initial={{ opacity: 0 }}
              whileInView={{ opacity: 1 }}
              viewport={{ once: true }}
              transition={{ duration: 0.8, delay: 0.2 }}
              className="overflow-x-auto"
            >
              <table className="w-full min-w-[800px]">
                <thead>
                  <tr className="border-b border-border/50">
                    <th className="text-left py-4 px-4 font-semibold text-muted-foreground">Feature</th>
                    <th className="text-center py-4 px-4 font-semibold">Free</th>
                    <th className="text-center py-4 px-4 font-semibold">Team</th>
                    <th className="text-center py-4 px-4 font-semibold text-[oklch(0.7_0.25_320)]">Growth</th>
                    <th className="text-center py-4 px-4 font-semibold">Enterprise</th>
                  </tr>
                </thead>
                <tbody>
                  {comparisonFeatures.map((category) => (
                    <>
                      <tr key={category.category} className="bg-card/20">
                        <td colSpan={5} className="py-3 px-4 font-semibold text-cyan text-sm uppercase tracking-wider">
                          {category.category}
                        </td>
                      </tr>
                      {category.features.map((feature) => (
                        <tr key={feature.name} className="border-b border-border/30 hover:bg-card/30 transition-colors">
                          <td className="py-3 px-4 text-sm">{feature.name}</td>
                          {["free", "team", "growth", "enterprise"].map((tier) => {
                            const value = feature[tier as keyof typeof feature];
                            return (
                              <td key={tier} className="text-center py-3 px-4">
                                {typeof value === "boolean" ? (
                                  value ? (
                                    <svg className="w-5 h-5 text-green-500 mx-auto" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                                    </svg>
                                  ) : (
                                    <svg className="w-5 h-5 text-muted-foreground/30 mx-auto" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                                    </svg>
                                  )
                                ) : (
                                  <span className="text-sm">{value}</span>
                                )}
                              </td>
                            );
                          })}
                        </tr>
                      ))}
                    </>
                  ))}
                </tbody>
              </table>
            </motion.div>
          </div>
        </section>

        {/* FAQ Section */}
        <section className="py-16 px-6 border-t border-border/30">
          <div className="max-w-4xl mx-auto">
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.8 }}
              className="text-center mb-12"
            >
              <h2 className="text-3xl font-bold mb-4">
                Frequently Asked <span className="gradient-text">Questions</span>
              </h2>
            </motion.div>

            <div className="space-y-4">
              {faqs.map((faq, index) => (
                <motion.div
                  key={faq.question}
                  initial={{ opacity: 0, y: 20 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true }}
                  transition={{ duration: 0.5, delay: index * 0.1 }}
                  className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm"
                >
                  <h3 className="text-lg font-semibold mb-2">{faq.question}</h3>
                  <p className="text-muted-foreground">{faq.answer}</p>
                </motion.div>
              ))}
            </div>
          </div>
        </section>

        {/* CTA Section */}
        <section className="py-20 px-6 border-t border-border/30">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.8 }}
            className="max-w-2xl mx-auto text-center"
          >
            <h2 className="text-3xl sm:text-4xl font-bold mb-4">
              Ready to <span className="gradient-text">Ship Faster</span>?
            </h2>
            <p className="text-lg text-muted-foreground mb-8">
              Start free today. No credit card required.
            </p>
            <div className="flex flex-col sm:flex-row justify-center gap-4">
              <a
                href="https://app.5dlabs.ai"
                className="px-8 py-4 rounded-lg bg-gradient-to-r from-cyan-500 to-blue-500 text-white font-semibold text-lg hover:from-cyan-600 hover:to-blue-600 transition-all shadow-xl shadow-cyan-500/30 hover:shadow-cyan-500/50 hover:scale-105"
              >
                Start Free
              </a>
              <a
                href="mailto:sales@5dlabs.ai"
                className="px-8 py-4 rounded-lg border border-border bg-card/30 text-foreground font-semibold text-lg hover:bg-card/50 transition-all"
              >
                Contact Sales
              </a>
            </div>
          </motion.div>
        </section>

        {/* Footer */}
        <footer className="py-8 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto flex flex-col sm:flex-row items-center justify-between gap-4">
            <a href={homeHref} className="flex items-center gap-2" aria-label="Back to 5D Labs">
              <Image
                src="/5dlabs-logo-3d.jpg"
                alt="5D Labs"
                width={160}
                height={40}
                className="h-10 w-auto opacity-90"
              />
            </a>
            <p className="text-sm text-muted-foreground">
              © {new Date().getFullYear()} 5D Labs. Transmitting from the Fifth Dimension.
            </p>
          </div>
        </footer>
      </main>
    </div>
  );
}

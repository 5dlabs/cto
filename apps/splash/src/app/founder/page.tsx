"use client";

import { motion } from "framer-motion";
import { RainEffect } from "@/components/rain-effect";
import { Header } from "@/components/header";
import { Footer } from "@/components/footer";

export default function FounderPage() {
  return (
    <div className="relative min-h-screen overflow-hidden">
      {/* Background layers */}
      <div className="fixed inset-0 bg-gradient-to-b from-background via-background to-[oklch(0.06_0.03_260)] z-0" />
      <div className="fixed inset-0 circuit-bg z-0" />
      <RainEffect />
      <div className="fixed inset-0 noise-overlay z-0" />

      <Header />

      <main className="relative z-10 pt-24">
        {/* Hero */}
        <section className="py-20 px-6">
          <div className="max-w-4xl mx-auto">
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.8 }}
              className="flex flex-col md:flex-row items-center gap-12"
            >
              {/* Avatar placeholder */}
              <motion.div
                initial={{ opacity: 0, scale: 0.9 }}
                animate={{ opacity: 1, scale: 1 }}
                transition={{ delay: 0.2, duration: 0.8 }}
                className="shrink-0"
              >
                <div className="w-48 h-48 rounded-full bg-gradient-to-br from-cyan-500 to-blue-500 flex items-center justify-center ring-4 ring-cyan/20">
                  <span className="text-6xl font-bold text-white">JF</span>
                </div>
              </motion.div>

              {/* Info */}
              <div>
                <motion.div
                  initial={{ opacity: 0, y: 12 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: 0.3, duration: 0.8 }}
                >
                  <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-cyan/20 bg-cyan/5 mb-4">
                    <span className="text-xs text-cyan font-medium uppercase tracking-wider">
                      Founder & CEO
                    </span>
                  </div>
                  <h1 className="text-4xl sm:text-5xl font-bold mb-4">
                    <span className="gradient-text">Jonathon Fritz</span>
                  </h1>
                  <p className="text-lg text-muted-foreground max-w-lg">
                    Building 5D Labs at the intersection of AI, crypto, and
                    autonomous systems. Obsessed with the idea that software can
                    build itself — and proving it daily.
                  </p>
                </motion.div>

                {/* Social links */}
                <motion.div
                  initial={{ opacity: 0, y: 12 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: 0.5, duration: 0.8 }}
                  className="flex gap-4 mt-6"
                >
                  <a
                    href="https://github.com/5dlabs"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-muted-foreground hover:text-foreground transition-colors"
                    aria-label="GitHub"
                    data-umami-event="founder-github"
                  >
                    <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
                      <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12" />
                    </svg>
                  </a>
                  <a
                    href="https://x.com/5dlabs"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-muted-foreground hover:text-foreground transition-colors"
                    aria-label="X / Twitter"
                    data-umami-event="founder-twitter"
                  >
                    <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
                      <path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z" />
                    </svg>
                  </a>
                  <a
                    href="https://cal.com/jonathon-fritz-2uhdqe/discovery"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-muted-foreground hover:text-foreground transition-colors"
                    aria-label="Schedule a Call"
                    data-umami-event="founder-schedule-call"
                  >
                    <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                    </svg>
                  </a>
                </motion.div>
              </div>
            </motion.div>
          </div>
        </section>

        {/* Background & Vision */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-4xl mx-auto">
            <div className="grid md:grid-cols-2 gap-12">
              {/* Background */}
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6 }}
              >
                <h2 className="text-2xl font-bold mb-6">
                  <span className="gradient-text">Background</span>
                </h2>
                <div className="space-y-4 text-muted-foreground">
                  <p>
                    A technologist with deep experience across AI, infrastructure,
                    and distributed systems. The kind of person who builds their
                    own bare-metal Kubernetes clusters for fun.
                  </p>
                  <p>
                    Before founding 5D Labs, Jonathon spent years at the
                    intersection of software engineering and emerging technology —
                    always pushing the boundary of what autonomous systems can do.
                  </p>
                  <p className="text-sm italic text-muted-foreground/60">
                    More details coming soon.
                  </p>
                </div>
              </motion.div>

              {/* Vision */}
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: 0.1 }}
              >
                <h2 className="text-2xl font-bold mb-6">
                  <span className="gradient-text">Vision</span>
                </h2>
                <div className="space-y-4 text-muted-foreground">
                  <p>
                    The future of startups isn&apos;t a team of 50 people in an
                    office. It&apos;s a founder with a vision, backed by
                    autonomous AI agents that can build, test, deploy, and iterate
                    at machine speed.
                  </p>
                  <p>
                    5D Labs exists to prove that thesis. By building OpenClaw and
                    deploying it across multiple ventures, we&apos;re showing that
                    the startup studio model + AI agents = a fundamentally
                    different — and better — way to build companies.
                  </p>
                </div>
              </motion.div>
            </div>
          </div>
        </section>

        {/* What Drives Me */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-4xl mx-auto">
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.8 }}
              className="text-center mb-12"
            >
              <h2 className="text-2xl font-bold mb-4">
                What <span className="gradient-text">Drives</span> This
              </h2>
            </motion.div>

            <div className="grid sm:grid-cols-3 gap-6">
              {[
                {
                  title: "AI-Native Everything",
                  description:
                    "Every process, every workflow, every decision loop should be augmented or fully automated by AI agents.",
                  icon: (
                    <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
                    </svg>
                  ),
                },
                {
                  title: "Own Your Infrastructure",
                  description:
                    "Bare metal over cloud. Self-hosted over SaaS. Sovereignty over convenience — with the tooling to make it painless.",
                  icon: (
                    <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2" />
                    </svg>
                  ),
                },
                {
                  title: "Build in Public",
                  description:
                    "Open source by default. Transparent about what works, what doesn't, and what we're learning along the way.",
                  icon: (
                    <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                    </svg>
                  ),
                },
              ].map((item, i) => (
                <motion.div
                  key={item.title}
                  initial={{ opacity: 0, y: 20 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true }}
                  transition={{ duration: 0.5, delay: i * 0.1 }}
                  className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm text-center"
                >
                  <div className="w-12 h-12 rounded-lg bg-cyan/10 flex items-center justify-center mx-auto mb-4 text-cyan">
                    {item.icon}
                  </div>
                  <h3 className="text-lg font-semibold mb-2">{item.title}</h3>
                  <p className="text-sm text-muted-foreground">
                    {item.description}
                  </p>
                </motion.div>
              ))}
            </div>
          </div>
        </section>

        <Footer />
      </main>
    </div>
  );
}

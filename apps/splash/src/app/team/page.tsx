"use client";

import { motion } from "framer-motion";
import { RainEffect } from "@/components/rain-effect";
import { Header } from "@/components/header";
import { Footer } from "@/components/footer";
import { Avatar, AvatarImage, AvatarFallback } from "@/components/ui/avatar";
import { cn } from "@/lib/utils";

interface Agent {
  name: string;
  role: string;
  avatar?: string;
  color: string;
  personality: string;
  stack: string[];
}

interface Squad {
  title: string;
  emoji: string;
  agents: Agent[];
}

const squads: Squad[] = [
  {
    title: "The Boss",
    emoji: "🎯",
    agents: [
      {
        name: "Morgan",
        role: "Technical Program Manager",
        avatar: "/agents/morgan-avatar-512.png",
        color: "from-cyan-400 to-pink-500",
        personality: "Keeps the trains running. Decomposes PRDs, assigns tasks, tracks progress. Runs a tight ship.",
        stack: ["Linear", "GitHub", "PRDs"],
      },
    ],
  },
  {
    title: "The Builders",
    emoji: "🦀",
    agents: [
      {
        name: "Rex",
        role: "Rust Architect",
        avatar: "/agents/rex-avatar-512.png",
        color: "from-orange-500 to-red-500",
        personality: "Lives for zero-cost abstractions. If it compiles, it ships. Opinions on lifetimes are... strong.",
        stack: ["Rust", "Tokio", "Axum"],
      },
      {
        name: "Grizz",
        role: "Go Specialist",
        avatar: "/agents/grizz-avatar-512.png",
        color: "from-amber-500 to-orange-400",
        personality: "Pragmatic. Ships clean Go services without overengineering. Fan of simplicity and goroutines.",
        stack: ["Go", "gRPC", "PostgreSQL"],
      },
      {
        name: "Nova",
        role: "Node.js Engineer",
        avatar: "/agents/nova-avatar-512.png",
        color: "from-purple-500 to-cyan-400",
        personality: "The speed demon. Gets APIs up and running faster than you can write the spec.",
        stack: ["Node.js", "TypeScript", "Fastify"],
      },
    ],
  },
  {
    title: "The Designers",
    emoji: "🎨",
    agents: [
      {
        name: "Blaze",
        role: "Web App Developer",
        avatar: "/agents/blaze-avatar-512.png",
        color: "from-blue-500 to-cyan-500",
        personality: "Pixel-perfect or bust. React components so clean they belong in a museum.",
        stack: ["React", "Next.js", "shadcn/ui"],
      },
      {
        name: "Tap",
        role: "Mobile Developer",
        avatar: "/agents/tap-avatar-512.png",
        color: "from-green-500 to-emerald-400",
        personality: "One codebase, two platforms. Makes cross-platform feel native because it is.",
        stack: ["Expo", "React Native", "NativeWind"],
      },
      {
        name: "Spark",
        role: "Desktop Developer",
        avatar: "/agents/spark-avatar-512.png",
        color: "from-blue-500 to-yellow-400",
        personality: "Bringing the web to the desktop, with native superpowers. Offline-first evangelist.",
        stack: ["Electron", "Tauri", "React"],
      },
    ],
  },
  {
    title: "The Guardians",
    emoji: "🛡️",
    agents: [
      {
        name: "Cleo",
        role: "Quality Guardian",
        avatar: "/agents/cleo-avatar-512.png",
        color: "from-emerald-500 to-teal-500",
        personality: "Will refactor your code and you'll thank her. Best practices aren't optional.",
        stack: ["Code Review", "Patterns", "Standards"],
      },
      {
        name: "Cipher",
        role: "Security Sentinel",
        avatar: "/agents/cipher-avatar-512.png",
        color: "from-red-500 to-rose-500",
        personality: "Paranoid by design. If there's a vulnerability, Cipher already found it.",
        stack: ["Trivy", "Gitleaks", "OWASP"],
      },
      {
        name: "Tess",
        role: "Testing Genius",
        avatar: "/agents/tess-avatar-512.png",
        color: "from-violet-500 to-purple-500",
        personality: "100% coverage or she's not done. Writes tests you didn't know you needed.",
        stack: ["Jest", "Playwright", "Vitest"],
      },
    ],
  },
  {
    title: "The Operators",
    emoji: "🚀",
    agents: [
      {
        name: "Stitch",
        role: "Code Reviewer",
        avatar: "/agents/stitch-avatar-512.png",
        color: "from-orange-500 to-blue-400",
        personality: "Reviews every PR with surgical precision. Catches what others miss. Politely.",
        stack: ["PR Review", "Linting", "Standards"],
      },
      {
        name: "Atlas",
        role: "Integration Master",
        avatar: "/agents/atlas-avatar-512.png",
        color: "from-slate-500 to-zinc-500",
        personality: "The gatekeeper. If it merges, Atlas approved it. If it conflicts, Atlas fixed it.",
        stack: ["Git", "Rebasing", "CI/CD"],
      },
      {
        name: "Bolt",
        role: "Deployment Specialist",
        avatar: "/agents/bolt-avatar-512.png",
        color: "from-yellow-500 to-amber-500",
        personality: "Zero-downtime deployments are the only kind of deployments. GitOps purist.",
        stack: ["Kubernetes", "Helm", "GitOps"],
      },
    ],
  },
];

function AgentTeamCard({ agent, index, squadIndex }: { agent: Agent; index: number; squadIndex: number }) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      transition={{
        delay: squadIndex * 0.08 + index * 0.04,
        duration: 0.7,
        ease: "easeOut",
      }}
      whileHover={{ scale: 1.01, y: -2 }}
      className="group"
    >
      <div
        className={cn(
          "relative p-5 rounded-xl",
          "bg-card border border-border",
          "hover:border-primary/50",
          "transition-all duration-300"
        )}
      >
        {/* Glow */}
        <div
          className={cn(
            "absolute inset-0 rounded-xl opacity-0 group-hover:opacity-20",
            "bg-gradient-to-br transition-opacity duration-300",
            agent.color
          )}
        />

        <div className="relative z-10 flex items-start gap-4">
          {/* Avatar */}
          <Avatar className="size-16 ring-2 ring-border group-hover:ring-primary/50 transition-all shrink-0">
            {agent.avatar ? (
              <AvatarImage src={agent.avatar} alt={agent.name} />
            ) : null}
            <AvatarFallback
              className={cn(
                "bg-gradient-to-br text-white text-2xl font-bold",
                agent.color
              )}
            >
              {agent.name.charAt(0)}
            </AvatarFallback>
          </Avatar>

          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <p className="font-semibold text-lg text-foreground">{agent.name}</p>
              <div
                className={cn(
                  "size-2 rounded-full shrink-0",
                  "bg-gradient-to-r animate-[pulse_3s_ease-in-out_infinite]",
                  agent.color
                )}
              />
            </div>
            <p className="text-xs text-cyan mb-2">{agent.role}</p>
            <p className="text-sm text-muted-foreground mb-3">
              {agent.personality}
            </p>
            <div className="flex flex-wrap gap-1">
              {agent.stack.map((tech) => (
                <span
                  key={tech}
                  className="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium bg-muted/50 text-muted-foreground"
                >
                  {tech}
                </span>
              ))}
            </div>
          </div>
        </div>
      </div>
    </motion.div>
  );
}

export default function TeamPage() {
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
          <div className="max-w-6xl mx-auto text-center">
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.8 }}
            >
              <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full border border-cyan/30 bg-cyan/5 mb-8">
                <span className="w-2 h-2 rounded-full bg-cyan animate-[pulse_3s_ease-in-out_infinite]" />
                <span className="text-sm text-cyan font-medium">
                  13 Agents. 0 Humans.*
                </span>
              </div>

              <h1 className="text-4xl sm:text-5xl md:text-6xl font-bold tracking-tight mb-6">
                Meet the <span className="gradient-text glow-text-cyan">Team</span>
              </h1>

              <p className="text-xl text-muted-foreground max-w-2xl mx-auto mb-4">
                Our engineering team never sleeps, never takes PTO, and
                definitely never argues about tabs vs. spaces. They&apos;re AI
                agents — and they&apos;re really good at their jobs.
              </p>

              <p className="text-sm text-muted-foreground/60 max-w-lg mx-auto">
                *Well, one human. See the{" "}
                <a href="/founder" className="text-cyan hover:text-cyan/80 underline underline-offset-4 transition-colors">
                  Founder
                </a>{" "}
                page.
              </p>
            </motion.div>
          </div>
        </section>

        {/* Agent Grid */}
        <section className="pb-20 px-6">
          <div className="max-w-6xl mx-auto space-y-10">
            {squads.map((squad, squadIndex) => (
              <motion.div
                key={squad.title}
                initial={{ opacity: 0, y: 10 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ delay: squadIndex * 0.08, duration: 0.8, ease: "easeOut" }}
              >
                {/* Squad header */}
                <div className="flex items-center gap-3 mb-4">
                  <span className="text-xl">{squad.emoji}</span>
                  <h3 className="text-sm font-medium text-muted-foreground uppercase tracking-wider">
                    {squad.title}
                  </h3>
                  <div className="flex-1 h-px bg-border/50" />
                </div>

                {/* Cards */}
                <div
                  className={cn(
                    "grid gap-4",
                    squad.agents.length === 1
                      ? "grid-cols-1 max-w-lg"
                      : "grid-cols-1 sm:grid-cols-2 lg:grid-cols-3"
                  )}
                >
                  {squad.agents.map((agent, agentIndex) => (
                    <AgentTeamCard
                      key={agent.name}
                      agent={agent}
                      index={agentIndex}
                      squadIndex={squadIndex}
                    />
                  ))}
                </div>
              </motion.div>
            ))}
          </div>
        </section>

        {/* Fun CTA */}
        <section className="py-20 px-6 border-t border-border/30">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.8 }}
            className="max-w-2xl mx-auto text-center"
          >
            <h2 className="text-3xl font-bold mb-4">
              Want to see them in <span className="gradient-text">action</span>?
            </h2>
            <p className="text-lg text-muted-foreground mb-8">
              Watch our agents ship features autonomously on the CTO platform.
            </p>
            <a
              href="https://cto.5dlabs.ai"
              className="inline-flex px-8 py-4 rounded-lg bg-gradient-to-r from-cyan-500 to-blue-500 text-white font-semibold text-lg hover:from-cyan-600 hover:to-blue-600 transition-all shadow-xl shadow-cyan-500/30 hover:shadow-cyan-500/50 hover:scale-105"
            >
              Visit CTO
            </a>
          </motion.div>
        </section>

        <Footer />
      </main>
    </div>
  );
}

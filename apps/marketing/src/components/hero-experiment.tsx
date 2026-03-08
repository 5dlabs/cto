"use client";

import { WaitlistForm } from "@/components/waitlist-form";
import { featureFlags } from "@/config/feature-flags";

export function HeroExperiment() {
  return (
    <section id="hero" className="min-h-screen flex flex-col items-center justify-center px-6 py-20 pt-24">
      <div className="max-w-4xl mx-auto text-center">
        <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full border border-cyan/30 bg-cyan/5 mb-8">
          <span className="w-2 h-2 rounded-full bg-cyan animate-[pulse_3s_ease-in-out_infinite]" />
          <span className="text-sm text-cyan font-medium">Idea to Production - Autonomously</span>
        </div>

        <h1 className="text-5xl sm:text-6xl md:text-7xl font-bold tracking-tight mb-6">
          <span className="gradient-text glow-text-cyan">From Idea to Impact,</span>
          <br />
          <span className="text-foreground">Fast</span>
        </h1>

        <p className="text-xl sm:text-2xl text-muted-foreground max-w-2xl mx-auto mb-10">
          Describe what you want. CTO plans it, builds it, tests it, and ships it - with a full team of AI agents working in parallel. You think it,{" "}
          <span className="text-foreground">it gets built.</span>
        </p>

        <div className="flex flex-col sm:flex-row justify-center gap-4 mb-16">
          {featureFlags.showStartNowButton && (
            <a
              href="https://app.5dlabs.ai"
              className="px-8 py-4 rounded-lg bg-gradient-to-r from-cyan-500 to-blue-500 text-white font-semibold text-lg hover:from-cyan-600 hover:to-blue-600 transition-all shadow-xl shadow-cyan-500/30 hover:shadow-cyan-500/50 hover:scale-105"
            >
              Start Now
            </a>
          )}
          <WaitlistForm />
        </div>

        <div className="flex flex-wrap justify-center gap-8 text-sm text-muted-foreground">
          <div className="flex items-center gap-2">
            <span className="text-2xl font-bold text-foreground">60-80%</span>
            <span>cost savings vs cloud</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-2xl font-bold text-foreground">17+</span>
            <span>infra providers</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-2xl font-bold text-foreground">∞</span>
            <span>faster shipping</span>
          </div>
        </div>
      </div>

      <div className="absolute bottom-10 left-1/2 -translate-x-1/2">
        <div className="w-6 h-10 rounded-full border-2 border-muted-foreground/30 flex justify-center pt-2 scroll-bounce">
          <div className="w-1 h-2 rounded-full bg-cyan" />
        </div>
      </div>
    </section>
  );
}

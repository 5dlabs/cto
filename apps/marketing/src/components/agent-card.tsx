"use client";

import { useState, useRef, useEffect } from "react";
import { motion } from "framer-motion";
import { Avatar, AvatarImage, AvatarFallback } from "@/components/ui/avatar";
import { cn } from "@/lib/utils";

export type Agent = {
  name: string;
  role: string;
  color: string;
  avatar?: string;
  description?: string;
  badge?: string;
  tools?: string[];
  skills?: string[];
};

export type AgentSquad = {
  title: string;
  emoji: string;
  agents: Agent[];
};

interface AgentCardProps {
  agent: Agent;
}

export function AgentCard({ agent }: AgentCardProps) {
  const [flipped, setFlipped] = useState(false);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const hasDetails = (agent.tools && agent.tools.length > 0) || (agent.skills && agent.skills.length > 0);

  useEffect(() => {
    return () => {
      if (timeoutRef.current) clearTimeout(timeoutRef.current);
    };
  }, []);

  const handleMouseEnter = () => {
    if (timeoutRef.current) clearTimeout(timeoutRef.current);
  };

  const handleMouseLeave = () => {
    if (flipped) {
      timeoutRef.current = setTimeout(() => setFlipped(false), 3000);
    }
  };

  return (
    <button
      type="button"
      className={cn(
        "group block w-full text-left",
        hasDetails ? "cursor-pointer touch-manipulation" : "cursor-default"
      )}
      style={{ perspective: "1000px" }}
      onClick={() => hasDetails && setFlipped((f) => !f)}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      disabled={!hasDetails}
      aria-pressed={hasDetails ? flipped : undefined}
      aria-label={`${agent.name} card${hasDetails ? flipped ? ", tap to return" : ", flip for details" : ""}`}
    >
      <motion.div
        className="relative h-full min-h-[176px] sm:min-h-[188px] lg:min-h-[196px] w-full"
        style={{ transformStyle: "preserve-3d" }}
        animate={{ rotateY: flipped ? 180 : 0 }}
        transition={{ duration: 0.5, ease: "easeInOut" }}
        initial={false}
      >
        {/* Front: avatar, name, role, hint */}
        <div
          className={cn(
            "absolute inset-0 flex items-start sm:items-center gap-3 sm:gap-4 lg:gap-5 rounded-xl p-3 sm:p-4 lg:p-5",
            "bg-card border border-border",
            "hover:border-primary/50 transition-colors duration-300"
          )}
          style={{ backfaceVisibility: "hidden" }}
        >
          <div
            className={cn(
              "absolute inset-0 rounded-xl bg-gradient-to-br opacity-0 transition-opacity duration-300 group-hover:opacity-20",
              agent.color
            )}
            aria-hidden
          />
          <Avatar className="relative z-10 size-[60px] sm:size-[88px] lg:size-[112px] shrink-0 ring-2 ring-border">
            {agent.avatar ? (
              <AvatarImage src={agent.avatar} alt={agent.name} />
            ) : null}
            <AvatarFallback
              className={cn(
                "bg-gradient-to-br text-white text-2xl sm:text-3xl font-bold",
                agent.color
              )}
            >
              {agent.name.charAt(0)}
            </AvatarFallback>
          </Avatar>
          <div className="relative z-10 min-w-0 flex-1 self-center sm:self-auto">
            <p className="font-semibold text-[15px] sm:text-lg leading-tight text-foreground">
              {agent.name}
            </p>
            <p className="mt-1 text-[11px] sm:text-sm leading-snug text-muted-foreground">
              {agent.role}
            </p>
            {agent.badge && (
              <p className="mt-2 text-[9px] sm:text-[10px] uppercase tracking-[0.18em] text-[oklch(0.7_0.25_320)]/80 font-medium">
                {agent.badge}
              </p>
            )}
            {hasDetails && !agent.badge && (
              <p className="mt-2 text-[9px] sm:text-[10px] uppercase tracking-[0.18em] text-muted-foreground/80">
                Flip for details
              </p>
            )}
          </div>
          {hasDetails && !agent.badge && (
            <div
              className={cn(
                "absolute right-3 top-3 z-10 size-2 rounded-full bg-gradient-to-r animate-[pulse_3s_ease-in-out_infinite]",
                agent.color
              )}
            />
          )}
        </div>

        {/* Back: tools + skills in two sections */}
        <div
          className="absolute inset-0 overflow-hidden rounded-xl border border-border bg-card p-3 sm:p-4 lg:p-5"
          style={{
            backfaceVisibility: "hidden",
            transform: "rotateY(180deg)",
          }}
        >
          <div
            className={cn(
              "absolute inset-0 rounded-xl bg-gradient-to-br opacity-10",
              agent.color
            )}
            aria-hidden
          />
          <div className="relative z-10 flex h-full min-h-0 flex-col">
            {agent.tools && agent.tools.length > 0 && (
              <div className="min-h-0">
                <p className="mb-2 text-[10px] sm:text-[11px] font-medium uppercase tracking-[0.18em] text-muted-foreground">
                  {agent.name} · tools
                </p>
                <div className="flex max-h-[68px] flex-wrap gap-1.5 overflow-y-auto pr-1 sm:max-h-[72px]">
                  {agent.tools.map((tool) => (
                    <span
                      key={tool}
                      className="rounded-md border border-white/10 bg-white/[0.04] px-2 py-1 font-mono text-[10px] sm:text-[11px] leading-tight text-foreground/90"
                    >
                      {tool}
                    </span>
                  ))}
                </div>
              </div>
            )}
            {agent.skills && agent.skills.length > 0 && (
              <div className="mt-3 min-h-0">
                <p className="mb-2 text-[10px] sm:text-[11px] font-medium uppercase tracking-[0.18em] text-muted-foreground">
                  skills
                </p>
                <div className="flex max-h-[68px] flex-wrap gap-1.5 overflow-y-auto pr-1 sm:max-h-[72px]">
                  {agent.skills.map((skill) => (
                    <span
                      key={skill}
                      className="rounded-md border border-white/10 bg-white/[0.04] px-2 py-1 font-mono text-[10px] sm:text-[11px] leading-tight text-foreground/90"
                    >
                      {skill}
                    </span>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>
      </motion.div>
    </button>
  );
}

interface AgentGridProps {
  squads: AgentSquad[];
}

export function AgentGrid({ squads }: AgentGridProps) {
  return (
    <div className="space-y-8">
      {squads.map((squad) => (
        <div key={squad.title}>
          <div className="mb-4 flex items-center gap-3">
            <span className="text-xl">{squad.emoji}</span>
            <h3 className="text-sm font-medium uppercase tracking-wider text-muted-foreground">
              {squad.title}
            </h3>
            <div className="h-px flex-1 bg-border/50" />
          </div>

          <div
            className={cn(
              "grid gap-4",
              squad.agents.length === 1
                ? "max-w-sm grid-cols-1 mx-auto"
                : squad.agents.length === 2
                  ? "grid-cols-1 sm:grid-cols-2"
                  : squad.agents.length === 4
                    ? "grid-cols-1 sm:grid-cols-2"
                    : "grid-cols-1 md:grid-cols-3"
            )}
          >
            {squad.agents.map((agent) => (
              <AgentCard
                key={agent.name}
                agent={agent}
              />
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

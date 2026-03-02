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
  index: number;
  squadIndex: number;
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
    <div
      className="group cursor-pointer"
      style={{ perspective: "1000px" }}
      onClick={() => hasDetails && setFlipped((f) => !f)}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      <motion.div
        className="relative h-full min-h-[150px] sm:min-h-[170px] lg:min-h-[180px] w-full"
        style={{ transformStyle: "preserve-3d" }}
        animate={{ rotateY: flipped ? 180 : 0 }}
        transition={{ duration: 0.5, ease: "easeInOut" }}
        initial={false}
      >
        {/* Front: avatar, name, role, hint */}
        <div
          className={cn(
            "absolute inset-0 flex items-center gap-3 sm:gap-4 lg:gap-5 rounded-xl p-3 sm:p-4 lg:p-5",
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
          <Avatar className="relative z-10 size-[72px] sm:size-[88px] lg:size-[112px] shrink-0 ring-2 ring-border">
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
          <div className="relative z-10 min-w-0 flex-1">
            <p className="font-semibold text-base sm:text-lg text-foreground truncate">
              {agent.name}
            </p>
            <p className="text-xs sm:text-sm text-muted-foreground truncate">
              {agent.role}
            </p>
            {hasDetails && (
              <p className="mt-1 text-[10px] uppercase tracking-widest text-muted-foreground/80">
                Click to flip →
              </p>
            )}
          </div>
          {hasDetails && (
            <div
              className={cn(
                "relative z-10 size-2 shrink-0 rounded-full bg-gradient-to-r animate-[pulse_3s_ease-in-out_infinite]",
                agent.color
              )}
            />
          )}
        </div>

        {/* Back: tools + skills in two sections */}
        <div
          className="absolute inset-0 flex flex-col justify-center rounded-xl border border-border bg-card p-3 sm:p-4 lg:p-5"
          style={{
            backfaceVisibility: "hidden",
            transform: "rotateY(180deg)",
          }}
        >
          {agent.tools && agent.tools.length > 0 && (
            <div className="mb-3">
              <p className="mb-1.5 text-xs font-medium uppercase tracking-widest text-muted-foreground">
                {agent.name} · tools
              </p>
              <div className="flex flex-wrap gap-x-3 gap-y-1 text-xs text-muted-foreground">
                {agent.tools.map((tool) => (
                  <span
                    key={tool}
                    className="font-mono tracking-tight text-foreground/90"
                  >
                    {tool}
                  </span>
                ))}
              </div>
            </div>
          )}
          {agent.skills && agent.skills.length > 0 && (
            <div>
              <p className="mb-1.5 text-xs font-medium uppercase tracking-widest text-muted-foreground">
                skills
              </p>
              <div className="flex flex-wrap gap-x-3 gap-y-1 text-xs text-muted-foreground">
                {agent.skills.map((skill) => (
                  <span
                    key={skill}
                    className="font-mono tracking-tight text-foreground/90"
                  >
                    {skill}
                  </span>
                ))}
              </div>
            </div>
          )}
          <p className="mt-3 text-[10px] uppercase tracking-widest text-muted-foreground/70">
            Click to flip back
          </p>
        </div>
      </motion.div>
    </div>
  );
}

interface AgentGridProps {
  squads: AgentSquad[];
}

export function AgentGrid({ squads }: AgentGridProps) {
  return (
    <div className="space-y-8">
      {squads.map((squad, squadIndex) => (
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
                  : "grid-cols-1 md:grid-cols-3"
            )}
          >
            {squad.agents.map((agent, agentIndex) => (
              <AgentCard
                key={agent.name}
                agent={agent}
                index={agentIndex}
                squadIndex={squadIndex}
              />
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

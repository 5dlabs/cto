"use client";

import { useRef, useCallback } from "react";
import Link from "next/link";
import { cn } from "@/lib/utils";

export interface Venture {
  name: string;
  tagline: string;
  description: string;
  tags: string[];
  color: string;
  href?: string;
  status: "live" | "building" | "exploring";
}

interface VentureCardProps {
  venture: Venture;
  index: number;
}

const statusConfig = {
  live: { label: "Live", bg: "bg-green-500/10", text: "text-green-400", dot: "bg-green-400" },
  building: { label: "Building", bg: "bg-cyan/10", text: "text-cyan", dot: "bg-cyan" },
  exploring: { label: "Exploring", bg: "bg-yellow-500/10", text: "text-yellow-400", dot: "bg-yellow-400" },
};

export function VentureCard({ venture }: VentureCardProps) {
  const status = statusConfig[venture.status];
  const isClickable = Boolean(venture.href);
  const cardRef = useRef<HTMLDivElement>(null);

  const handleMouseMove = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
    const el = cardRef.current;
    if (!el) return;
    const rect = el.getBoundingClientRect();
    el.style.setProperty("--spot-x", `${e.clientX - rect.left}px`);
    el.style.setProperty("--spot-y", `${e.clientY - rect.top}px`);
  }, []);

  const card = (
    <div className={cn("group", isClickable && "cursor-pointer")}>
      <div
        ref={cardRef}
        onMouseMove={handleMouseMove}
        className={cn(
          "relative p-6 rounded-2xl",
          "bg-card/80 border border-white/10",
          "hover:scale-[1.015] hover:-translate-y-1",
          "transition-all duration-300 shadow-[0_18px_36px_rgba(3,10,30,0.35)]",
          "backdrop-blur-sm",
          "h-full flex flex-col"
        )}
        style={
          { "--spot-x": "50%", "--spot-y": "50%" } as React.CSSProperties
        }
      >
        {/* Spotlight border glow -- tracks cursor */}
        <div className="absolute -inset-px rounded-2xl pointer-events-none opacity-0 group-hover:opacity-100 transition-opacity duration-300 [background:radial-gradient(500px_circle_at_var(--spot-x)_var(--spot-y),rgba(6,182,212,0.35),rgba(139,92,246,0.2)_40%,transparent_70%)]" />

        <div
          className={cn(
            "absolute inset-0 rounded-2xl opacity-0 group-hover:opacity-12",
            "bg-gradient-to-br transition-opacity duration-300",
            venture.color
          )}
        />
        <div className="absolute inset-0 rounded-2xl opacity-0 transition-opacity duration-300 group-hover:opacity-30 [background:linear-gradient(145deg,rgba(255,255,255,0.08),rgba(255,255,255,0.01)_42%)]" />
        <div className="absolute inset-0 rounded-2xl bg-[rgba(6,10,23,0.28)] group-hover:bg-[rgba(6,10,23,0.34)] transition-colors duration-300" />

        <div className="relative z-10 flex-1 flex flex-col">
          <div className="flex items-start justify-between mb-4">
            <h3 className="text-xl font-bold text-foreground">
              {venture.name}
            </h3>
            <span
              className={cn(
                "inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium",
                status.bg,
                status.text
              )}
            >
              <span className={cn("w-1.5 h-1.5 rounded-full animate-[glowPulse_3s_ease-in-out_infinite]", status.dot)} />
              {status.label}
            </span>
          </div>

          <p className="text-sm font-medium text-cyan mb-3 tracking-wide drop-shadow-[0_1px_1px_rgba(0,0,0,0.45)]">
            {venture.tagline}
          </p>

          <p className="text-sm text-foreground mb-4 flex-1">
            {venture.description}
          </p>

          <div className="flex flex-wrap gap-2">
            {venture.tags.map((tag) => (
              <span
                key={tag}
                className="text-xs px-2.5 py-1 rounded-md premium-chip text-foreground/80 font-medium"
              >
                {tag}
              </span>
            ))}
          </div>
        </div>
      </div>
    </div>
  );

  if (venture.href) {
    if (venture.href.startsWith("http")) {
      return (
        <a href={venture.href} target="_blank" rel="noopener noreferrer" className="block">
          {card}
        </a>
      );
    }

    return (
      <Link href={venture.href} className="block">
        {card}
      </Link>
    );
  }

  return card;
}

interface VentureGridProps {
  ventures: Venture[];
}

export function VentureGrid({ ventures }: VentureGridProps) {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-12 gap-6">
      {ventures.map((venture, index) => (
        <div
          key={venture.name}
          className={cn(
            "md:col-span-1 lg:col-span-4",
            index === 1 && "lg:col-span-5 lg:-mt-2",
            index === 2 && "lg:col-span-3"
          )}
        >
          <VentureCard venture={venture} index={index} />
        </div>
      ))}
    </div>
  );
}

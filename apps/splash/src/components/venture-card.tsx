"use client";

import { motion } from "framer-motion";
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

export function VentureCard({ venture, index }: VentureCardProps) {
  const status = statusConfig[venture.status];

  const card = (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      transition={{ duration: 0.6, delay: index * 0.1 }}
      whileHover={{ scale: 1.01, y: -2 }}
      className="group cursor-pointer"
    >
      <div
        className={cn(
          "relative p-6 rounded-xl",
          "bg-card/50 border border-border/50",
          "hover:border-primary/50",
          "transition-all duration-300",
          "backdrop-blur-sm",
          "h-full flex flex-col"
        )}
      >
        {/* Glow effect on hover */}
        <div
          className={cn(
            "absolute inset-0 rounded-xl opacity-0 group-hover:opacity-10",
            "bg-gradient-to-br transition-opacity duration-300",
            venture.color
          )}
        />

        <div className="relative z-10 flex-1 flex flex-col">
          {/* Header */}
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
              <span className={cn("w-1.5 h-1.5 rounded-full animate-pulse", status.dot)} />
              {status.label}
            </span>
          </div>

          {/* Tagline */}
          <p className="text-sm font-medium text-cyan mb-3">
            {venture.tagline}
          </p>

          {/* Description */}
          <p className="text-sm text-muted-foreground mb-4 flex-1">
            {venture.description}
          </p>

          {/* Tags */}
          <div className="flex flex-wrap gap-2">
            {venture.tags.map((tag) => (
              <span
                key={tag}
                className="text-xs px-2.5 py-1 rounded-md bg-muted/50 text-muted-foreground font-medium"
              >
                {tag}
              </span>
            ))}
          </div>
        </div>
      </div>
    </motion.div>
  );

  if (venture.href) {
    return (
      <a href={venture.href} target="_blank" rel="noopener noreferrer" className="block">
        {card}
      </a>
    );
  }

  return card;
}

interface VentureGridProps {
  ventures: Venture[];
}

export function VentureGrid({ ventures }: VentureGridProps) {
  return (
    <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
      {ventures.map((venture, index) => (
        <VentureCard key={venture.name} venture={venture} index={index} />
      ))}
    </div>
  );
}

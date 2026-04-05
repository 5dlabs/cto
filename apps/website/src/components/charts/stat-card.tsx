"use client";

import { type ReactNode } from "react";
import { motion } from "framer-motion";
import { AnimatedCounter } from "@/components/animated-counter";

interface StatCardProps {
  icon?: ReactNode;
  value: string;
  label: string;
  note?: string;
  accent?: boolean;
}

export function StatCard({ icon, value, label, note, accent }: StatCardProps) {
  return (
    <motion.article
      className={`rounded-xl p-5 backdrop-blur-sm ${
        accent
          ? "border border-cyan/30 bg-cyan/5"
          : "border border-border/50 bg-card/30"
      }`}
      initial={{ opacity: 0, y: 16 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: "-40px" }}
      transition={{ duration: 0.5, ease: [0.25, 0.4, 0, 1] }}
    >
      {icon && (
        <div className="w-10 h-10 rounded-lg bg-cyan/10 flex items-center justify-center text-cyan mb-3">
          {icon}
        </div>
      )}
      <AnimatedCounter
        value={value}
        className="text-3xl font-bold gradient-text block"
      />
      <p className="text-sm font-semibold mt-2">{label}</p>
      {note && (
        <p className="text-xs text-muted-foreground mt-1">{note}</p>
      )}
    </motion.article>
  );
}

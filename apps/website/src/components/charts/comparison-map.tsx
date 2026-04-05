"use client";

import { motion } from "framer-motion";

interface ComparisonRow {
  from: string;
  to: string;
  via?: string;
  status?: "live" | "next" | "planned";
}

const statusBadge: Record<string, { label: string; cls: string }> = {
  live: { label: "Live", cls: "text-emerald-400 bg-emerald-400/10 border-emerald-400/30" },
  next: { label: "Next", cls: "text-amber-400 bg-amber-400/10 border-amber-400/30" },
  planned: { label: "Planned", cls: "text-muted-foreground bg-muted-foreground/10 border-muted-foreground/30" },
};

interface ComparisonMapProps {
  rows: ComparisonRow[];
  callout?: string;
}

export function ComparisonMap({ rows, callout }: ComparisonMapProps) {
  return (
    <div>
      <div className="space-y-2.5">
        {rows.map((row, i) => (
          <motion.div
            key={`comparison-row-${i}`}
            className="grid grid-cols-[1fr_40px_1fr] items-center gap-2 rounded-lg border border-border/40 bg-card/20 px-4 py-3"
            initial={{ opacity: 0, x: -16 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true, margin: "-40px" }}
            transition={{
              duration: 0.5,
              delay: i * 0.08,
              ease: [0.25, 0.4, 0, 1],
            }}
          >
            <span className="text-sm text-muted-foreground line-through decoration-muted-foreground/40">
              {row.from}
            </span>

            {/* Animated arrow (screen) + static fallback (print) */}
            <motion.svg
              viewBox="0 0 40 16"
              className="w-10 h-4 text-cyan print:hidden"
              initial={{ pathLength: 0 }}
              whileInView={{ pathLength: 1 }}
              viewport={{ once: true }}
            >
              <motion.path
                d="M2 8 L30 8 M26 4 L32 8 L26 12"
                fill="none"
                stroke="currentColor"
                strokeWidth={2}
                strokeLinecap="round"
                strokeLinejoin="round"
                initial={{ pathLength: 0 }}
                whileInView={{ pathLength: 1 }}
                viewport={{ once: true }}
                transition={{
                  duration: 0.6,
                  delay: 0.3 + i * 0.08,
                  ease: [0.25, 0.4, 0, 1],
                }}
              />
            </motion.svg>
            <svg
              viewBox="0 0 40 16"
              className="w-10 h-4 text-cyan hidden print:block"
            >
              <path
                d="M2 8 L30 8 M26 4 L32 8 L26 12"
                fill="none"
                stroke="currentColor"
                strokeWidth={2}
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>

            <div className="flex items-center gap-2">
              <span className="text-sm font-semibold text-cyan">{row.to}</span>
              {row.status && statusBadge[row.status] && (
                <span className={`text-[10px] font-mono px-1.5 py-0.5 rounded border ${statusBadge[row.status].cls}`}>
                  {statusBadge[row.status].label}
                </span>
              )}
              {row.via && (
                <span className="text-[10px] text-muted-foreground/70 font-mono">
                  {row.via}
                </span>
              )}
            </div>
          </motion.div>
        ))}
      </div>

      {callout && (
        <motion.p
          className="mt-5 text-center text-sm font-semibold text-cyan"
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ delay: 0.6 }}
        >
          {callout}
        </motion.p>
      )}
    </div>
  );
}

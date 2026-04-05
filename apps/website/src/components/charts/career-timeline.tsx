"use client";

import { motion } from "framer-motion";

interface Milestone {
  company: string;
  role: string;
  highlight: string;
  current?: boolean;
}

interface CareerTimelineProps {
  milestones: Milestone[];
}

export function CareerTimeline({ milestones }: CareerTimelineProps) {
  return (
    <div className="relative pl-6">
      {/* Vertical line */}
      <motion.div
        className="absolute left-[7px] top-2 bottom-2 w-px bg-gradient-to-b from-cyan/60 via-cyan/30 to-transparent"
        initial={{ scaleY: 0 }}
        whileInView={{ scaleY: 1 }}
        viewport={{ once: true, margin: "-40px" }}
        transition={{ duration: 1, ease: [0.25, 0.4, 0, 1] }}
        style={{ transformOrigin: "top" }}
      />

      <div className="space-y-6">
        {milestones.map((m, i) => (
          <motion.div
            key={m.company}
            className="relative"
            initial={{ opacity: 0, x: -12 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true, margin: "-40px" }}
            transition={{
              duration: 0.5,
              delay: i * 0.12,
              ease: [0.25, 0.4, 0, 1],
            }}
          >
            {/* Dot */}
            <div
              className={`absolute -left-6 top-1.5 size-[15px] rounded-full border-2 ${
                m.current
                  ? "border-cyan bg-cyan/30 shadow-[0_0_12px_oklch(0.8_0.2_195/0.4)]"
                  : "border-border bg-background"
              }`}
            />

            <div>
              <p className="text-sm font-semibold">
                {m.company}
                {m.current && (
                  <span className="ml-2 text-[10px] uppercase tracking-widest text-cyan">
                    Now
                  </span>
                )}
              </p>
              <p className="text-xs text-muted-foreground">{m.role}</p>
              <p className="text-sm font-medium mt-1">{m.highlight}</p>
            </div>
          </motion.div>
        ))}
      </div>
    </div>
  );
}

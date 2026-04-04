"use client";

import { motion } from "framer-motion";

interface TimelinePoint {
  year: string;
  label: string;
  detail: string;
}

interface AccelerationTimelineProps {
  points: TimelinePoint[];
  callout?: string;
}

export function AccelerationTimeline({
  points,
  callout,
}: AccelerationTimelineProps) {
  return (
    <div>
      {/* Horizontal track */}
      <div className="relative">
        {/* Base line */}
        <motion.div
          className="absolute top-[18px] left-0 right-0 h-px bg-gradient-to-r from-muted-foreground/30 via-cyan/40 to-cyan/60"
          initial={{ scaleX: 0 }}
          whileInView={{ scaleX: 1 }}
          viewport={{ once: true, margin: "-40px" }}
          transition={{ duration: 1, ease: [0.25, 0.4, 0, 1] }}
          style={{ transformOrigin: "left" }}
        />

        <div className="grid gap-4" style={{ gridTemplateColumns: `repeat(${points.length}, 1fr)` }}>
          {points.map((point, i) => {
            const isLast = i === points.length - 1;

            return (
              <motion.div
                key={point.year}
                className="relative pt-10"
                initial={{ opacity: 0, y: 10 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true, margin: "-40px" }}
                transition={{
                  duration: 0.5,
                  delay: 0.2 + i * 0.15,
                  ease: [0.25, 0.4, 0, 1],
                }}
              >
                {/* Node dot */}
                <div
                  className={`absolute top-[11px] left-1/2 -translate-x-1/2 size-[15px] rounded-full border-2 ${
                    isLast
                      ? "border-cyan bg-cyan/30 shadow-[0_0_12px_oklch(0.8_0.2_195/0.4)]"
                      : "border-muted-foreground/40 bg-background"
                  }`}
                />

                <p
                  className={`text-xs font-mono font-bold uppercase tracking-wider ${
                    isLast ? "text-cyan" : "text-muted-foreground"
                  }`}
                >
                  {point.year}
                </p>
                <p className="text-sm font-semibold mt-1">{point.label}</p>
                <p className="text-xs text-muted-foreground mt-0.5">
                  {point.detail}
                </p>
              </motion.div>
            );
          })}
        </div>
      </div>

      {callout && (
        <motion.p
          className="mt-6 text-sm font-semibold text-cyan text-center"
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ delay: 0.8 }}
        >
          {callout}
        </motion.p>
      )}
    </div>
  );
}

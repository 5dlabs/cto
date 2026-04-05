"use client";

import { motion } from "framer-motion";

interface FunnelStage {
  label: string;
  description: string;
}

interface FunnelDiagramProps {
  stages: FunnelStage[];
}

export function FunnelDiagram({ stages }: FunnelDiagramProps) {
  const total = stages.length;

  return (
    <div className="space-y-2">
      {stages.map((stage, i) => {
        const widthPercent = 100 - (i / total) * 40;

        return (
          <motion.div
            key={stage.label}
            className="mx-auto rounded-lg border border-border/40 bg-card/20 px-5 py-3 backdrop-blur-sm"
            style={{ width: `${widthPercent}%` }}
            initial={{ opacity: 0, scaleX: 0.7 }}
            whileInView={{ opacity: 1, scaleX: 1 }}
            viewport={{ once: true, margin: "-40px" }}
            transition={{
              duration: 0.6,
              delay: i * 0.15,
              ease: [0.25, 0.4, 0, 1],
            }}
          >
            <p className="text-sm font-semibold">{stage.label}</p>
            <p className="text-xs text-muted-foreground mt-0.5">
              {stage.description}
            </p>
          </motion.div>
        );
      })}

      <motion.div
        className="flex justify-center pt-2"
        initial={{ opacity: 0 }}
        whileInView={{ opacity: 1 }}
        viewport={{ once: true }}
        transition={{ delay: stages.length * 0.15 + 0.2 }}
      >
        <svg viewBox="0 0 24 24" className="w-5 h-5 text-cyan" fill="none">
          <path
            d="M12 5v14M7 14l5 5 5-5"
            stroke="currentColor"
            strokeWidth={2}
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      </motion.div>
    </div>
  );
}

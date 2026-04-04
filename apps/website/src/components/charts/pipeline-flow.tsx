"use client";

import { motion } from "framer-motion";

interface PipelineNode {
  label: string;
  sublabel?: string;
  icon: React.ReactNode;
}

interface PipelineFlowProps {
  nodes: PipelineNode[];
}

export function PipelineFlow({ nodes }: PipelineFlowProps) {
  return (
    <div className="flex flex-col md:flex-row items-stretch md:items-center gap-3 md:gap-0">
      {nodes.map((node, i) => (
        <div
          key={node.label}
          className="flex flex-col md:flex-row items-center flex-1"
        >
          {/* Node */}
          <motion.div
            className="premium-shell rounded-xl px-4 py-4 backdrop-blur-sm flex flex-col items-center text-center w-full md:w-auto md:min-w-[120px]"
            initial={{ opacity: 0, y: 14 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: "-40px" }}
            transition={{
              duration: 0.5,
              delay: i * 0.12,
              ease: [0.25, 0.4, 0, 1],
            }}
          >
            <div className="w-10 h-10 rounded-lg bg-cyan/10 border border-cyan/20 flex items-center justify-center text-cyan mb-2">
              {node.icon}
            </div>
            <p className="text-sm font-semibold leading-tight">{node.label}</p>
            {node.sublabel && (
              <p className="text-[11px] text-muted-foreground mt-0.5">
                {node.sublabel}
              </p>
            )}
          </motion.div>

          {/* Connector arrow (not after last node) */}
          {i < nodes.length - 1 && (
            <motion.div
              className="flex items-center justify-center py-1 md:py-0 md:px-1"
              initial={{ opacity: 0 }}
              whileInView={{ opacity: 1 }}
              viewport={{ once: true }}
              transition={{ delay: 0.3 + i * 0.12 }}
            >
              {/* Vertical arrow on mobile */}
              <svg
                viewBox="0 0 24 24"
                className="w-5 h-5 text-cyan/50 md:hidden"
                fill="none"
              >
                <path
                  d="M12 5v14M7 14l5 5 5-5"
                  stroke="currentColor"
                  strokeWidth={2}
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
              </svg>
              {/* Horizontal arrow on desktop */}
              <svg
                viewBox="0 0 32 16"
                className="w-8 h-4 text-cyan/50 hidden md:block"
                fill="none"
              >
                <path
                  d="M2 8h22M20 4l6 4-6 4"
                  stroke="currentColor"
                  strokeWidth={2}
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
              </svg>
            </motion.div>
          )}
        </div>
      ))}
    </div>
  );
}

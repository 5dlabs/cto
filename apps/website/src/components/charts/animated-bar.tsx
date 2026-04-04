"use client";

import { motion } from "framer-motion";

interface BarItem {
  label: string;
  value: string;
  /** 0-100 proportional width */
  percent: number;
  colorVar?: string;
}

interface AnimatedBarChartProps {
  items: BarItem[];
  heading?: string;
  subheading?: string;
}

const barEase = [0.22, 1, 0.36, 1] as const;

export function AnimatedBarChart({
  items,
  heading,
  subheading,
}: AnimatedBarChartProps) {
  return (
    <div>
      {heading && (
        <p className="text-lg font-semibold mb-1">{heading}</p>
      )}
      {subheading && (
        <p className="text-sm text-muted-foreground mb-5">{subheading}</p>
      )}
      <div className="space-y-4">
        {items.map((item, i) => (
          <div key={item.label}>
            <div className="flex items-center justify-between text-sm mb-1.5">
              <span className="font-medium">{item.label}</span>
              <span className="font-mono text-muted-foreground tabular-nums">
                {item.value}
              </span>
            </div>
            <div className="h-3 w-full rounded-full bg-secondary/50 overflow-hidden">
              <motion.div
                className="h-full rounded-full"
                style={{
                  background: item.colorVar
                    ? `var(${item.colorVar})`
                    : "linear-gradient(90deg, var(--cyan), var(--chart-4))",
                }}
                initial={{ width: 0 }}
                whileInView={{ width: `${item.percent}%` }}
                viewport={{ once: true, margin: "-40px" }}
                transition={{
                  duration: 1.2,
                  delay: i * 0.1,
                  ease: barEase,
                }}
              />
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

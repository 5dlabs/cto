"use client";

import { motion } from "framer-motion";

interface Ring {
  label: string;
  value: string;
  description: string;
}

interface ConcentricRingsProps {
  rings: [Ring, Ring, Ring]; // outer, middle, inner
  source?: string;
}

export function ConcentricRings({ rings, source }: ConcentricRingsProps) {
  const [outer, middle, inner] = rings;

  return (
    <div className="flex flex-col items-center gap-6">
      <div className="relative w-[340px] h-[340px] sm:w-[400px] sm:h-[400px]">
        {/* Outer ring */}
        <motion.div
          className="absolute inset-0 rounded-full border-2 border-muted-foreground/20 flex items-center justify-center"
          style={{
            background:
              "radial-gradient(circle, transparent 60%, oklch(0.2 0.02 260 / 0.4) 100%)",
          }}
          initial={{ scale: 0.8, opacity: 0 }}
          whileInView={{ scale: 1, opacity: 1 }}
          viewport={{ once: true, margin: "-40px" }}
          transition={{ duration: 0.8, ease: [0.25, 0.4, 0, 1] }}
        >
          <span className="absolute top-4 left-1/2 -translate-x-1/2 text-center">
            <span className="block text-xs uppercase tracking-widest text-muted-foreground">
              {outer.label}
            </span>
            <span className="block text-lg font-bold font-mono tabular-nums text-muted-foreground">
              {outer.value}
            </span>
          </span>
        </motion.div>

        {/* Middle ring */}
        <motion.div
          className="absolute rounded-full border-2 border-chart-4/40 flex items-center justify-center"
          style={{
            top: "18%",
            left: "18%",
            right: "18%",
            bottom: "18%",
            background:
              "radial-gradient(circle, transparent 50%, oklch(0.3 0.1 280 / 0.15) 100%)",
          }}
          initial={{ scale: 0.8, opacity: 0 }}
          whileInView={{ scale: 1, opacity: 1 }}
          viewport={{ once: true, margin: "-40px" }}
          transition={{
            duration: 0.8,
            delay: 0.15,
            ease: [0.25, 0.4, 0, 1],
          }}
        >
          <span className="absolute top-3 left-1/2 -translate-x-1/2 text-center">
            <span className="block text-xs uppercase tracking-widest text-chart-4">
              {middle.label}
            </span>
            <span className="block text-lg font-bold font-mono tabular-nums text-chart-4">
              {middle.value}
            </span>
          </span>
        </motion.div>

        {/* Inner ring (beachhead - glowing cyan) */}
        <motion.div
          className="absolute rounded-full border-2 border-cyan/60 flex items-center justify-center"
          style={{
            top: "36%",
            left: "36%",
            right: "36%",
            bottom: "36%",
            background:
              "radial-gradient(circle, oklch(0.8 0.2 195 / 0.15) 0%, transparent 70%)",
            boxShadow: "0 0 40px oklch(0.8 0.2 195 / 0.2)",
          }}
          initial={{ scale: 0.8, opacity: 0 }}
          whileInView={{ scale: 1, opacity: 1 }}
          viewport={{ once: true, margin: "-40px" }}
          transition={{
            duration: 0.8,
            delay: 0.3,
            ease: [0.25, 0.4, 0, 1],
          }}
        >
          <span className="text-center">
            <span className="block text-xs uppercase tracking-widest text-cyan">
              {inner.label}
            </span>
            <span className="block text-xl font-bold font-mono tabular-nums text-cyan">
              {inner.value}
            </span>
          </span>
        </motion.div>
      </div>

      {/* Legend below */}
      <div className="grid grid-cols-3 gap-4 text-center max-w-md">
        {rings.map((ring, i) => (
          <div key={ring.label}>
            <p
              className={`text-xs font-medium ${
                i === 2
                  ? "text-cyan"
                  : i === 1
                    ? "text-chart-4"
                    : "text-muted-foreground"
              }`}
            >
              {ring.value}
            </p>
            <p className="text-xs text-muted-foreground mt-0.5">
              {ring.description}
            </p>
          </div>
        ))}
      </div>

      {source && (
        <p className="text-[10px] text-muted-foreground/60 mt-2">
          {source}
        </p>
      )}
    </div>
  );
}

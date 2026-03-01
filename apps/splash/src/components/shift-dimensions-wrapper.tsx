"use client";

import { useEffect } from "react";
import { motion, useAnimation } from "framer-motion";

const SHIFT_INTERVAL_MS = 5_000;
const SHIFT_DURATION_S = 2.5;

export function ShiftDimensionsWrapper({ children }: { children: React.ReactNode }) {
  const controls = useAnimation();

  useEffect(() => {
    const runShift = () => {
      controls.start({
        transform: [
          "perspective(1200px) rotateX(0deg) rotateY(0deg) scale(1) translateZ(0)",
          "perspective(1200px) rotateX(0.35deg) rotateY(0.25deg) scale(1.003) translateZ(12px)",
          "perspective(1200px) rotateX(0deg) rotateY(0deg) scale(1) translateZ(0)",
        ],
        transition: {
          duration: SHIFT_DURATION_S,
          ease: [0.25, 0.1, 0.25, 1],
        },
      });
    };
    const t = setInterval(runShift, SHIFT_INTERVAL_MS);
    runShift();
    return () => clearInterval(t);
  }, [controls]);

  return (
    <motion.div
      style={{ transformOrigin: "50% 50%", minHeight: "100%", display: "flex", flexDirection: "column", transformStyle: "flat" as const }}
      animate={controls}
      initial={{ transform: "perspective(1200px) rotateX(0deg) rotateY(0deg) scale(1) translateZ(0)" }}
    >
      {children}
    </motion.div>
  );
}

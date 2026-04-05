"use client";

import { useEffect, useRef } from "react";
import {
  useMotionValue,
  useTransform,
  motion,
  animate,
  useInView,
} from "framer-motion";

interface AnimatedCounterProps {
  value: string;
  className?: string;
}

export function AnimatedCounter({ value, className }: AnimatedCounterProps) {
  const ref = useRef<HTMLSpanElement>(null);
  const isInView = useInView(ref, { once: true, margin: "-40px" });

  const numericMatch = value.match(/^(\d+)/);
  const numericPart = numericMatch ? parseInt(numericMatch[1], 10) : 0;
  const suffix = numericMatch ? value.slice(numericMatch[1].length) : value;
  const isNumeric = numericMatch !== null;

  const motionVal = useMotionValue(0);
  const rounded = useTransform(motionVal, (v) => Math.round(v));

  useEffect(() => {
    if (!isInView || !isNumeric) return;

    const controls = animate(motionVal, numericPart, {
      duration: 2,
      ease: [0.16, 1, 0.3, 1],
    });

    return () => controls.stop();
  }, [isInView, isNumeric, numericPart, motionVal]);

  if (!isNumeric) {
    return (
      <span ref={ref} className={className}>
        {value}
      </span>
    );
  }

  return (
    <span ref={ref} className={className}>
      <motion.span className="font-mono tabular-nums print:hidden">{rounded}</motion.span>
      <span className="font-mono tabular-nums hidden print:inline">{numericPart}</span>
      {suffix}
    </span>
  );
}

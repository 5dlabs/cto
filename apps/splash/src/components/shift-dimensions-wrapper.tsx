"use client";

import { useEffect, useRef, useState } from "react";

const SHIFT_INTERVAL_MS = 5_000;
const SHIFT_DURATION_MS = 2500;

export function ShiftDimensionsWrapper({ children }: { children: React.ReactNode }) {
  const ref = useRef<HTMLDivElement>(null);
  const [ready, setReady] = useState(false);

  useEffect(() => {
    setReady(true);
    const el = ref.current;
    if (!el) return;

    let frame: number;
    let start: number | null = null;

    const applyShift = () => {
      if (!el) return;
      start = performance.now();

      const animate = (now: number) => {
        const elapsed = now - (start ?? now);
        const t = Math.min(elapsed / SHIFT_DURATION_MS, 1);
        const ease = t < 0.5
          ? 2 * t * t
          : 1 - (-2 * t + 2) ** 2 / 2;

        const progress = t <= 0.5 ? ease : 1 - (ease - 0.5) * 2;
        const rx = 0.35 * progress;
        const ry = 0.25 * progress;
        const s = 1 + 0.003 * progress;
        const tz = 12 * progress;

        el.style.transform = `perspective(1200px) rotateX(${rx}deg) rotateY(${ry}deg) scale(${s}) translateZ(${tz}px)`;

        if (t < 1) {
          frame = requestAnimationFrame(animate);
        }
      };

      frame = requestAnimationFrame(animate);
    };

    const interval = setInterval(applyShift, SHIFT_INTERVAL_MS);
    const timeout = setTimeout(applyShift, 100);

    return () => {
      clearInterval(interval);
      clearTimeout(timeout);
      cancelAnimationFrame(frame);
    };
  }, []);

  return (
    <div
      ref={ref}
      style={{
        transformOrigin: "50% 50%",
        minHeight: "100%",
        display: "flex",
        flexDirection: "column",
        transformStyle: "flat",
        willChange: ready ? "transform" : "auto",
      }}
    >
      {children}
    </div>
  );
}

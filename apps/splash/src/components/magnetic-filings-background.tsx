"use client";

import { useEffect, useMemo, Suspense, useState } from "react";
import { useSearchParams } from "next/navigation";
import { motion, useMotionValue, useTransform, useSpring } from "framer-motion";

const CELL_SIZE = 56;
const COLS = 20;
const ROWS = 13;

function Filing({
  col,
  row,
  mouseX,
  mouseY,
  variant,
}: {
  col: number;
  row: number;
  mouseX: ReturnType<typeof useMotionValue<number>>;
  mouseY: ReturnType<typeof useMotionValue<number>>;
  variant: "spring" | "plain";
}) {
  const centerX = col * CELL_SIZE + CELL_SIZE / 2;
  const centerY = row * CELL_SIZE + CELL_SIZE / 2;

  const angle = useTransform(() => {
    const mx = mouseX.get();
    const my = mouseY.get();
    return (Math.atan2(my - centerY, mx - centerX) * 180) / Math.PI;
  });

  const springAngle = useSpring(angle, { stiffness: 40, damping: 18 });

  return (
    <motion.div
      className="absolute flex items-center justify-center"
      style={{
        left: col * CELL_SIZE,
        top: row * CELL_SIZE,
        width: CELL_SIZE,
        height: CELL_SIZE,
        rotate: variant === "spring" ? springAngle : angle,
      }}
      initial={false}
    >
      {/* Short line segment = "filing" */}
      <div
        className="w-px h-3 rounded-full"
        style={{
          background:
            "linear-gradient(to top, transparent, oklch(0.6 0.14 195 / 0.35), transparent)",
        }}
      />
    </motion.div>
  );
}

export function MagneticFilingsBackground({
  variant = "spring",
}: {
  variant?: "spring" | "plain";
}) {
  const [prefersReducedMotion, setPrefersReducedMotion] = useState(false);
  const mouseX = useMotionValue(0);
  const mouseY = useMotionValue(0);

  const cells = useMemo(
    () =>
      Array.from({ length: COLS * ROWS }, (_, i) => ({
        col: i % COLS,
        row: Math.floor(i / COLS),
      })),
    [],
  );

  useEffect(() => {
    const mediaQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
    setPrefersReducedMotion(mediaQuery.matches);
    const onChange = (e: MediaQueryListEvent) => setPrefersReducedMotion(e.matches);
    mediaQuery.addEventListener("change", onChange);
    return () => mediaQuery.removeEventListener("change", onChange);
  }, []);

  useEffect(() => {
    const setInitial = () => {
      mouseX.set((COLS * CELL_SIZE) / 2);
      mouseY.set((ROWS * CELL_SIZE) / 2);
    };
    setInitial();
    const onMove = (e: MouseEvent) => {
      const containerOffsetX = window.innerWidth / 2 - (COLS * CELL_SIZE) / 2;
      const containerOffsetY = window.innerHeight / 2 - (ROWS * CELL_SIZE) / 2;
      mouseX.set(e.clientX - containerOffsetX);
      mouseY.set(e.clientY - containerOffsetY);
    };
    document.addEventListener("mousemove", onMove);
    return () => document.removeEventListener("mousemove", onMove);
  }, [mouseX, mouseY]);

  if (prefersReducedMotion) {
    return null;
  }

  return (
    <div
      className="fixed inset-0 pointer-events-none overflow-hidden z-[4]"
      style={{ contain: "layout style paint", isolation: "isolate" }}
      aria-hidden
    >
      <div
        className="absolute"
        style={{
          left: "50%",
          top: "50%",
          width: COLS * CELL_SIZE,
          height: ROWS * CELL_SIZE,
          transform: "translate(-50%, -50%)",
        }}
      >
        {cells.map(({ col, row }) => (
          <Filing
            key={`${col}-${row}`}
            col={col}
            row={row}
            mouseX={mouseX}
            mouseY={mouseY}
            variant={variant}
          />
        ))}
      </div>
    </div>
  );
}

/** In dev: use ?filings=spring or ?filings=plain to compare. Default is spring. */
function MagneticFilingsSwitchInner() {
  const searchParams = useSearchParams();
  const filings = searchParams.get("filings");
  const variant = filings === "plain" ? "plain" : "spring";
  return <MagneticFilingsBackground variant={variant} />;
}

export function MagneticFilingsBackgroundSwitch() {
  return (
    <Suspense fallback={null}>
      <MagneticFilingsSwitchInner />
    </Suspense>
  );
}

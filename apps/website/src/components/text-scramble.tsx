"use client";

import { useEffect, useRef, useState } from "react";

const GLYPHS = "!@#$%^&*()_+-=[]{}|;:,./<>?~`01";

interface TextScrambleProps {
  text: string;
  className?: string;
  duration?: number;
}

export function TextScramble({
  text,
  className,
  duration = 1400,
}: TextScrambleProps) {
  const [display, setDisplay] = useState(text);
  const hasRun = useRef(false);

  useEffect(() => {
    if (hasRun.current) return;
    hasRun.current = true;

    const chars = text.split("");
    const total = chars.length;
    const stepMs = duration / total;
    let resolved = 0;
    let frame: number;

    const scramble = () => {
      const out = chars.map((ch, i) => {
        if (i < resolved) return ch;
        if (ch === " ") return " ";
        return GLYPHS[Math.floor(Math.random() * GLYPHS.length)];
      });
      setDisplay(out.join(""));

      const now = performance.now();
      const elapsed = now - start;
      resolved = Math.min(total, Math.floor(elapsed / stepMs));

      if (resolved < total) {
        frame = requestAnimationFrame(scramble);
      } else {
        setDisplay(text);
      }
    };

    const start = performance.now();
    frame = requestAnimationFrame(scramble);

    return () => cancelAnimationFrame(frame);
  }, [text, duration]);

  return <span className={className}>{display}</span>;
}

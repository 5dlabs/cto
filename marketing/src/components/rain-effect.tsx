"use client";

import { useEffect, useState } from "react";

interface RainDrop {
  id: number;
  left: number;
  delay: number;
  duration: number;
  opacity: number;
}

export function RainEffect() {
  const [drops, setDrops] = useState<RainDrop[]>([]);

  useEffect(() => {
    const newDrops: RainDrop[] = [];
    for (let i = 0; i < 40; i++) {
      newDrops.push({
        id: i,
        left: Math.random() * 100,
        delay: Math.random() * 8,
        duration: 2 + Math.random() * 3,
        opacity: 0.05 + Math.random() * 0.15,
      });
    }
    setDrops(newDrops);
  }, []);

  return (
    <div className="fixed inset-0 pointer-events-none overflow-hidden z-0">
      {drops.map((drop) => (
        <div
          key={drop.id}
          className="absolute w-px bg-gradient-to-b from-transparent via-cyan to-transparent rain-drop"
          style={{
            left: `${drop.left}%`,
            height: "60px",
            animationDelay: `${drop.delay}s`,
            animationDuration: `${drop.duration}s`,
            opacity: drop.opacity,
          }}
        />
      ))}
    </div>
  );
}

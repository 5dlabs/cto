"use client";

interface RainDrop {
  id: number;
  left: number;
  delay: number;
  duration: number;
  opacity: number;
}

function seededValue(seed: number): number {
  const x = Math.sin(seed * 9301 + 49297) * 233280;
  return Math.round((x - Math.floor(x)) * 10000) / 10000;
}

const drops: RainDrop[] = Array.from({ length: 40 }, (_, i) => ({
  id: i,
  left: seededValue(i * 4) * 100,
  delay: seededValue(i * 4 + 1) * 8,
  duration: 2 + seededValue(i * 4 + 2) * 3,
  opacity: 0.05 + seededValue(i * 4 + 3) * 0.15,
}));

export function RainEffect() {
  return (
    <div className="fixed inset-0 pointer-events-none overflow-hidden z-0" suppressHydrationWarning>
      {drops.map((drop) => (
        <div
          key={drop.id}
          suppressHydrationWarning
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

"use client";

function seededValue(seed: number): number {
  const x = Math.sin(seed * 9301 + 49297) * 233280;
  return Math.round((x - Math.floor(x)) * 10000) / 10000;
}

export function GridPulse() {
  return (
    <div className="fixed inset-0 pointer-events-none overflow-hidden z-0">
      {/* Drifting grid */}
      <div
        className="absolute -inset-20"
        style={{
          backgroundImage: `
            linear-gradient(oklch(0.8 0.2 195 / 0.12) 1px, transparent 1px),
            linear-gradient(90deg, oklch(0.8 0.2 195 / 0.12) 1px, transparent 1px)
          `,
          backgroundSize: "60px 60px",
          animation: "grid-breathe 8s ease-in-out infinite, grid-drift 40s linear infinite",
        }}
      />
      {/* Wandering node glows — cyan/teal only */}
      {Array.from({ length: 10 }, (_, i) => (
        <div
          key={`node-${i}`}
          className="absolute rounded-full"
          style={{
            left: `${seededValue(i * 3) * 100}%`,
            top: `${seededValue(i * 3 + 1) * 100}%`,
            width: "400px",
            height: "400px",
            background: `radial-gradient(circle, oklch(0.8 0.2 195 / ${0.1 + seededValue(i * 3 + 2) * 0.1}), transparent 70%)`,
            animation: `grid-node-wander-${i % 4} ${18 + i * 3}s ease-in-out ${i * 1.5}s infinite alternate`,
          }}
        />
      ))}
      {/* Horizontal energy pulses */}
      {Array.from({ length: 5 }, (_, i) => (
        <div
          key={`hpulse-${i}`}
          className="absolute h-px left-0 right-0"
          style={{
            top: `${15 + i * 18}%`,
            background: `linear-gradient(90deg, transparent, oklch(0.8 0.2 195 / 0.4), transparent)`,
            animation: `grid-energy-h ${4 + seededValue(i * 7) * 3}s ease-in-out ${i * 2.5}s infinite`,
          }}
        />
      ))}
      {/* Vertical energy pulses */}
      {Array.from({ length: 4 }, (_, i) => (
        <div
          key={`vpulse-${i}`}
          className="absolute w-px top-0 bottom-0"
          style={{
            left: `${12 + i * 22}%`,
            background: `linear-gradient(180deg, transparent, oklch(0.8 0.2 195 / 0.35), transparent)`,
            animation: `grid-energy-v ${5 + seededValue(i * 9) * 3}s ease-in-out ${i * 3}s infinite`,
          }}
        />
      ))}
    </div>
  );
}

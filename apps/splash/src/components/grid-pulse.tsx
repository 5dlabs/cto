function seededValue(seed: number): number {
  const x = Math.sin(seed * 9301 + 49297) * 233280;
  return Math.round((x - Math.floor(x)) * 10000) / 10000;
}

/* Alternate between cyan (195) and purple (300) */
const hues = [195, 300] as const;

export function GridPulse() {
  return (
    <div
      className="fixed inset-0 pointer-events-none overflow-hidden z-[2]"
      style={{ contain: "layout style paint", transform: "translateZ(0)" }}
    >
      {/* Drifting grid — cyan horizontal, purple vertical */}
      <div
        className="absolute -inset-20"
        style={{
          backgroundImage: `
            linear-gradient(oklch(0.8 0.18 195 / 0.18) 1px, transparent 1px),
            linear-gradient(90deg, oklch(0.75 0.18 300 / 0.14) 1px, transparent 1px)
          `,
          backgroundSize: "60px 60px",
          willChange: "transform",
          animation: "grid-drift 30s linear infinite",
        }}
      />
      {/* Wandering node glows — vivid, transform-only animation */}
      {Array.from({ length: 6 }, (_, i) => {
        const hue = hues[i % 2];
        const opacity = 0.15 + seededValue(i * 3 + 2) * 0.15;
        return (
          <div
            key={`node-${i}`}
            className="absolute"
            style={{
              left: `${seededValue(i * 3) * 100}%`,
              top: `${seededValue(i * 3 + 1) * 100}%`,
              width: "400px",
              height: "400px",
              background: `radial-gradient(circle, oklch(0.75 0.22 ${hue} / ${opacity}), transparent 70%)`,
              willChange: "transform",
              animation: `grid-node-wander-${i % 4} ${12 + i * 2}s ease-in-out ${i * 1}s infinite alternate`,
            }}
          />
        );
      })}
      {/* Horizontal energy pulses */}
      {Array.from({ length: 5 }, (_, i) => {
        const hue = hues[i % 2];
        return (
          <div
            key={`hpulse-${i}`}
            className="absolute left-0 right-0"
            style={{
              top: `${15 + i * 18}%`,
              height: "2px",
              background: `linear-gradient(90deg, transparent, oklch(0.8 0.2 ${hue} / 0.5), transparent)`,
              willChange: "transform",
              animation: `grid-energy-h ${3 + seededValue(i * 7) * 2}s ease-in-out ${i * 1.5}s infinite`,
            }}
          />
        );
      })}
      {/* Vertical energy pulses */}
      {Array.from({ length: 4 }, (_, i) => {
        const hue = hues[(i + 1) % 2];
        return (
          <div
            key={`vpulse-${i}`}
            className="absolute top-0 bottom-0"
            style={{
              left: `${12 + i * 22}%`,
              width: "2px",
              background: `linear-gradient(180deg, transparent, oklch(0.8 0.2 ${hue} / 0.45), transparent)`,
              willChange: "transform",
              animation: `grid-energy-v ${3.5 + seededValue(i * 9) * 2}s ease-in-out ${i * 2}s infinite`,
            }}
          />
        );
      })}
    </div>
  );
}

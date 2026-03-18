function seededValue(seed: number): number {
  const x = Math.sin(seed * 9301 + 49297) * 233280;
  return Math.round((x - Math.floor(x)) * 10000) / 10000;
}

const hues = [195, 300] as const;

export function GridPulse() {
  return (
    <div
      className="fixed inset-0 pointer-events-none overflow-hidden z-0"
      style={{ contain: "layout style paint", isolation: "isolate" }}
    >
      {/* Drifting grid lines — subtle, stays behind text */}
      <div
        className="absolute -inset-20"
        style={{
          backgroundImage: `
            linear-gradient(oklch(0.65 0.14 195 / 0.12) 1px, transparent 1px),
            linear-gradient(90deg, oklch(0.6 0.14 300 / 0.09) 1px, transparent 1px)
          `,
          backgroundSize: "60px 60px",
          willChange: "transform",
          animation: "grid-drift 30s linear infinite",
        }}
      />
      {/* Wandering node glows — fewer, softer */}
      {Array.from({ length: 4 }, (_, i) => {
        const hue = hues[i % 2];
        const opacity = 0.06 + seededValue(i * 3 + 2) * 0.06;
        return (
          <div
            key={`node-${i}`}
            className="absolute"
            style={{
              left: `${seededValue(i * 3) * 100}%`,
              top: `${seededValue(i * 3 + 1) * 100}%`,
              width: "400px",
              height: "400px",
              background: `radial-gradient(circle, oklch(0.6 0.16 ${hue} / ${opacity}), transparent 70%)`,
              willChange: "transform",
              animation: `grid-node-wander-${i % 4} ${14 + i * 3}s ease-in-out ${i * 1.5}s infinite alternate`,
            }}
          />
        );
      })}
      {/* Horizontal energy pulses — fewer, thinner, softer */}
      {Array.from({ length: 3 }, (_, i) => {
        const hue = hues[i % 2];
        return (
          <div
            key={`hpulse-${i}`}
            className="absolute left-0 right-0"
            style={{
              top: `${20 + i * 25}%`,
              height: "1px",
              background: `linear-gradient(90deg, transparent, oklch(0.6 0.15 ${hue} / 0.18), transparent)`,
              willChange: "transform",
              animation: `grid-energy-h ${4 + seededValue(i * 7) * 3}s ease-in-out ${i * 2}s infinite`,
            }}
          />
        );
      })}
      {/* Vertical energy pulses — fewer, thinner, softer */}
      {Array.from({ length: 2 }, (_, i) => {
        const hue = hues[(i + 1) % 2];
        return (
          <div
            key={`vpulse-${i}`}
            className="absolute top-0 bottom-0"
            style={{
              left: `${25 + i * 35}%`,
              width: "1px",
              background: `linear-gradient(180deg, transparent, oklch(0.6 0.15 ${hue} / 0.15), transparent)`,
              willChange: "transform",
              animation: `grid-energy-v ${4.5 + seededValue(i * 9) * 3}s ease-in-out ${i * 2.5}s infinite`,
            }}
          />
        );
      })}
    </div>
  );
}

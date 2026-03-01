function seededValue(seed: number): number {
  const x = Math.sin(seed * 9301 + 49297) * 233280;
  return Math.round((x - Math.floor(x)) * 10000) / 10000;
}

/* Alternate between cyan (195) and purple (300) */
const hues = [195, 300] as const;

export function GridPulse() {
  return (
    <>
      <div
        className="fixed inset-0 pointer-events-none overflow-hidden z-[5]"
        style={{ contain: "layout style paint", isolation: "isolate", opacity: 0.55 }}
      >
        {/* Drifting grid — cyan horizontal, purple vertical */}
        <div
          className="absolute -inset-20"
          style={{
            backgroundImage: `
              linear-gradient(oklch(0.75 0.17 195 / 0.15) 1px, transparent 1px),
              linear-gradient(90deg, oklch(0.7 0.17 300 / 0.11) 1px, transparent 1px)
            `,
            backgroundSize: "60px 60px",
            willChange: "transform",
            animation: "grid-drift 30s linear infinite",
          }}
        />
        {/* Wandering node glows — vivid, transform-only animation */}
        {Array.from({ length: 6 }, (_, i) => {
          const hue = hues[i % 2];
          const opacity = 0.10 + seededValue(i * 3 + 2) * 0.10;
          return (
            <div
              key={`node-${i}`}
              className="absolute"
              style={{
                left: `${seededValue(i * 3) * 100}%`,
                top: `${seededValue(i * 3 + 1) * 100}%`,
                width: "400px",
                height: "400px",
                background: `radial-gradient(circle, oklch(0.70 0.20 ${hue} / ${opacity}), transparent 70%)`,
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
                height: "1px",
                background: `linear-gradient(90deg, transparent, oklch(0.75 0.19 ${hue} / 0.25), transparent)`,
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
                width: "1px",
                background: `linear-gradient(180deg, transparent, oklch(0.75 0.19 ${hue} / 0.22), transparent)`,
                willChange: "transform",
                animation: `grid-energy-v ${3.5 + seededValue(i * 9) * 2}s ease-in-out ${i * 2}s infinite`,
              }}
            />
          );
        })}
      </div>

      {/* Rare foreground shimmer — sits BELOW main content (z-10) so it never hides tiles. */}
      <div
        className="fixed inset-0 pointer-events-none overflow-hidden z-[9]"
        style={{
          contain: "layout style paint",
          opacity: 0,
          animation: "grid-foreground-rare 45s linear infinite",
        }}
      >
        <div
          className="absolute left-0 right-0"
          style={{
            top: "34%",
            height: "1px",
            background: "linear-gradient(90deg, transparent, oklch(0.7 0.17 195 / 0.25), transparent)",
            willChange: "transform",
            animation: "grid-energy-h 8.5s ease-in-out 0.5s infinite",
          }}
        />
        <div
          className="absolute top-0 bottom-0"
          style={{
            left: "62%",
            width: "1px",
            background: "linear-gradient(180deg, transparent, oklch(0.65 0.17 300 / 0.2), transparent)",
            willChange: "transform",
            animation: "grid-energy-v 9.25s ease-in-out 1.2s infinite",
          }}
        />
      </div>
    </>
  );
}

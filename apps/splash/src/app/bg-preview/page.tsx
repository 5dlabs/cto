"use client";

import { useState, useEffect, useRef } from "react";
import { motion, useAnimation } from "framer-motion";

function seededValue(seed: number): number {
  const x = Math.sin(seed * 9301 + 49297) * 233280;
  return Math.round((x - Math.floor(x)) * 10000) / 10000;
}

function OriginalRain() {
  const drops = Array.from({ length: 40 }, (_, i) => ({
    id: i,
    left: seededValue(i * 4) * 100,
    delay: seededValue(i * 4 + 1) * 8,
    duration: 2 + seededValue(i * 4 + 2) * 3,
    opacity: 0.05 + seededValue(i * 4 + 3) * 0.15,
  }));
  return (
    <>
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
    </>
  );
}

function AuroraEffect() {
  return (
    <>
      <div
        className="absolute w-[200%] h-[200%] -top-1/2 -left-1/2"
        style={{
          background: `
            radial-gradient(ellipse 60% 40% at 20% 50%, oklch(0.8 0.2 195 / 0.35), transparent),
            radial-gradient(ellipse 50% 60% at 80% 30%, oklch(0.7 0.25 320 / 0.30), transparent),
            radial-gradient(ellipse 70% 30% at 50% 80%, oklch(0.8 0.2 195 / 0.25), transparent)
          `,
          animation: "aurora 25s ease-in-out infinite alternate",
        }}
      />
      <div
        className="absolute w-[200%] h-[200%] -top-1/2 -left-1/2"
        style={{
          background: `
            radial-gradient(ellipse 40% 50% at 70% 60%, oklch(0.7 0.25 320 / 0.25), transparent),
            radial-gradient(ellipse 60% 40% at 30% 20%, oklch(0.8 0.2 195 / 0.30), transparent)
          `,
          animation: "aurora-reverse 30s ease-in-out infinite alternate",
        }}
      />
    </>
  );
}

function FloatingParticles() {
  const particles = Array.from({ length: 50 }, (_, i) => ({
    id: i,
    x: seededValue(i * 5) * 100,
    y: seededValue(i * 5 + 1) * 100,
    size: 3 + seededValue(i * 5 + 2) * 5,
    duration: 10 + seededValue(i * 5 + 3) * 15,
    delay: seededValue(i * 5 + 4) * 6,
    opacity: 0.3 + seededValue(i * 5 + 2) * 0.7,
  }));
  return (
    <>
      {particles.map((p) => (
        <div
          key={p.id}
          className="absolute rounded-full"
          style={{
            left: `${p.x}%`,
            top: `${p.y}%`,
            width: `${p.size}px`,
            height: `${p.size}px`,
            background: p.id % 3 === 0
              ? "oklch(0.7 0.25 320)"
              : "oklch(0.8 0.2 195)",
            boxShadow: p.id % 3 === 0
              ? "0 0 15px 5px oklch(0.7 0.25 320 / 0.6)"
              : "0 0 15px 5px oklch(0.8 0.2 195 / 0.6)",
            opacity: p.opacity,
            animation: `particle-float ${p.duration}s ease-in-out ${p.delay}s infinite alternate`,
          }}
        />
      ))}
    </>
  );
}

const hues = [195, 300] as const;

function GridPulse() {
  return (
    <>
      {/* Drifting grid — subtle cyan/purple blend */}
      <div
        className="absolute -inset-20"
        style={{
          backgroundImage: `
            linear-gradient(oklch(0.8 0.15 195 / 0.10) 1px, transparent 1px),
            linear-gradient(90deg, oklch(0.75 0.15 300 / 0.08) 1px, transparent 1px)
          `,
          backgroundSize: "60px 60px",
          animation: "grid-drift 30s linear infinite",
          opacity: 0.8,
        }}
      />
      {/* Wandering node glows — transform-only, no repaints */}
      {Array.from({ length: 6 }, (_, i) => {
        const hue = hues[i % 2];
        const opacity = 0.06 + seededValue(i * 3 + 2) * 0.08;
        return (
          <div
            key={`node-${i}`}
            className="absolute"
            style={{
              left: `${seededValue(i * 3) * 100}%`,
              top: `${seededValue(i * 3 + 1) * 100}%`,
              width: "350px",
              height: "350px",
              background: `radial-gradient(circle, oklch(0.75 0.2 ${hue} / ${opacity}), transparent 70%)`,
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
            className="absolute h-px left-0 right-0"
            style={{
              top: `${15 + i * 18}%`,
              background: `linear-gradient(90deg, transparent, oklch(0.8 0.2 ${hue} / 0.4), transparent)`,
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
            className="absolute w-px top-0 bottom-0"
            style={{
              left: `${12 + i * 22}%`,
              background: `linear-gradient(180deg, transparent, oklch(0.8 0.2 ${hue} / 0.35), transparent)`,
              animation: `grid-energy-v ${3.5 + seededValue(i * 9) * 2}s ease-in-out ${i * 2}s infinite`,
            }}
          />
        );
      })}
    </>
  );
}

function MatrixScanlines() {
  return (
    <>
      <div
        className="absolute inset-0"
        style={{
          backgroundImage: `repeating-linear-gradient(
            0deg,
            transparent,
            transparent 2px,
            oklch(0.8 0.2 195 / 0.06) 2px,
            oklch(0.8 0.2 195 / 0.06) 4px
          )`,
        }}
      />
      <div
        className="absolute left-0 right-0 h-[300px]"
        style={{
          background: `linear-gradient(
            180deg,
            transparent,
            oklch(0.8 0.2 195 / 0.15) 40%,
            oklch(0.7 0.25 320 / 0.10) 60%,
            transparent
          )`,
          animation: "scanline-sweep 8s linear infinite",
        }}
      />
      {Array.from({ length: 12 }, (_, i) => (
        <div
          key={i}
          className="absolute"
          style={{
            left: 0,
            right: 0,
            top: `${seededValue(i * 2) * 100}%`,
            height: "2px",
            background: `linear-gradient(90deg, transparent 10%, oklch(0.8 0.2 195 / ${0.3 + seededValue(i * 2 + 1) * 0.3}) 50%, transparent 90%)`,
            animation: `trace-line ${3 + seededValue(i * 2 + 1) * 4}s ease-in-out ${i * 0.8}s infinite`,
          }}
        />
      ))}
    </>
  );
}

function CanvasNebula() {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    let frame: number;
    const resize = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
    };
    resize();
    window.addEventListener("resize", resize);

    const blobs = Array.from({ length: 6 }, (_, i) => ({
      x: seededValue(i * 6) * canvas.width,
      y: seededValue(i * 6 + 1) * canvas.height,
      r: 180 + seededValue(i * 6 + 2) * 220,
      vx: (seededValue(i * 6 + 3) - 0.5) * 0.4,
      vy: (seededValue(i * 6 + 4) - 0.5) * 0.4,
      hue: i % 2 === 0 ? 195 : 320,
    }));

    function draw() {
      ctx!.clearRect(0, 0, canvas!.width, canvas!.height);
      for (const b of blobs) {
        b.x += b.vx;
        b.y += b.vy;
        if (b.x < -b.r || b.x > canvas!.width + b.r) b.vx *= -1;
        if (b.y < -b.r || b.y > canvas!.height + b.r) b.vy *= -1;

        const grad = ctx!.createRadialGradient(b.x, b.y, 0, b.x, b.y, b.r);
        if (b.hue === 195) {
          grad.addColorStop(0, "rgba(6, 182, 212, 0.22)");
          grad.addColorStop(1, "rgba(6, 182, 212, 0)");
        } else {
          grad.addColorStop(0, "rgba(190, 50, 150, 0.18)");
          grad.addColorStop(1, "rgba(190, 50, 150, 0)");
        }
        ctx!.fillStyle = grad;
        ctx!.beginPath();
        ctx!.arc(b.x, b.y, b.r, 0, Math.PI * 2);
        ctx!.fill();
      }
      frame = requestAnimationFrame(draw);
    }
    draw();
    return () => {
      cancelAnimationFrame(frame);
      window.removeEventListener("resize", resize);
    };
  }, []);

  return <canvas ref={canvasRef} className="absolute inset-0" />;
}

const backgrounds = [
  { id: "rain", label: "Digital Rain", desc: "Current production effect — falling cyan streaks", Component: OriginalRain },
  { id: "aurora", label: "Aurora Drift", desc: "Slow color clouds — ambient, zero fatigue", Component: AuroraEffect },
  { id: "particles", label: "Floating Particles", desc: "Sparse glowing dots — calm but alive", Component: FloatingParticles },
  { id: "grid", label: "Grid Pulse", desc: "Breathing wireframe + cyan/purple node glows — techy", Component: GridPulse },
  { id: "scanlines", label: "Scanline Traces", desc: "CRT sweep + traces — cypherpunk classic", Component: MatrixScanlines },
  { id: "nebula", label: "Canvas Nebula", desc: "Drifting color blobs — organic, living", Component: CanvasNebula },
];

const SHIFT_INTERVAL_MS = 5_000;
const SHIFT_DURATION_S = 2.5;

export default function BgPreviewPage() {
  const [active, setActive] = useState("rain");
  const bg = backgrounds.find((b) => b.id === active)!;
  const shiftControls = useAnimation();

  useEffect(() => {
    const runShift = () => {
      shiftControls.start({
        transform: [
          "perspective(1200px) rotateX(0deg) rotateY(0deg) scale(1) translateZ(0)",
          "perspective(1200px) rotateX(0.35deg) rotateY(0.25deg) scale(1.003) translateZ(12px)",
          "perspective(1200px) rotateX(0deg) rotateY(0deg) scale(1) translateZ(0)",
        ],
        transition: {
          duration: SHIFT_DURATION_S,
          ease: [0.25, 0.1, 0.25, 1],
        },
      });
    };
    const t = setInterval(runShift, SHIFT_INTERVAL_MS);
    runShift();
    return () => clearInterval(t);
  }, [shiftControls]);

  return (
    <motion.div
      className="relative min-h-screen overflow-hidden bg-[#030712]"
      style={{ transformOrigin: "50% 50%" }}
      animate={shiftControls}
      initial={{ transform: "perspective(1200px) rotateX(0deg) rotateY(0deg) scale(1) translateZ(0)" }}
    >
      {/* Base gradient */}
      <div className="fixed inset-0 bg-gradient-to-b from-[#030712] via-[#030712] to-[oklch(0.06_0.03_260)] z-0" />

      {/* Circuit grid */}
      <div className="fixed inset-0 circuit-bg z-[1]" />

      {/* Swappable background effect */}
      <div className="fixed inset-0 z-[2] pointer-events-none overflow-hidden">
        {active === "rain" && <OriginalRain />}
        {active === "aurora" && <AuroraEffect />}
        {active === "particles" && <FloatingParticles />}
        {active === "grid" && <GridPulse />}
        {active === "scanlines" && <MatrixScanlines />}
        {active === "nebula" && <CanvasNebula />}
      </div>

      {/* Noise */}
      <div className="fixed inset-0 noise-overlay z-[3]" />

      {/* Selector — pinned to top */}
      <div className="fixed top-0 left-0 right-0 z-50 p-4">
        <div className="max-w-4xl mx-auto p-4 rounded-2xl border border-white/10 bg-black/80 backdrop-blur-xl">
          <div className="flex items-center gap-3 mb-3">
            <h2 className="text-sm font-bold text-white/90 uppercase tracking-wider">Background Variant</h2>
            <span className="text-xs text-cyan px-2 py-0.5 rounded-full border border-cyan/30 bg-cyan/10">{bg.label}</span>
          </div>
          <div className="grid grid-cols-3 sm:grid-cols-6 gap-2">
            {backgrounds.map((b) => (
              <button
                key={b.id}
                onClick={() => setActive(b.id)}
                className={`px-3 py-2.5 rounded-lg text-left transition-all ${
                  active === b.id
                    ? "border border-cyan bg-cyan/15 shadow-lg shadow-cyan/20"
                    : "border border-white/10 bg-white/5 hover:border-cyan/40 hover:bg-cyan/5"
                }`}
              >
                <p className={`text-xs font-semibold leading-tight ${active === b.id ? "text-cyan" : "text-white/80"}`}>
                  {b.label}
                </p>
              </button>
            ))}
          </div>
          <p className="text-xs text-white/40 mt-2">{bg.desc}</p>
        </div>
      </div>

      {/* Sample page content */}
      <main className="relative z-10 min-h-screen flex flex-col items-center justify-center px-6 pt-36 pb-20">
        <div className="text-center mb-16">
          <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full border border-cyan/30 bg-cyan/5 mb-8">
            <span className="w-2 h-2 rounded-full bg-cyan animate-[pulse_3s_ease-in-out_infinite]" />
            <span className="text-sm text-cyan font-medium">From PRD to Production — Autonomously</span>
          </div>

          <h1 className="text-5xl sm:text-6xl md:text-7xl font-bold tracking-tight mb-6">
            <span className="gradient-text glow-text-cyan">Your Engineering Team</span>
            <br />
            <span className="text-white">Lives Here</span>
          </h1>

          <p className="text-xl text-white/50 max-w-2xl mx-auto mb-10">
            Thirteen specialized AI agents that ship complete features. From requirements to deployed code — automatically.
          </p>

          <div className="flex flex-wrap justify-center gap-8 text-sm text-white/50">
            <div className="flex items-center gap-2">
              <span className="text-2xl font-bold text-white">60-80%</span>
              <span>cost savings vs cloud</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-2xl font-bold text-white">13</span>
              <span>specialized agents</span>
            </div>
          </div>
        </div>

        {/* Sample cards to see how bg works with content */}
        <div className="grid md:grid-cols-3 gap-4 max-w-4xl w-full">
          {["Rex — Rust Architect", "Blaze — Web Developer", "Bolt — Infrastructure & SRE"].map((title) => (
            <div key={title} className="p-6 rounded-xl border border-white/10 bg-white/5 backdrop-blur-sm">
              <h3 className="text-sm font-semibold text-white mb-2">{title}</h3>
              <p className="text-xs text-white/40">Specialized AI agent shipping production code autonomously.</p>
            </div>
          ))}
        </div>
      </main>
    </motion.div>
  );
}

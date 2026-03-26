"use client";

export function MeshGradientBg() {
  return (
    <div className="fixed inset-0 z-[1] overflow-hidden pointer-events-none">
      <div
        className="absolute w-[130vw] h-[130vh] -top-[15vh] -left-[15vw] animate-[meshDrift1_25s_ease-in-out_infinite_alternate]"
        style={{
          background:
            "radial-gradient(ellipse 60% 50% at 20% 30%, rgba(99,60,255,0.18), transparent 70%)",
        }}
      />
      <div
        className="absolute w-[120vw] h-[120vh] -top-[10vh] -right-[10vw] animate-[meshDrift2_30s_ease-in-out_infinite_alternate]"
        style={{
          background:
            "radial-gradient(ellipse 55% 45% at 75% 60%, rgba(6,182,212,0.14), transparent 65%)",
        }}
      />
      <div
        className="absolute w-[100vw] h-[100vh] top-[20vh] left-[10vw] animate-[meshDrift3_35s_ease-in-out_infinite_alternate]"
        style={{
          background:
            "radial-gradient(ellipse 50% 55% at 45% 50%, rgba(139,92,246,0.16), transparent 60%)",
        }}
      />
      <div
        className="absolute w-[90vw] h-[90vh] bottom-0 right-0 animate-[meshDrift4_20s_ease-in-out_infinite_alternate]"
        style={{
          background:
            "radial-gradient(ellipse 45% 40% at 80% 80%, rgba(59,130,246,0.12), transparent 65%)",
        }}
      />
      <div
        className="absolute w-[110vw] h-[110vh] top-[40vh] -left-[5vw] animate-[meshDrift5_28s_ease-in-out_infinite_alternate]"
        style={{
          background:
            "radial-gradient(ellipse 40% 60% at 30% 70%, rgba(168,85,247,0.1), transparent 60%)",
        }}
      />
    </div>
  );
}

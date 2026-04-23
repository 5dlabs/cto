import Image from "next/image";

export type DeterministicAvatarProps = {
  voiceState: "idle" | "connecting" | "listening" | "speaking" | "error";
  latestUserText?: string;
  latestAgentText?: string;
  compact?: boolean;
};

function VoicePulse({ active, color }: { active: boolean; color: string }) {
  return (
    <span
      className="w-1.5 rounded-full transition-all duration-200"
      style={{
        height: `${active ? 14 : 8}px`,
        background: active ? color : "rgba(255,255,255,0.12)",
        opacity: active ? 1 : 0.45,
      }}
    />
  );
}

export default function DeterministicAvatar({
  voiceState,
  latestUserText,
  latestAgentText,
  compact = false,
}: DeterministicAvatarProps) {
  const speaking = voiceState === "speaking";
  const listening = voiceState === "listening";
  const connecting = voiceState === "connecting";
  const error = voiceState === "error";

  const ringClass = error
    ? "border-rose-400/35 shadow-[0_0_80px_-24px_rgba(251,113,133,0.55)]"
    : speaking
      ? "border-fuchsia-400/35 shadow-[0_0_90px_-20px_rgba(217,70,239,0.55)]"
      : listening
        ? "border-emerald-400/35 shadow-[0_0_90px_-20px_rgba(16,185,129,0.45)]"
        : "border-cyan-400/25 shadow-[0_0_80px_-24px_rgba(34,211,238,0.35)]";

  const panelCopy = error
    ? "Session error"
    : speaking
      ? latestAgentText || "Morgan is speaking."
      : listening
        ? latestUserText || "Morgan is listening for your next turn."
        : connecting
          ? "Bringing Morgan online."
          : "Deterministic local avatar fallback is ready.";

  return (
    <div className="relative h-full w-full overflow-hidden bg-[radial-gradient(circle_at_top,#164e63_0%,#020617_42%,#020617_100%)]">
      <div className="absolute inset-0 bg-[linear-gradient(180deg,rgba(8,47,73,0.16),rgba(2,6,23,0.86))]" />
      <div className="absolute inset-0 opacity-40 [background-image:linear-gradient(rgba(148,163,184,0.08)_1px,transparent_1px),linear-gradient(90deg,rgba(148,163,184,0.08)_1px,transparent_1px)] [background-size:28px_28px]" />

      <div className="relative flex h-full flex-col items-center justify-between px-6 py-6 sm:px-8 sm:py-8">
        <div className="flex w-full items-center justify-between gap-3 text-[11px] uppercase tracking-[0.28em] text-slate-300/80">
          <span>Fallback avatar</span>
          <span>{voiceState}</span>
        </div>

        <div className="relative flex flex-1 items-center justify-center py-6">
          <div className={`absolute h-[72%] w-[72%] rounded-full border ${ringClass} transition-all duration-300`} />
          <div className="absolute h-[82%] w-[82%] rounded-full bg-cyan-400/8 blur-3xl" />

          <div
            className="relative flex w-full max-w-[420px] flex-col items-center"
            style={{ transform: `translateY(${speaking ? "-4px" : listening ? "-2px" : "0px"}) scale(${speaking ? 1.01 : 1})` }}
          >
            <div className="relative overflow-hidden rounded-[2.5rem] border border-white/10 bg-black/25 p-2 backdrop-blur-sm">
              <Image
                src="/morgan.jpg"
                alt="Morgan avatar concept"
                width={768}
                height={1024}
                priority
                className={`h-auto w-full max-w-[340px] rounded-[2rem] object-cover transition duration-300 ${
                  speaking ? "scale-[1.02] saturate-110" : listening ? "scale-[1.01] saturate-105" : "scale-100"
                } ${error ? "grayscale-[0.2]" : ""}`}
              />

              <div className="pointer-events-none absolute inset-x-[18%] bottom-[13%] flex h-8 items-end justify-center gap-1.5">
                {[0, 1, 2, 3, 4, 5].map((idx) => {
                  const active = speaking ? idx % 2 === 0 || idx === 3 : listening ? idx === 2 || idx === 3 : false;
                  return (
                    <VoicePulse
                      key={idx}
                      active={active}
                      color={speaking ? "rgba(244,114,182,0.95)" : "rgba(52,211,153,0.9)"}
                    />
                  );
                })}
              </div>
            </div>

            <div className="mt-5 flex items-center gap-2 rounded-full border border-white/10 bg-slate-950/70 px-4 py-2 text-xs text-slate-200 backdrop-blur-md">
              <span className={`h-2.5 w-2.5 rounded-full ${error ? "bg-rose-300" : speaking ? "bg-fuchsia-300" : listening ? "bg-emerald-300" : "bg-cyan-300"}`} />
              <span>{panelCopy}</span>
            </div>
          </div>
        </div>

        {!compact ? (
          <div className="grid w-full gap-3 text-sm text-slate-200 sm:grid-cols-2">
            <div className="rounded-[1.4rem] border border-white/10 bg-slate-950/65 p-4 backdrop-blur-md">
              <p className="text-[11px] uppercase tracking-[0.28em] text-emerald-200/85">Heard you</p>
              <p className="mt-3 leading-6 text-slate-100">{latestUserText || "Waiting for microphone activity."}</p>
            </div>
            <div className="rounded-[1.4rem] border border-white/10 bg-slate-950/65 p-4 backdrop-blur-md">
              <p className="text-[11px] uppercase tracking-[0.28em] text-fuchsia-200/85">Morgan said</p>
              <p className="mt-3 leading-6 text-slate-100">{latestAgentText || "Reply text will appear here as soon as the agent speaks."}</p>
            </div>
          </div>
        ) : null}
      </div>
    </div>
  );
}

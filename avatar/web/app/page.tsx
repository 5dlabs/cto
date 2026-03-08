import Room from "@/components/Room";

export default function Home() {
  return (
    <main className="min-h-screen bg-[radial-gradient(circle_at_top,#164e63_0%,#020617_35%,#020617_100%)] px-6 py-10 text-white sm:px-8 lg:px-12">
      <div className="mx-auto flex max-w-7xl flex-col gap-8">
        <header className="flex flex-col gap-4">
          <p className="text-xs uppercase tracking-[0.3em] text-cyan-300/80">
            Proof of Concept
          </p>
          <div className="max-w-4xl">
            <h1 className="text-4xl font-semibold tracking-tight sm:text-5xl">
              Morgan as a low-latency talking avatar.
            </h1>
            <p className="mt-4 max-w-3xl text-base leading-7 text-slate-300 sm:text-lg">
              This client is built to validate the full voice loop first: browser microphone,
              streaming agent reply, LemonSlice avatar video, and the room-level timings needed
              for iterative latency tuning.
            </p>
          </div>
        </header>

        <Room />
      </div>
    </main>
  );
}

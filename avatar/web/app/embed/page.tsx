import Room from "@/components/Room";

export default function EmbedPage() {
  return (
    <main className="min-h-screen bg-[radial-gradient(circle_at_top,#164e63_0%,#020617_36%,#020617_100%)] p-4 text-white sm:p-6">
      <div className="mx-auto max-w-[1100px]">
        <Room compact autoConnect />
      </div>
    </main>
  );
}

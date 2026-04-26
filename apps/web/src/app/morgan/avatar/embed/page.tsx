import type { Metadata } from 'next';
import { MorganAvatarRoom } from '@/components/morgan-avatar/morgan-avatar-room';

export const metadata: Metadata = {
  title: 'Morgan LiveKit Avatar Embed',
};

export default function MorganAvatarEmbedPage() {
  return (
    <main className="h-screen overflow-hidden bg-[radial-gradient(circle_at_top,#164e63_0%,#020617_36%,#020617_100%)] text-white">
      <MorganAvatarRoom autoConnect embedded />
    </main>
  );
}

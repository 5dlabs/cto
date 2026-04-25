import type { Metadata } from 'next';
import { MorganAvatarRoom } from '@/components/morgan-avatar/morgan-avatar-room';

export const metadata: Metadata = {
  title: 'Morgan LiveKit Avatar',
  description: 'Self-hosted LiveKit route for talking with Morgan.',
};

export default function MorganAvatarPage() {
  return <MorganAvatarRoom />;
}

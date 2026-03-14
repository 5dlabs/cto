export type AgentBranding = {
  avatar?: string
  accent: string
}

export const AGENT_BRANDING: Record<string, AgentBranding> = {
  morgan: {
    avatar: '/agents/morgan-avatar-512.png',
    accent: 'from-cyan-400 to-pink-500',
  },
  atlas: {
    avatar: '/agents/atlas-avatar-512.png',
    accent: 'from-slate-500 to-zinc-500',
  },
  stitch: {
    avatar: '/agents/stitch-avatar-512.png',
    accent: 'from-orange-500 to-blue-400',
  },
  rex: {
    avatar: '/agents/rex-avatar-512.png',
    accent: 'from-orange-500 to-red-500',
  },
  blaze: {
    avatar: '/agents/blaze-avatar-512.png',
    accent: 'from-blue-500 to-cyan-500',
  },
  grizz: {
    avatar: '/agents/grizz-avatar-512.png',
    accent: 'from-amber-500 to-orange-400',
  },
  tess: {
    avatar: '/agents/tess-avatar-512.png',
    accent: 'from-violet-500 to-purple-500',
  },
  cleo: {
    avatar: '/agents/cleo-avatar-512.png',
    accent: 'from-emerald-500 to-teal-500',
  },
  cipher: {
    avatar: '/agents/cipher-avatar-512.png',
    accent: 'from-red-500 to-rose-500',
  },
  bolt: {
    avatar: '/agents/bolt-avatar-512.png',
    accent: 'from-yellow-500 to-amber-500',
  },
  angie: {
    avatar: '/agents/angie-avatar-512.png',
    accent: 'from-indigo-500 to-cyan-400',
  },
  healer: {
    accent: 'from-emerald-400 to-cyan-400',
  },
  keeper: {
    accent: 'from-zinc-400 to-slate-400',
  },
}

export function getAgentBranding(agentId: string): AgentBranding {
  return AGENT_BRANDING[agentId] ?? {
    accent: 'from-cyan-400 to-blue-500',
  }
}

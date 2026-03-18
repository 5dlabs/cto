import { ExternalLink, RefreshCw } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import type { MorganAvatarState } from '@/components/MorganAvatarRoom'
import type { LocalMorganHealth, MorganDiagnostics } from '@/lib/tauri'

interface MorganDebugPanelProps {
  health: LocalMorganHealth | null
  diagnostics: MorganDiagnostics | null
  avatarState: MorganAvatarState
  roomName: string
  onReload?: () => void
  onOpenBrowser?: () => void
}

export function MorganDebugPanel({
  health,
  diagnostics,
  avatarState,
  roomName,
  onReload,
  onOpenBrowser,
}: MorganDebugPanelProps) {
  return (
    <div className="flex min-h-0 flex-col gap-4 overflow-auto pr-1">
      <DebugCard
        title="Local runtime"
        rows={[
          ['Active kubectl context', health?.activeContext ?? 'unknown'],
          ['Expected context', health?.expectedContext ?? 'kind-cto-lite'],
          ['Docker', health?.dockerAvailable ? 'available' : 'unavailable'],
          ['Kind cluster', health?.kindClusterExists ? 'present' : 'missing'],
          ['Kind reachable', health?.kindContextReachable ? 'yes' : 'no'],
          ['Ingress', health?.morganIngressHost ?? 'missing'],
          ['Morgan service', health?.morganServicePresent ? 'present' : 'missing'],
          ['Gateway health', health?.gatewayReachable ? 'reachable' : 'offline'],
        ]}
      />

      <DebugCard
        title="Session"
        rows={[
          ['Connection', avatarState.connectionState],
          ['Voice state', avatarState.voiceState],
          ['Room', avatarState.roomName ?? roomName],
          ['Identity', avatarState.identity ?? 'guest'],
          ['Audio track', avatarState.audioTrackReady ? 'ready' : 'pending'],
          ['Video track', avatarState.videoTrackReady ? 'ready' : 'pending'],
          ['Mic', avatarState.microphoneEnabled ? 'enabled' : 'muted'],
        ]}
      />

      <DebugCard
        title="Morgan"
        rows={[
          ['Healthy', diagnostics?.healthy ? 'yes' : 'no'],
          ['Primary model', diagnostics?.modelPrimary ?? 'unresolved'],
          ['Fallbacks', diagnostics?.modelFallbacks.join(', ') || 'none'],
        ]}
      />

      {health?.problems?.length ? (
        <Card className="rounded-[24px] border-white/10 bg-black/15">
          <CardHeader>
            <CardTitle className="text-base text-white">Local blockers</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            {health.problems.map((entry) => (
              <div
                key={entry}
                className="rounded-[18px] border border-white/8 bg-black/20 px-3 py-2 text-xs leading-6 text-slate-300"
              >
                {entry}
              </div>
            ))}
          </CardContent>
        </Card>
      ) : null}

      {diagnostics?.recentErrors?.length ? (
        <Card className="rounded-[24px] border-white/10 bg-black/15">
          <CardHeader>
            <CardTitle className="text-base text-white">Recent Morgan errors</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            {diagnostics.recentErrors.map((entry) => (
              <div
                key={entry}
                className="rounded-[18px] border border-white/8 bg-black/20 px-3 py-2 text-xs leading-6 text-slate-300"
              >
                {entry}
              </div>
            ))}
          </CardContent>
        </Card>
      ) : null}

      {onReload || onOpenBrowser ? (
        <Card className="rounded-[24px] border-white/10 bg-black/15">
          <CardHeader>
            <CardTitle className="text-base text-white">Avatar source</CardTitle>
          </CardHeader>
          <CardContent className="flex flex-wrap gap-2">
            {onReload ? (
              <Button variant="secondary" size="sm" onClick={onReload}>
                <RefreshCw className="mr-2 h-4 w-4" />
                Reload surface
              </Button>
            ) : null}
            {onOpenBrowser ? (
              <Button variant="outline" size="sm" onClick={onOpenBrowser}>
                <ExternalLink className="mr-2 h-4 w-4" />
                Open in browser
              </Button>
            ) : null}
          </CardContent>
        </Card>
      ) : null}
    </div>
  )
}

function DebugCard({
  title,
  rows,
}: {
  title: string
  rows: Array<[string, string]>
}) {
  return (
    <Card className="rounded-[24px] border-white/10 bg-black/15">
      <CardHeader>
        <CardTitle className="text-base text-white">{title}</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        {rows.map(([label, value]) => (
          <div key={label} className="flex items-start justify-between gap-3 text-sm">
            <span className="text-slate-400">{label}</span>
            <span className="max-w-[14rem] text-right text-slate-100">{value}</span>
          </div>
        ))}
      </CardContent>
    </Card>
  )
}

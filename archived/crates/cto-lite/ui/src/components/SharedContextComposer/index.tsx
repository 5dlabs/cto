import { Loader2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'

interface SharedContextComposerProps {
  agentName: string
  roomName: string | null
  value: string
  status: string | null
  sending: boolean
  onValueChange: (value: string) => void
  onSend: () => void
}

export function SharedContextComposer({
  agentName,
  roomName,
  value,
  status,
  sending,
  onValueChange,
  onSend,
}: SharedContextComposerProps) {
  return (
    <div className="rounded-[24px] border border-white/10 bg-black/20 p-4">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <p className="text-[11px] uppercase tracking-[0.28em] text-cyan-100/70">Context</p>
          <p className="mt-2 max-w-xl text-sm leading-6 text-slate-300">
            Paste a link, PRD excerpt, or working notes here. Then talk naturally and refer to
            “this link” or “this brief.”
          </p>
        </div>
        <Button size="sm" onClick={onSend} disabled={sending || !roomName}>
          {sending ? (
            <>
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              Sending
            </>
          ) : (
            `Send to ${agentName}`
          )}
        </Button>
      </div>

      <Textarea
        value={value}
        onChange={(event) => onValueChange(event.target.value)}
        placeholder={`Paste supporting context for ${agentName}.`}
        className="mt-4 min-h-[92px] border-white/10 bg-black/25 text-slate-100 placeholder:text-slate-500"
      />

      <div className="mt-3 flex flex-wrap items-center justify-between gap-3 text-xs text-slate-400">
        <span>{roomName ? `Targets room ${roomName}.` : 'Start a call first to bind the active room.'}</span>
        {status ? <span className="text-slate-200">{status}</span> : null}
      </div>
    </div>
  )
}

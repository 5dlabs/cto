import { Badge } from '@/components/ui/badge'
import {
  CheckCircle2,
  Circle,
  Loader2,
  XCircle,
} from 'lucide-react'

// ============================================================================
// Types
// ============================================================================

export interface LifecycleStage {
  id: string
  name: string
  status: 'pending' | 'active' | 'completed' | 'failed'
  message?: string
  agent?: string
}

interface LifecycleProgressProps {
  stages: LifecycleStage[]
  workflowId: string
}

// ============================================================================
// Component
// ============================================================================

export function LifecycleProgress({ stages }: LifecycleProgressProps) {
  return (
    <div className="space-y-1">
      {stages.map((stage, idx) => {
        const isLast = idx === stages.length - 1

        return (
          <div key={stage.id} className="flex items-start gap-3">
            {/* Timeline connector */}
            <div className="flex flex-col items-center">
              <StageIcon status={stage.status} />
              {!isLast && (
                <div
                  className={`w-px h-8 ${
                    stage.status === 'completed'
                      ? 'bg-green-500'
                      : stage.status === 'active'
                        ? 'bg-blue-500/30'
                        : 'bg-border'
                  }`}
                />
              )}
            </div>

            {/* Stage info */}
            <div className="pb-6 min-w-0">
              <div className="flex items-center gap-2">
                <span
                  className={`font-medium text-sm ${
                    stage.status === 'pending'
                      ? 'text-muted-foreground'
                      : ''
                  }`}
                >
                  {stage.name}
                </span>
                {stage.agent && (
                  <Badge variant="outline" className="text-[10px]">
                    {stage.agent}
                  </Badge>
                )}
                <StageBadge status={stage.status} />
              </div>
              {stage.message && (
                <p className="text-xs text-muted-foreground mt-0.5">
                  {stage.message}
                </p>
              )}
            </div>
          </div>
        )
      })}
    </div>
  )
}

// ============================================================================
// Sub-components
// ============================================================================

function StageIcon({ status }: { status: LifecycleStage['status'] }) {
  switch (status) {
    case 'completed':
      return <CheckCircle2 className="h-5 w-5 text-green-500 shrink-0" />
    case 'active':
      return <Loader2 className="h-5 w-5 text-blue-500 animate-spin shrink-0" />
    case 'failed':
      return <XCircle className="h-5 w-5 text-red-500 shrink-0" />
    default:
      return <Circle className="h-5 w-5 text-muted-foreground/40 shrink-0" />
  }
}

function StageBadge({ status }: { status: LifecycleStage['status'] }) {
  switch (status) {
    case 'completed':
      return (
        <Badge
          variant="outline"
          className="text-[10px] text-green-600 border-green-500/30"
        >
          Done
        </Badge>
      )
    case 'active':
      return (
        <Badge
          variant="outline"
          className="text-[10px] text-blue-600 border-blue-500/30"
        >
          Running
        </Badge>
      )
    case 'failed':
      return (
        <Badge variant="destructive" className="text-[10px]">
          Failed
        </Badge>
      )
    default:
      return null
  }
}

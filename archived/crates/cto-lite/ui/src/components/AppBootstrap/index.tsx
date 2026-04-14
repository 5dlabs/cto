import { useEffect, useMemo, useRef, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import { AlertCircle, Loader2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import * as tauri from '@/lib/tauri'

type BootstrapState = 'checking' | 'installing' | 'ready' | 'failed'

const STEP_COPY: Record<tauri.InstallStep, string> = {
  CheckingPrerequisites: 'Installing dependencies',
  InstallingBinaries: 'Installing dependencies',
  CreatingCluster: 'Preparing local environment',
  PullingImages: 'Preparing local engine',
  DeployingServices: 'Starting local services',
  ConfiguringIngress: 'Starting local services',
  Complete: 'Launching CTO',
  Failed: 'Setup interrupted',
}

function hasTauriRuntime(): boolean {
  return (
    typeof window !== 'undefined' &&
    typeof (window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ !==
      'undefined'
  )
}

export function AppBootstrap() {
  const startedRef = useRef(false)
  const [state, setState] = useState<BootstrapState>('checking')
  const [status, setStatus] = useState<tauri.InstallStatus | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [showDetails, setShowDetails] = useState(false)
  const [simulatedProgress, setSimulatedProgress] = useState(8)

  async function runBootstrap() {
    if (!hasTauriRuntime()) {
      setState('ready')
      return
    }

    setError(null)
    setStatus(null)
    setState('checking')

    let complete = false
    try {
      complete = await Promise.race<boolean>([
        tauri.getInstallStatus(),
        new Promise((resolve) => {
          window.setTimeout(() => resolve(false), 1200)
        }),
      ])
    } catch {
      setState('ready')
      return
    }

    if (complete) {
      setState('ready')
      return
    }

    setState('installing')
    await tauri.runInstallation()
    setState('ready')
  }

  useEffect(() => {
    if (startedRef.current) {
      return
    }
    startedRef.current = true

    let disposed = false
    let unlisten: (() => void) | undefined

    const start = async () => {
      try {
        if (!hasTauriRuntime()) {
          setState('ready')
          return
        }

        void listen<tauri.InstallStatus>('install-progress', (event) => {
          if (disposed) {
            return
          }
          setStatus(event.payload)
        }).then((detach) => {
          if (disposed) {
            detach()
            return
          }
          unlisten = detach
        })

        await runBootstrap()
      } catch (err) {
        if (disposed) {
          return
        }
        setError(String(err))
        setState('failed')
      }
    }

    void start()

    return () => {
      disposed = true
      unlisten?.()
    }
  }, [])

  useEffect(() => {
    if (state === 'checking') {
      setSimulatedProgress(8)
      return
    }

    if (state !== 'installing' || status) {
      return
    }

    setSimulatedProgress((current) => Math.max(current, 12))

    const timer = window.setInterval(() => {
      setSimulatedProgress((current) => {
        if (current >= 88) {
          return current
        }
        if (current < 34) {
          return current + 4
        }
        if (current < 64) {
          return current + 3
        }
        return current + 2
      })
    }, 1400)

    return () => {
      window.clearInterval(timer)
    }
  }, [state, status])

  const title = useMemo(() => {
    if (state === 'checking') {
      return 'Preparing CTO'
    }
    if (state === 'failed') {
      return 'Setup interrupted'
    }
    if (!status) {
      return state === 'installing' ? 'Installing dependencies' : 'Preparing CTO'
    }
    return STEP_COPY[status.step] ?? 'Preparing CTO'
  }, [state, status])

  const subtitle = useMemo(() => {
    if (state === 'failed') {
      return 'CTO could not finish local setup.'
    }
    if (state === 'checking') {
      return 'Checking your local environment.'
    }
    if (status?.step === 'Complete') {
      return 'Finalizing the app.'
    }
    return 'This can take a few minutes the first time.'
  }, [state, status])

  if (state === 'ready') {
    return null
  }

  const progress = status?.progress ?? simulatedProgress

  return (
    <div className="absolute inset-0 z-50 flex items-center justify-center overflow-hidden bg-[radial-gradient(circle_at_top,#15314f_0%,#0f172a_38%,#050816_100%)]">
      <div className="absolute inset-0 bg-[linear-gradient(135deg,rgba(56,189,248,0.08),transparent_38%,rgba(244,114,182,0.06))]" />

      <div className="relative w-full max-w-xl px-8">
        <div className="rounded-3xl border border-white/10 bg-white/6 p-8 shadow-2xl backdrop-blur-xl">
          <div className="mb-6 flex items-center gap-3 text-white">
            {state === 'failed' ? (
              <AlertCircle className="h-5 w-5 text-rose-300" />
            ) : (
              <Loader2 className="h-5 w-5 animate-spin text-sky-300" />
            )}
            <span className="text-xs font-semibold uppercase tracking-[0.24em] text-sky-200/80">
              Local Setup
            </span>
          </div>

          <div className="space-y-2">
            <h1 className="text-3xl font-semibold tracking-tight text-white">{title}</h1>
            <p className="max-w-md text-sm text-slate-300">{subtitle}</p>
          </div>

          <div className="mt-8 space-y-3">
            <Progress value={progress} className="h-2 bg-white/10" />
            <div className="flex items-center justify-between text-xs text-slate-400">
              <span>{status?.message ?? 'Preparing your workspace.'}</span>
              <span>{progress}%</span>
            </div>
          </div>

          {state === 'failed' && (
            <div className="mt-6 space-y-4 rounded-2xl border border-rose-400/20 bg-rose-500/10 p-4">
              <p className="text-sm text-rose-100">
                CTO couldn&apos;t finish local setup. Retry after Docker Desktop is fully running.
              </p>
              <div className="flex gap-3">
                <Button
                  onClick={() => {
                    void runBootstrap().catch((err) => {
                      setError(String(err))
                      setState('failed')
                    })
                  }}
                >
                  Retry setup
                </Button>
                <Button variant="secondary" onClick={() => setShowDetails((current) => !current)}>
                  {showDetails ? 'Hide details' : 'Show details'}
                </Button>
              </div>
              {showDetails && (
                <pre className="overflow-x-auto rounded-xl bg-black/30 p-3 text-xs text-slate-200">
                  {error ?? status?.error ?? 'Unknown bootstrap error'}
                </pre>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

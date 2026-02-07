import { useState } from 'react'
import { Button } from '@/components/ui/button'
import { Loader2, Rocket } from 'lucide-react'
import * as tauri from '@/lib/tauri'

interface LaunchButtonProps {
  /** PRD content to launch */
  prdContent: string
  /** Optional PRD title for display */
  prdTitle?: string
  /** Callback on successful launch */
  onLaunch?: (workflowId: string) => void
  /** Callback on error */
  onError?: (error: string) => void
  /** Disable the button */
  disabled?: boolean
}

export function LaunchButton({
  prdContent,
  prdTitle,
  onLaunch,
  onError,
  disabled = false,
}: LaunchButtonProps) {
  const [launching, setLaunching] = useState(false)
  const [launched, setLaunched] = useState(false)

  async function handleLaunch() {
    if (!prdContent.trim()) return

    setLaunching(true)
    try {
      const result = await tauri.openclawStartWorkflow('intake', {
        prd_content: prdContent,
        prd_title: prdTitle ?? 'Untitled PRD',
      })
      setLaunched(true)
      onLaunch?.(result.workflowId)
    } catch (error) {
      onError?.(String(error))
    } finally {
      setLaunching(false)
    }
  }

  if (launched) {
    return (
      <Button
        size="lg"
        className="gap-2 bg-green-600 hover:bg-green-700 text-white"
        disabled
      >
        <Rocket className="h-5 w-5" />
        Launched
      </Button>
    )
  }

  return (
    <Button
      size="lg"
      onClick={handleLaunch}
      disabled={disabled || launching || !prdContent.trim()}
      className="gap-2 bg-gradient-to-r from-orange-500 to-red-500 hover:from-orange-600 hover:to-red-600 text-white shadow-lg shadow-orange-500/25 transition-all hover:shadow-orange-500/40 hover:scale-[1.02] active:scale-[0.98]"
    >
      {launching ? (
        <>
          <Loader2 className="h-5 w-5 animate-spin" />
          Launching...
        </>
      ) : (
        <>
          <Rocket className="h-5 w-5" />
          Launch
        </>
      )}
    </Button>
  )
}

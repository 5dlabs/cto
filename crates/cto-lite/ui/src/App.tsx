import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { SetupWizard } from './components/SetupWizard'
import { Dashboard } from './components/Dashboard'
import { Toaster } from './components/ui/toaster'

interface SetupStatus {
  current_step: number
  completed: boolean
  steps: Array<{
    id: number
    name: string
    description: string
    completed: boolean
  }>
}

function App() {
  const [loading, setLoading] = useState(true)
  const [setupComplete, setSetupComplete] = useState(false)
  const [setupStatus, setSetupStatus] = useState<SetupStatus | null>(null)

  useEffect(() => {
    checkSetupStatus()
  }, [])

  async function checkSetupStatus() {
    try {
      const status = await invoke<SetupStatus>('get_setup_status')
      setSetupStatus(status)
      setSetupComplete(status.completed)
    } catch (error) {
      console.error('Failed to get setup status:', error)
    } finally {
      setLoading(false)
    }
  }

  function handleSetupComplete() {
    setSetupComplete(true)
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="flex flex-col items-center gap-4">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
          <p className="text-sm text-muted-foreground">Loading CTO Lite...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen">
      {!setupComplete ? (
        <SetupWizard 
          initialStep={setupStatus?.current_step ?? 0}
          onComplete={handleSetupComplete}
        />
      ) : (
        <Dashboard />
      )}
      <Toaster />
    </div>
  )
}

export default App

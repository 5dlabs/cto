import { useEffect, useState } from 'react'
import { SetupWizard } from './components/SetupWizard'
import { Dashboard } from './components/Dashboard'
import { Toaster } from './components/ui/toaster'
import * as tauri from './lib/tauri'

function App() {
  const [loading, setLoading] = useState(true)
  const [setupComplete, setSetupComplete] = useState(false)
  const [currentStep, setCurrentStep] = useState(0)

  useEffect(() => {
    checkSetupStatus()
  }, [])

  async function checkSetupStatus() {
    try {
      const state = await tauri.getSetupState()
      setCurrentStep(state.currentStep)
      setSetupComplete(state.completed)
    } catch (error) {
      console.error('Failed to get setup status:', error)
      // Default to showing setup wizard
      setSetupComplete(false)
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
          initialStep={currentStep}
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

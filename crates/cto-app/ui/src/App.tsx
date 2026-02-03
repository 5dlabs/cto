import { useState } from 'react';
import { Home } from './pages/Home';
import { Setup } from './pages/Setup';
import { Cluster } from './pages/Cluster';
import { Settings } from './components/Settings';
import { Dashboard } from './components/Dashboard';

type View = 'home' | 'setup' | 'cluster' | 'settings' | 'dashboard';

function App() {
  const [currentView, setCurrentView] = useState<View>('home');

  const renderView = () => {
    switch (currentView) {
      case 'home':
        return <Home onNavigate={setCurrentView} />;
      case 'setup':
        return <Setup onBack={() => setCurrentView('home')} />;
      case 'cluster':
        return <Cluster onBack={() => setCurrentView('home')} />;
      case 'settings':
        return <Settings onBack={() => setCurrentView('home')} />;
      case 'dashboard':
        return (
          <DashboardView
            onBack={() => setCurrentView('home')}
            onNavigate={setCurrentView}
          />
        );
      default:
        return <Home onNavigate={setCurrentView} />;
    }
  };

  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-100">
      <nav className="border-b border-zinc-800 px-6 py-4">
        <div className="flex items-center justify-between">
          <h1 className="text-xl font-semibold tracking-tight">CTO App</h1>
          <div className="flex items-center gap-4">
            <button
              onClick={() => setCurrentView('home')}
              className={`text-sm font-medium transition-colors ${
                currentView === 'home' ? 'text-zinc-100' : 'text-zinc-400 hover:text-zinc-100'
              }`}
            >
              Home
            </button>
            <button
              onClick={() => setCurrentView('dashboard')}
              className={`text-sm font-medium transition-colors ${
                currentView === 'dashboard' ? 'text-zinc-100' : 'text-zinc-400 hover:text-zinc-100'
              }`}
            >
              Dashboard
            </button>
            <button
              onClick={() => setCurrentView('settings')}
              className={`text-sm font-medium transition-colors ${
                currentView === 'settings' ? 'text-zinc-100' : 'text-zinc-400 hover:text-zinc-100'
              }`}
            >
              Settings
            </button>
          </div>
        </div>
      </nav>
      <main className="container mx-auto px-6 py-8">
        {renderView()}
      </main>
    </div>
  );
}

interface DashboardViewProps {
  onBack: () => void;
  onNavigate: (view: View) => void;
}

function DashboardView({ onBack }: DashboardViewProps) {
  // In a real app, this would come from Tauri/Rust backend
  const clusterStatus: 'running' | 'stopped' | 'starting' = 'running';

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Dashboard</h1>
          <p className="text-zinc-400 mt-1">
            Monitor your CTO cluster, agents, and workflows
          </p>
        </div>
        <div className="flex gap-3">
          <button
            onClick={onBack}
            className="text-sm font-medium text-zinc-400 hover:text-zinc-100 transition-colors"
          >
            ← Back to Home
          </button>
        </div>
      </div>

      <Dashboard status={clusterStatus} />
    </div>
  );
}

export default App;

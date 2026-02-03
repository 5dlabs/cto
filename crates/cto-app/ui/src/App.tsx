import { useState } from 'react';
import { Home } from './pages/Home';
import { Setup } from './pages/Setup';
import { Cluster } from './pages/Cluster';
import { Settings } from './components/Settings';

type View = 'home' | 'setup' | 'cluster' | 'settings';

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
              className="text-sm font-medium text-zinc-400 hover:text-zinc-100 transition-colors"
            >
              Home
            </button>
            <button
              onClick={() => setCurrentView('settings')}
              className="text-sm font-medium text-zinc-400 hover:text-zinc-100 transition-colors"
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

export default App;

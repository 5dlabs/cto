'use client';

import { useState, useEffect } from 'react';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { 
  Play, 
  Pause, 
  RefreshCw, 
  CheckCircle, 
  XCircle, 
  Clock,
  Terminal,
  Wrench,
  AlertTriangle
} from 'lucide-react';

interface RalphState {
  sessionId: string;
  executor: {
    status: 'running' | 'paused' | 'complete' | 'failed' | 'stopped';
    currentStep: string;
    stepNumber: number;
    totalSteps: number;
    lastUpdate: string;
    lastError: string | null;
  };
  watcher: {
    status: string;
    lastCheck: string | null;
    checkCount: number;
  };
  stats: {
    totalRetries: number;
    issuesDetected: number;
    issuesFixed: number;
    successfulSteps: number;
    totalDuration: string;
  };
  hardeningActions: Array<{
    issue: string;
    workaround: string;
    timestamp: string;
    success: boolean;
  }>;
  progressLog: string[];
}

const defaultState: RalphState = {
  sessionId: 'not-connected',
  executor: {
    status: 'stopped',
    currentStep: 'Waiting...',
    stepNumber: 0,
    totalSteps: 0,
    lastUpdate: new Date().toISOString(),
    lastError: null,
  },
  watcher: {
    status: 'stopped',
    lastCheck: null,
    checkCount: 0,
  },
  stats: {
    totalRetries: 0,
    issuesDetected: 0,
    issuesFixed: 0,
    successfulSteps: 0,
    totalDuration: '0m',
  },
  hardeningActions: [],
  progressLog: ['Waiting for Ralph loop to connect...'],
};

function StatusBadge({ status }: { status: string }) {
  const config: Record<string, { color: string; icon: React.ReactNode }> = {
    running: { color: 'bg-green-500', icon: <Play className="w-3 h-3" /> },
    paused: { color: 'bg-yellow-500', icon: <Pause className="w-3 h-3" /> },
    complete: { color: 'bg-blue-500', icon: <CheckCircle className="w-3 h-3" /> },
    failed: { color: 'bg-red-500', icon: <XCircle className="w-3 h-3" /> },
    stopped: { color: 'bg-gray-500', icon: <Clock className="w-3 h-3" /> },
  };
  
  const { color, icon } = config[status] || config.stopped;
  
  return (
    <span className={`inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs text-white ${color}`}>
      {icon}
      {status}
    </span>
  );
}

function TimeAgo({ timestamp }: { timestamp: string }) {
  const [timeAgo, setTimeAgo] = useState('');
  
  useEffect(() => {
    const update = () => {
      const now = new Date();
      const then = new Date(timestamp);
      const diff = Math.floor((now.getTime() - then.getTime()) / 1000);
      
      if (diff < 60) setTimeAgo(`${diff}s ago`);
      else if (diff < 3600) setTimeAgo(`${Math.floor(diff / 60)}m ago`);
      else setTimeAgo(`${Math.floor(diff / 3600)}h ago`);
    };
    
    update();
    const interval = setInterval(update, 10000);
    return () => clearInterval(interval);
  }, [timestamp]);
  
  return <span className="text-gray-400 text-sm">{timeAgo}</span>;
}

export default function RalphDashboard() {
  const [state, setState] = useState<RalphState>(defaultState);
  const [loading, setLoading] = useState(false);
  const [lastRefresh, setLastRefresh] = useState<Date>(new Date());
  
  const fetchState = async () => {
    setLoading(true);
    try {
      const res = await fetch('/api/ralph/state');
      if (res.ok) {
        const data = await res.json();
        setState(data);
      }
    } catch (e) {
      console.error('Failed to fetch state:', e);
    } finally {
      setLoading(false);
      setLastRefresh(new Date());
    }
  };
  
  const sendCommand = async (command: 'pause' | 'resume' | 'stop') => {
    try {
      await fetch('/api/ralph/command', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ command }),
      });
      // Refresh after command
      setTimeout(fetchState, 1000);
    } catch (e) {
      console.error('Failed to send command:', e);
    }
  };
  
  useEffect(() => {
    fetchState();
    // Auto-refresh every 30 seconds
    const interval = setInterval(fetchState, 30000);
    return () => clearInterval(interval);
  }, []);
  
  const progress = state.executor.totalSteps > 0 
    ? Math.round((state.executor.stepNumber / state.executor.totalSteps) * 100)
    : 0;

  return (
    <div className="min-h-screen bg-black text-white p-4 pb-24">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold">Ralph Control</h1>
          <p className="text-gray-400 text-sm">Session: {state.sessionId.slice(0, 8)}...</p>
        </div>
        <Button 
          variant="outline" 
          size="icon"
          onClick={fetchState}
          disabled={loading}
          className="border-gray-700"
        >
          <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
        </Button>
      </div>
      
      {/* Main Status Card */}
      <Card className="bg-gray-900 border-gray-800 p-4 mb-4">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-3">
            <Terminal className="w-6 h-6 text-blue-400" />
            <div>
              <div className="font-semibold">Executor</div>
              <StatusBadge status={state.executor.status} />
            </div>
          </div>
          <TimeAgo timestamp={state.executor.lastUpdate} />
        </div>
        
        {/* Progress Bar */}
        <div className="mb-4">
          <div className="flex justify-between text-sm mb-1">
            <span>{state.executor.currentStep}</span>
            <span>{state.executor.stepNumber}/{state.executor.totalSteps}</span>
          </div>
          <div className="w-full bg-gray-800 rounded-full h-3">
            <div 
              className="bg-blue-500 h-3 rounded-full transition-all duration-500"
              style={{ width: `${progress}%` }}
            />
          </div>
        </div>
        
        {/* Error Display */}
        {state.executor.lastError && (
          <div className="bg-red-900/30 border border-red-800 rounded p-3 text-sm">
            <div className="flex items-center gap-2 text-red-400 mb-1">
              <AlertTriangle className="w-4 h-4" />
              <span className="font-semibold">Error</span>
            </div>
            <p className="text-red-300">{state.executor.lastError}</p>
          </div>
        )}
      </Card>
      
      {/* Stats Grid */}
      <div className="grid grid-cols-2 gap-3 mb-4">
        <Card className="bg-gray-900 border-gray-800 p-3">
          <div className="text-gray-400 text-xs mb-1">Duration</div>
          <div className="text-xl font-bold">{state.stats.totalDuration}</div>
        </Card>
        <Card className="bg-gray-900 border-gray-800 p-3">
          <div className="text-gray-400 text-xs mb-1">Steps Done</div>
          <div className="text-xl font-bold">{state.stats.successfulSteps}</div>
        </Card>
        <Card className="bg-gray-900 border-gray-800 p-3">
          <div className="text-gray-400 text-xs mb-1">Issues Found</div>
          <div className="text-xl font-bold text-yellow-400">{state.stats.issuesDetected}</div>
        </Card>
        <Card className="bg-gray-900 border-gray-800 p-3">
          <div className="text-gray-400 text-xs mb-1">Issues Fixed</div>
          <div className="text-xl font-bold text-green-400">{state.stats.issuesFixed}</div>
        </Card>
      </div>
      
      {/* Watcher Status */}
      <Card className="bg-gray-900 border-gray-800 p-4 mb-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Wrench className="w-5 h-5 text-purple-400" />
            <div>
              <div className="font-semibold">Watcher (Hardening)</div>
              <div className="text-gray-400 text-sm">
                {state.watcher.checkCount} checks completed
              </div>
            </div>
          </div>
          <StatusBadge status={state.watcher.status} />
        </div>
      </Card>
      
      {/* Recent Actions */}
      {state.hardeningActions.length > 0 && (
        <Card className="bg-gray-900 border-gray-800 p-4 mb-4">
          <h3 className="font-semibold mb-3">Recent Fixes</h3>
          <div className="space-y-2 max-h-48 overflow-y-auto">
            {state.hardeningActions.slice(-5).reverse().map((action, i) => (
              <div key={i} className="text-sm border-l-2 border-green-500 pl-3">
                <div className="text-gray-300">{action.issue}</div>
                <div className="text-gray-500 text-xs">{action.workaround}</div>
              </div>
            ))}
          </div>
        </Card>
      )}
      
      {/* Progress Log */}
      <Card className="bg-gray-900 border-gray-800 p-4 mb-4">
        <h3 className="font-semibold mb-3">Activity Log</h3>
        <div className="bg-black rounded p-3 max-h-64 overflow-y-auto font-mono text-xs">
          {state.progressLog.slice(-20).map((line, i) => (
            <div key={i} className="text-gray-400 leading-relaxed">
              {line}
            </div>
          ))}
        </div>
      </Card>
      
      {/* Fixed Control Bar */}
      <div className="fixed bottom-0 left-0 right-0 bg-gray-900 border-t border-gray-800 p-4 flex gap-3">
        {state.executor.status === 'running' ? (
          <Button 
            className="flex-1 bg-yellow-600 hover:bg-yellow-700"
            onClick={() => sendCommand('pause')}
          >
            <Pause className="w-4 h-4 mr-2" />
            Pause
          </Button>
        ) : (
          <Button 
            className="flex-1 bg-green-600 hover:bg-green-700"
            onClick={() => sendCommand('resume')}
          >
            <Play className="w-4 h-4 mr-2" />
            Resume
          </Button>
        )}
        <Button 
          variant="outline"
          className="border-red-600 text-red-400 hover:bg-red-900/30"
          onClick={() => sendCommand('stop')}
        >
          <XCircle className="w-4 h-4" />
        </Button>
      </div>
      
      {/* Last refresh indicator */}
      <div className="text-center text-gray-500 text-xs mt-2">
        Last refresh: {lastRefresh.toLocaleTimeString()}
      </div>
    </div>
  );
}

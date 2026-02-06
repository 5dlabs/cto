import { Button } from '../components/ui/button';
import { SetupWizard } from '../components/SetupWizard';
import { ArrowLeft } from 'lucide-react';

interface SetupProps {
  onBack: () => void;
}

export function Setup({ onBack }: SetupProps) {
  const handleComplete = () => {
    onBack();
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-4">
        <Button variant="ghost" size="icon" onClick={onBack}>
          <ArrowLeft className="w-4 h-4" />
        </Button>
        <div>
          <h1 className="text-2xl font-semibold">Setup Wizard</h1>
          <p className="text-zinc-400 mt-1">
            Configure your CTO App environment
          </p>
        </div>
      </div>

      <SetupWizard onComplete={handleComplete} />
    </div>
  );
}

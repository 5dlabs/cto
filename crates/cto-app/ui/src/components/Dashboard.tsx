import { Card, CardContent, CardHeader, CardTitle } from './ui/card';
import { Server, Database, GitBranch, Activity } from 'lucide-react';

interface DashboardProps {
  clusterName: string;
  status: 'running' | 'stopped' | 'starting';
}

export function Dashboard({ clusterName, status }: DashboardProps) {
  const stats = [
    {
      title: 'Cluster Status',
      value: status.charAt(0).toUpperCase() + status.slice(1),
      icon: Server,
      color: status === 'running' ? 'text-emerald-500' : 'text-amber-500',
    },
    {
      title: 'Active Workflows',
      value: '3',
      icon: Activity,
      color: 'text-blue-500',
    },
    {
      title: 'Repositories',
      value: '12',
      icon: GitBranch,
      color: 'text-purple-500',
    },
    {
      title: 'Deployments',
      value: '8',
      icon: Database,
      color: 'text-cyan-500',
    },
  ];

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
      {stats.map((stat) => (
        <Card key={stat.title} className="bg-zinc-900/50 border-zinc-800">
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium text-zinc-400">
              {stat.title}
            </CardTitle>
            <stat.icon className={`h-4 w-4 ${stat.color}`} />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stat.value}</div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}

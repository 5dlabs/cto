import { auth } from '@/lib/auth';
import { headers } from 'next/headers';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar';
import Link from 'next/link';

const agents = [
  {
    name: 'Morgan',
    role: 'Project Manager',
    status: 'idle',
    avatar: '/agents/morgan.png',
    color: 'from-violet-500 to-purple-600',
  },
  {
    name: 'Rex',
    role: 'Rust Specialist',
    status: 'working',
    avatar: '/agents/rex.png',
    color: 'from-orange-500 to-red-600',
  },
  {
    name: 'Blaze',
    role: 'Frontend',
    status: 'idle',
    avatar: '/agents/blaze.png',
    color: 'from-yellow-500 to-orange-600',
  },
  {
    name: 'Cleo',
    role: 'Code Review',
    status: 'idle',
    avatar: '/agents/cleo.png',
    color: 'from-pink-500 to-rose-600',
  },
];

const recentActivity = [
  { id: 1, agent: 'Rex', action: 'Opened PR #42', repo: 'acme/api', time: '2 min ago' },
  { id: 2, agent: 'Cleo', action: 'Approved PR #41', repo: 'acme/api', time: '15 min ago' },
  { id: 3, agent: 'Morgan', action: 'Created 5 tasks', repo: 'acme/web', time: '1 hour ago' },
];

export default async function DashboardPage() {
  const session = await auth.api.getSession({
    headers: await headers(),
  });

  return (
    <div className="flex h-full flex-col overflow-auto">
      {/* Header */}
      <div className="border-border border-b px-8 py-6">
        <h1 className="text-2xl font-bold">Dashboard</h1>
        <p className="text-muted-foreground">Welcome back, {session?.user.name?.split(' ')[0]}</p>
      </div>

      <div className="flex-1 space-y-8 p-8">
        {/* Quick Actions */}
        <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
          <Card className="from-primary/10 to-primary/5 border-primary/20 bg-gradient-to-br">
            <CardHeader className="pb-2">
              <CardTitle className="text-lg">New Project</CardTitle>
              <CardDescription>Create a new project from a PRD</CardDescription>
            </CardHeader>
            <CardContent>
              <Button asChild>
                <Link href="/dashboard/projects/new">Create Project</Link>
              </Button>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-lg">Chat with Morgan</CardTitle>
              <CardDescription>Get help or start a new task</CardDescription>
            </CardHeader>
            <CardContent>
              <Button variant="outline" asChild>
                <Link href="/onboarding">Open Chat</Link>
              </Button>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-lg">API Keys</CardTitle>
              <CardDescription>Manage your AI provider keys</CardDescription>
            </CardHeader>
            <CardContent>
              <Button variant="outline" asChild>
                <Link href="/settings">Settings</Link>
              </Button>
            </CardContent>
          </Card>
        </div>

        {/* Agent Status */}
        <div>
          <h2 className="mb-4 text-lg font-semibold">Agent Status</h2>
          <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
            {agents.map((agent) => (
              <Card key={agent.name} className="relative overflow-hidden">
                <CardContent className="p-4">
                  <div className="flex items-center gap-3">
                    <Avatar className="h-12 w-12">
                      <AvatarImage src={agent.avatar} alt={agent.name} />
                      <AvatarFallback className={`bg-gradient-to-br ${agent.color} text-white`}>
                        {agent.name[0]}
                      </AvatarFallback>
                    </Avatar>
                    <div className="min-w-0 flex-1">
                      <p className="truncate font-medium">{agent.name}</p>
                      <p className="text-muted-foreground truncate text-xs">{agent.role}</p>
                    </div>
                  </div>
                  <div className="mt-3">
                    <Badge
                      variant={agent.status === 'working' ? 'default' : 'secondary'}
                      className="text-xs"
                    >
                      {agent.status === 'working' ? (
                        <span className="flex items-center gap-1">
                          <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-green-400" />
                          Working
                        </span>
                      ) : (
                        'Idle'
                      )}
                    </Badge>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>

        {/* Recent Activity */}
        <div>
          <h2 className="mb-4 text-lg font-semibold">Recent Activity</h2>
          <Card>
            <CardContent className="p-0">
              <div className="divide-border divide-y">
                {recentActivity.map((activity) => (
                  <div key={activity.id} className="flex items-center gap-4 p-4">
                    <Avatar className="h-8 w-8">
                      <AvatarFallback className="bg-primary/10 text-primary text-xs">
                        {activity.agent[0]}
                      </AvatarFallback>
                    </Avatar>
                    <div className="min-w-0 flex-1">
                      <p className="text-sm">
                        <span className="font-medium">{activity.agent}</span> {activity.action}
                      </p>
                      <p className="text-muted-foreground text-xs">{activity.repo}</p>
                    </div>
                    <span className="text-muted-foreground text-xs whitespace-nowrap">
                      {activity.time}
                    </span>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Projects placeholder */}
        <div>
          <div className="mb-4 flex items-center justify-between">
            <h2 className="text-lg font-semibold">Projects</h2>
            <Button variant="ghost" size="sm" asChild>
              <Link href="/dashboard/projects">View all</Link>
            </Button>
          </div>
          <Card className="border-dashed">
            <CardContent className="flex flex-col items-center justify-center py-12 text-center">
              <div className="bg-muted mb-4 flex h-12 w-12 items-center justify-center rounded-full">
                <svg
                  className="text-muted-foreground h-6 w-6"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M12 6v6m0 0v6m0-6h6m-6 0H6"
                  />
                </svg>
              </div>
              <h3 className="mb-1 font-medium">No projects yet</h3>
              <p className="text-muted-foreground mb-4 text-sm">
                Create your first project to get started
              </p>
              <Button asChild>
                <Link href="/dashboard/projects/new">Create Project</Link>
              </Button>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}

import { Avatar, AvatarImage, AvatarFallback } from "@/components/ui/avatar";
import { cn } from "@/lib/utils";

export type Agent = {
  name: string;
  role: string;
  color: string;
  avatar?: string;
  description?: string;
  stack?: string[];
};

export type AgentSquad = {
  title: string;
  emoji: string;
  agents: Agent[];
};

interface AgentCardProps {
  agent: Agent;
  index: number;
  squadIndex: number;
}

export function AgentCard({ agent }: AgentCardProps) {
  return (
    <div className="group cursor-pointer">
      <div
        className={cn(
          "relative flex items-center gap-5 p-5 rounded-xl",
          "bg-card border border-border",
          "hover:border-primary/50 hover:scale-[1.01] hover:-translate-y-0.5",
          "transition-all duration-300"
        )}
      >
        <div
          className={cn(
            "absolute inset-0 rounded-xl opacity-0 group-hover:opacity-20",
            "bg-gradient-to-br transition-opacity duration-300",
            agent.color
          )}
        />

        <Avatar className="size-[90px] ring-2 ring-border group-hover:ring-primary/50 transition-all shrink-0">
          {agent.avatar ? (
            <AvatarImage src={agent.avatar} alt={agent.name} />
          ) : null}
          <AvatarFallback
            className={cn(
              "bg-gradient-to-br text-white text-3xl font-bold",
              agent.color
            )}
          >
            {agent.name.charAt(0)}
          </AvatarFallback>
        </Avatar>

        <div className="relative z-10 flex-1 min-w-0">
          <p className="font-semibold text-lg text-foreground truncate">{agent.name}</p>
          <p className="text-sm text-muted-foreground truncate">{agent.role}</p>
          {agent.stack && agent.stack.length > 0 && (
            <div className="flex flex-wrap gap-1 mt-1">
              {agent.stack.slice(0, 3).map((tech) => (
                <span
                  key={tech}
                  className="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium bg-muted/50 text-muted-foreground"
                >
                  {tech}
                </span>
              ))}
            </div>
          )}
        </div>

        <div
          className={cn(
            "size-2 rounded-full shrink-0",
            "bg-gradient-to-r animate-[pulse_3s_ease-in-out_infinite]",
            agent.color
          )}
        />
      </div>
    </div>
  );
}

interface AgentGridProps {
  squads: AgentSquad[];
}

export function AgentGrid({ squads }: AgentGridProps) {
  return (
    <div className="space-y-8">
      {squads.map((squad, squadIndex) => (
        <div key={squad.title}>
          <div className="flex items-center gap-3 mb-4">
            <span className="text-xl">{squad.emoji}</span>
            <h3 className="text-sm font-medium text-muted-foreground uppercase tracking-wider">
              {squad.title}
            </h3>
            <div className="flex-1 h-px bg-border/50" />
          </div>

          <div
            className={cn(
              "grid gap-4",
              squad.agents.length === 1
                ? "grid-cols-1 max-w-sm mx-auto"
                : "grid-cols-1 sm:grid-cols-2 lg:grid-cols-3"
            )}
          >
            {squad.agents.map((agent, agentIndex) => (
              <AgentCard
                key={agent.name}
                agent={agent}
                index={agentIndex}
                squadIndex={squadIndex}
              />
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

'use client';

import { useState, useRef, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Card } from '@/components/ui/card';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';

interface Message {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  actions?: Action[];
}

interface Action {
  type: 'button' | 'select' | 'input';
  label: string;
  value: string;
  options?: { label: string; value: string }[];
}

const INITIAL_MESSAGES: Message[] = [
  {
    id: '1',
    role: 'assistant',
    content:
      "Welcome to CTO! I'm Morgan, your AI project manager. I'll help you set up your engineering team.\n\nFirst, I need to connect to your GitHub account to see your repositories. This allows our agents to create branches, open PRs, and ship code for you.",
    actions: [
      {
        type: 'button',
        label: 'Connect GitHub',
        value: 'connect_github',
      },
    ],
  },
];

export default function OnboardingPage() {
  const [messages, setMessages] = useState<Message[]>(INITIAL_MESSAGES);
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [step, setStep] = useState<string>('github');
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages]);

  const handleAction = async (action: Action) => {
    setIsLoading(true);

    // Add user action as message
    setMessages((prev) => [
      ...prev,
      {
        id: Date.now().toString(),
        role: 'user',
        content: action.label,
      },
    ]);

    // Simulate response based on action
    setTimeout(() => {
      let response: Message;

      switch (action.value) {
        case 'connect_github':
          response = {
            id: Date.now().toString(),
            role: 'assistant',
            content:
              'GitHub connected successfully! I can see your repositories.\n\nNow I need an API key to power our AI agents. We support multiple providers, but I recommend starting with Anthropic (Claude).\n\nYour API key is stored securely and never logged or exposed.',
            actions: [
              {
                type: 'input',
                label: 'Enter Anthropic API Key',
                value: 'anthropic_key',
              },
            ],
          };
          setStep('api_key');
          break;

        case 'submit_key':
          response = {
            id: Date.now().toString(),
            role: 'assistant',
            content:
              "API key validated. You have access to Claude Sonnet and Opus models.\n\nLast step: Which repository should we start with? I'll analyze it and recommend the right agents for your stack.",
            actions: [
              {
                type: 'select',
                label: 'Select Repository',
                value: 'select_repo',
                options: [
                  { label: 'acme/api (Rust)', value: 'acme/api' },
                  { label: 'acme/web (React/Next.js)', value: 'acme/web' },
                  { label: 'acme/mobile (React Native)', value: 'acme/mobile' },
                ],
              },
            ],
          };
          setStep('repo');
          break;

        case 'select_repo':
          response = {
            id: Date.now().toString(),
            role: 'assistant',
            content:
              "Excellent choice! I've analyzed your repository and here's my recommendation:\n\n**Recommended Squad:**\n• Rex (Rust specialist) - for backend development\n• Cleo (Code review) - for quality assurance\n• Tess (Testing) - for comprehensive test coverage\n• Bolt (Infrastructure) - for deployment setup\n\nI'm now provisioning your workspace. This takes about 2 minutes...",
            actions: [
              {
                type: 'button',
                label: 'Start Provisioning',
                value: 'provision',
              },
            ],
          };
          setStep('provision');
          break;

        case 'provision':
          response = {
            id: Date.now().toString(),
            role: 'assistant',
            content:
              "Your workspace is ready! Here's what I've set up:\n\n✓ Namespace: tenant-acme\n✓ Agents: Rex, Cleo, Tess, Bolt\n✓ GitHub App installed\n✓ Secrets configured\n\nYou're all set! Head to the dashboard to create your first project, or describe what you want to build and I'll create a PRD for you.",
            actions: [
              {
                type: 'button',
                label: 'Go to Dashboard',
                value: 'dashboard',
              },
              {
                type: 'button',
                label: 'Create a Project',
                value: 'create_project',
              },
            ],
          };
          setStep('complete');
          break;

        case 'dashboard':
          window.location.href = '/dashboard';
          return;

        default:
          response = {
            id: Date.now().toString(),
            role: 'assistant',
            content: "I'm processing your request...",
          };
      }

      setMessages((prev) => [...prev, response]);
      setIsLoading(false);
    }, 1500);
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim() || isLoading) return;

    const userMessage: Message = {
      id: Date.now().toString(),
      role: 'user',
      content: input,
    };

    setMessages((prev) => [...prev, userMessage]);
    setInput('');

    // Handle API key submission
    if (step === 'api_key' && input.startsWith('sk-')) {
      handleAction({
        type: 'button',
        label: `API Key: ${input.slice(0, 10)}...`,
        value: 'submit_key',
      });
    }
  };

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <div className="border-border flex items-center justify-between border-b px-6 py-4">
        <div className="flex items-center gap-3">
          <Avatar className="border-primary h-10 w-10 border-2">
            <AvatarImage src="/agents/morgan.png" alt="Morgan" />
            <AvatarFallback className="bg-gradient-to-br from-violet-500 to-purple-600 text-white">
              M
            </AvatarFallback>
          </Avatar>
          <div>
            <h1 className="font-semibold">Setup with Morgan</h1>
            <p className="text-muted-foreground text-sm">AI Project Manager</p>
          </div>
        </div>
        <Badge variant="outline" className="text-xs">
          Step{' '}
          {step === 'github'
            ? '1'
            : step === 'api_key'
              ? '2'
              : step === 'repo'
                ? '3'
                : step === 'provision'
                  ? '4'
                  : '5'}{' '}
          of 5
        </Badge>
      </div>

      {/* Messages */}
      <ScrollArea className="flex-1 p-6" ref={scrollRef}>
        <div className="mx-auto max-w-2xl space-y-6">
          {messages.map((message) => (
            <div
              key={message.id}
              className={cn(
                'flex gap-3',
                message.role === 'user' ? 'justify-end' : 'justify-start'
              )}
            >
              {message.role === 'assistant' && (
                <Avatar className="h-8 w-8 shrink-0">
                  <AvatarImage src="/agents/morgan.png" alt="Morgan" />
                  <AvatarFallback className="bg-gradient-to-br from-violet-500 to-purple-600 text-xs text-white">
                    M
                  </AvatarFallback>
                </Avatar>
              )}
              <div
                className={cn(
                  'flex max-w-[80%] flex-col gap-3',
                  message.role === 'user' ? 'items-end' : 'items-start'
                )}
              >
                <Card
                  className={cn(
                    'px-4 py-3',
                    message.role === 'user' ? 'bg-primary text-primary-foreground' : 'bg-muted'
                  )}
                >
                  <p className="text-sm whitespace-pre-wrap">{message.content}</p>
                </Card>

                {/* Action buttons */}
                {message.actions && (
                  <div className="flex flex-wrap gap-2">
                    {message.actions.map((action, idx) => {
                      if (action.type === 'button') {
                        return (
                          <Button
                            key={idx}
                            onClick={() => handleAction(action)}
                            disabled={isLoading}
                            size="sm"
                          >
                            {action.label}
                          </Button>
                        );
                      }
                      if (action.type === 'select' && action.options) {
                        return (
                          <div key={idx} className="flex flex-col gap-2">
                            {action.options.map((opt) => (
                              <Button
                                key={opt.value}
                                variant="outline"
                                size="sm"
                                onClick={() =>
                                  handleAction({
                                    ...action,
                                    label: opt.label,
                                    value: 'select_repo',
                                  })
                                }
                                disabled={isLoading}
                              >
                                {opt.label}
                              </Button>
                            ))}
                          </div>
                        );
                      }
                      return null;
                    })}
                  </div>
                )}
              </div>
            </div>
          ))}

          {isLoading && (
            <div className="flex gap-3">
              <Avatar className="h-8 w-8 shrink-0">
                <AvatarFallback className="bg-gradient-to-br from-violet-500 to-purple-600 text-xs text-white">
                  M
                </AvatarFallback>
              </Avatar>
              <Card className="bg-muted px-4 py-3">
                <div className="flex gap-1">
                  <span className="bg-muted-foreground/50 h-2 w-2 animate-bounce rounded-full" />
                  <span className="bg-muted-foreground/50 h-2 w-2 animate-bounce rounded-full [animation-delay:0.1s]" />
                  <span className="bg-muted-foreground/50 h-2 w-2 animate-bounce rounded-full [animation-delay:0.2s]" />
                </div>
              </Card>
            </div>
          )}
        </div>
      </ScrollArea>

      {/* Input */}
      <div className="border-border border-t p-4">
        <form onSubmit={handleSubmit} className="mx-auto flex max-w-2xl gap-2">
          <Input
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder={
              step === 'api_key' ? 'Paste your Anthropic API key (sk-ant-...)' : 'Type a message...'
            }
            disabled={isLoading}
            className="flex-1"
          />
          <Button type="submit" disabled={isLoading || !input.trim()}>
            Send
          </Button>
        </form>
      </div>
    </div>
  );
}

"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";

interface ApiKey {
  provider: string;
  name: string;
  configured: boolean;
  lastFour?: string;
}

const apiKeys: ApiKey[] = [
  { provider: "anthropic", name: "Anthropic", configured: true, lastFour: "...abc1" },
  { provider: "openai", name: "OpenAI", configured: false },
  { provider: "google", name: "Google (Gemini)", configured: false },
];

export default function SettingsPage() {
  const [editingKey, setEditingKey] = useState<string | null>(null);
  const [keyValue, setKeyValue] = useState("");

  const handleSaveKey = (provider: string) => {
    // TODO: Call API to save key to OpenBao
    console.log("Saving key for", provider, keyValue);
    setEditingKey(null);
    setKeyValue("");
  };

  return (
    <div className="flex flex-col h-full overflow-auto">
      {/* Header */}
      <div className="px-8 py-6 border-b border-border">
        <h1 className="text-2xl font-bold">Settings</h1>
        <p className="text-muted-foreground">
          Manage your API keys and integrations
        </p>
      </div>

      <div className="flex-1 p-8 max-w-3xl space-y-8">
        {/* API Keys */}
        <section>
          <h2 className="text-lg font-semibold mb-4">API Keys</h2>
          <Card>
            <CardHeader>
              <CardTitle className="text-base">AI Provider Keys</CardTitle>
              <CardDescription>
                Your API keys are stored securely in our vault and never exposed after entry.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              {apiKeys.map((key, index) => (
                <div key={key.provider}>
                  {index > 0 && <Separator className="my-4" />}
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <div className="w-10 h-10 rounded-lg bg-muted flex items-center justify-center">
                        <span className="text-sm font-medium">
                          {key.name.charAt(0)}
                        </span>
                      </div>
                      <div>
                        <p className="font-medium">{key.name}</p>
                        {key.configured ? (
                          <p className="text-xs text-muted-foreground">
                            Configured {key.lastFour}
                          </p>
                        ) : (
                          <p className="text-xs text-muted-foreground">
                            Not configured
                          </p>
                        )}
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      {key.configured && (
                        <Badge variant="secondary" className="text-xs">
                          Active
                        </Badge>
                      )}
                      {editingKey === key.provider ? (
                        <div className="flex items-center gap-2">
                          <Input
                            type="password"
                            value={keyValue}
                            onChange={(e) => setKeyValue(e.target.value)}
                            placeholder="sk-..."
                            className="w-48"
                          />
                          <Button size="sm" onClick={() => handleSaveKey(key.provider)}>
                            Save
                          </Button>
                          <Button
                            size="sm"
                            variant="ghost"
                            onClick={() => {
                              setEditingKey(null);
                              setKeyValue("");
                            }}
                          >
                            Cancel
                          </Button>
                        </div>
                      ) : (
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => setEditingKey(key.provider)}
                        >
                          {key.configured ? "Update" : "Add Key"}
                        </Button>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </CardContent>
          </Card>
        </section>

        {/* Integrations */}
        <section>
          <h2 className="text-lg font-semibold mb-4">Integrations</h2>
          <Card>
            <CardContent className="p-0">
              <div className="divide-y divide-border">
                <div className="flex items-center justify-between p-4">
                  <div className="flex items-center gap-3">
                    <div className="w-10 h-10 rounded-lg bg-muted flex items-center justify-center">
                      <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                        <path fillRule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" clipRule="evenodd" />
                      </svg>
                    </div>
                    <div>
                      <p className="font-medium">GitHub</p>
                      <p className="text-xs text-muted-foreground">Connected</p>
                    </div>
                  </div>
                  <Badge>Connected</Badge>
                </div>
                <div className="flex items-center justify-between p-4">
                  <div className="flex items-center gap-3">
                    <div className="w-10 h-10 rounded-lg bg-muted flex items-center justify-center">
                      <span className="text-sm font-medium">L</span>
                    </div>
                    <div>
                      <p className="font-medium">Linear</p>
                      <p className="text-xs text-muted-foreground">Project management</p>
                    </div>
                  </div>
                  <Button variant="outline" size="sm">Connect</Button>
                </div>
              </div>
            </CardContent>
          </Card>
        </section>

        {/* Organization */}
        <section>
          <h2 className="text-lg font-semibold mb-4">Organization</h2>
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Organization Settings</CardTitle>
              <CardDescription>
                Manage your organization details and team members.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-2">
                <label className="text-sm font-medium">Organization Name</label>
                <Input defaultValue="Acme Inc" />
              </div>
              <div className="grid gap-2">
                <label className="text-sm font-medium">Slug</label>
                <Input defaultValue="acme" disabled />
                <p className="text-xs text-muted-foreground">
                  Your tenant namespace: tenant-acme
                </p>
              </div>
              <Button>Save Changes</Button>
            </CardContent>
          </Card>
        </section>
      </div>
    </div>
  );
}

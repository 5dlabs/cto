"use client";

import { useState } from "react";
import { motion } from "framer-motion";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

type WaitlistFormProps = {
  onSubmitStart?: () => void;
  onSubmitSuccess?: () => void;
  onSubmitError?: () => void;
};

export function WaitlistForm({
  onSubmitStart,
  onSubmitSuccess,
  onSubmitError,
}: WaitlistFormProps) {
  const [email, setEmail] = useState("");
  const [status, setStatus] = useState<"idle" | "loading" | "success" | "error">("idle");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!email) return;

    setStatus("loading");
    onSubmitStart?.();
    
    try {
      const response = await fetch("/api/waitlist", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email }),
      });
      
      const data = await response.json();
      
      if (data.success) {
        setStatus("success");
        setEmail("");
        onSubmitSuccess?.();
      } else {
        setStatus("error");
        onSubmitError?.();
      }
    } catch {
      setStatus("error");
      onSubmitError?.();
    }
  };

  if (status === "success") {
    return (
      <motion.div
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        className="flex items-center gap-3 px-6 py-4 rounded-lg border border-cyan/30 bg-cyan/5"
      >
        <div className="w-2 h-2 rounded-full bg-cyan animate-pulse" />
        <span className="text-cyan font-medium">
          You&apos;re on the list. We&apos;ll be in touch.
        </span>
      </motion.div>
    );
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-col sm:flex-row gap-3 w-full max-w-md">
      <div className="relative flex-1">
        <Input
          type="email"
          placeholder="Enter your email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          className="h-12 bg-card/50 border-border/50 focus:border-cyan/50 focus:ring-cyan/20 placeholder:text-muted-foreground"
          required
        />
        <div className="absolute inset-0 rounded-md pointer-events-none glow-border-cyan opacity-0 transition-opacity focus-within:opacity-100" />
      </div>
      <Button
        type="submit"
        disabled={status === "loading"}
        className="h-12 px-8 bg-cyan text-background font-semibold hover:bg-cyan/90 glow-cyan transition-all"
      >
        {status === "loading" ? (
          <motion.div
            animate={{ rotate: 360 }}
            transition={{ duration: 1, repeat: Infinity, ease: "linear" }}
            className="w-5 h-5 border-2 border-background/30 border-t-background rounded-full"
          />
        ) : (
          "Join Waitlist"
        )}
      </Button>
    </form>
  );
}

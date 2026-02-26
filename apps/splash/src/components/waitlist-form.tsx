"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";

interface WaitlistFormProps {
  source?: "waitlist" | "investor" | "consulting";
  heading?: string;
  subheading?: string;
  compact?: boolean;
}

export function WaitlistForm({
  source = "waitlist",
  heading = "Join the Waitlist",
  subheading = "Be the first to know when we launch. Early access for waitlist members.",
  compact = false,
}: WaitlistFormProps) {
  const [email, setEmail] = useState("");
  const [name, setName] = useState("");
  const [status, setStatus] = useState<"idle" | "loading" | "success" | "error" | "exists">("idle");
  const [errorMsg, setErrorMsg] = useState("");

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!email) return;

    setStatus("loading");
    setErrorMsg("");

    try {
      const res = await fetch("/api/waitlist", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email, name: name || undefined, source }),
      });

      const data = await res.json();

      if (!res.ok) {
        setStatus("error");
        setErrorMsg(data.error || "Something went wrong.");
        return;
      }

      const isNew = !data.alreadyOnList;
      setStatus(isNew ? "success" : "exists");
      if (isNew) {
        setEmail("");
        setName("");
      }
      if (typeof window !== "undefined" && window.umami) {
        window.umami.track(`waitlist-signup-${source}`, { new: isNew });
      }
    } catch {
      setStatus("error");
      setErrorMsg("Network error. Please try again.");
    }
  }

  const isSubmitted = status === "success" || status === "exists";

  if (compact) {
    return (
      <div className="w-full max-w-md mx-auto">
        <AnimatePresence mode="wait">
          {isSubmitted ? (
            <motion.div
              key="done"
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              className="flex items-center gap-2 justify-center py-3"
            >
              <svg className="w-5 h-5 text-emerald-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
              <span className="text-sm text-emerald-400 font-medium">
                {status === "exists" ? "You're already on the list!" : "You're in! We'll be in touch."}
              </span>
            </motion.div>
          ) : (
            <motion.form
              key="form"
              onSubmit={handleSubmit}
              className="flex gap-2"
            >
              <input
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                placeholder="Enter your email"
                required
                className="flex-1 px-4 py-3 rounded-xl glass-subtle text-foreground placeholder:text-muted-foreground/50 focus:outline-none focus:border-cyan/50 focus:ring-1 focus:ring-cyan/30 text-sm"
              />
              <button
                type="submit"
                disabled={status === "loading"}
                className="px-6 py-3 rounded-xl bg-gradient-to-r from-cyan-500 to-blue-500 text-white font-semibold text-sm hover:from-cyan-600 hover:to-blue-600 transition-all glass-cta shadow-lg shadow-cyan-500/20 hover:shadow-cyan-500/40 disabled:opacity-60 disabled:cursor-not-allowed whitespace-nowrap"
              >
                {status === "loading" ? (
                  <svg className="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24">
                    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                  </svg>
                ) : (
                  "Join"
                )}
              </button>
            </motion.form>
          )}
        </AnimatePresence>
        {status === "error" && (
          <p className="text-red-400 text-xs mt-2 text-center">{errorMsg}</p>
        )}
      </div>
    );
  }

  return (
    <div className="w-full max-w-lg mx-auto">
      <div className="text-center mb-6">
        <h3 className="text-2xl font-bold mb-2">{heading}</h3>
        <p className="text-sm text-muted-foreground">{subheading}</p>
      </div>

      <AnimatePresence mode="wait">
        {isSubmitted ? (
          <motion.div
            key="done"
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            className="p-8 rounded-2xl border border-emerald-500/20 bg-emerald-500/[0.06] backdrop-blur-xl text-center"
          >
            <svg className="w-12 h-12 text-emerald-400 mx-auto mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <p className="text-lg font-semibold text-emerald-400 mb-1">
              {status === "exists" ? "You're already on the list!" : "You're in!"}
            </p>
            <p className="text-sm text-muted-foreground">
              {status === "exists"
                ? "We already have your email. Stay tuned for updates."
                : "We'll notify you as soon as we're ready. Thanks for your interest."}
            </p>
          </motion.div>
        ) : (
          <motion.form
            key="form"
            onSubmit={handleSubmit}
            className="space-y-3"
          >
            <div className="flex gap-3">
              <input
                type="text"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="Name (optional)"
                className="flex-1 px-4 py-3 rounded-xl glass-subtle text-foreground placeholder:text-muted-foreground/50 focus:outline-none focus:border-cyan/50 focus:ring-1 focus:ring-cyan/30 text-sm"
              />
              <input
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                placeholder="you@example.com"
                required
                className="flex-1 px-4 py-3 rounded-xl glass-subtle text-foreground placeholder:text-muted-foreground/50 focus:outline-none focus:border-cyan/50 focus:ring-1 focus:ring-cyan/30 text-sm"
              />
            </div>
            <button
              type="submit"
              disabled={status === "loading"}
              className="w-full px-6 py-3 rounded-xl bg-gradient-to-r from-cyan-500 to-blue-500 text-white font-semibold hover:from-cyan-600 hover:to-blue-600 transition-all glass-cta shadow-lg shadow-cyan-500/20 hover:shadow-cyan-500/40 disabled:opacity-60 disabled:cursor-not-allowed"
            >
              {status === "loading" ? (
                <span className="flex items-center justify-center gap-2">
                  <svg className="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24">
                    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                  </svg>
                  Joining...
                </span>
              ) : (
                "Join the Waitlist"
              )}
            </button>
            {status === "error" && (
              <p className="text-red-400 text-sm text-center">{errorMsg}</p>
            )}
            <p className="text-xs text-muted-foreground/50 text-center">
              No spam, ever. Unsubscribe anytime.
            </p>
          </motion.form>
        )}
      </AnimatePresence>
    </div>
  );
}

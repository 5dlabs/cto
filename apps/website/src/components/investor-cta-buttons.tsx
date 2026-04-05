"use client";

import { useState, useCallback } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { X } from "lucide-react";

import { INVESTOR_MORGAN_AGENT_ID } from "@/lib/morgan-agents";
import { LemonSliceWidget } from "@/components/cto/lemon-slice-widget";

const primaryClass =
  "px-8 py-4 rounded-lg border border-cyan/30 bg-cyan/10 backdrop-blur-sm text-foreground font-semibold text-lg hover:border-cyan/50 hover:bg-cyan/20 transition-colors text-center inline-block cursor-pointer";

const secondaryClass =
  "px-8 py-4 rounded-lg border border-border/40 bg-card/30 backdrop-blur-sm text-foreground font-semibold text-lg hover:border-border/60 hover:bg-card/50 transition-colors text-center inline-block cursor-pointer";

export function InvestorCtaButtons() {
  const [showMorgan, setShowMorgan] = useState(false);

  const closeMorgan = useCallback(() => setShowMorgan(false), []);

  return (
    <>
      <div className="flex flex-col sm:flex-row justify-center gap-4">
        <motion.a
          href="https://cal.com/jonathon-fritz-2uhdqe/discovery"
          target="_blank"
          rel="noopener noreferrer"
          className={primaryClass}
          whileHover={{ scale: 1.02 }}
          whileTap={{ scale: 0.98 }}
          transition={{ duration: 0.2 }}
          initial={false}
        >
          Schedule a Call
        </motion.a>
        <motion.button
          onClick={() => setShowMorgan(true)}
          className={secondaryClass}
          whileHover={{ scale: 1.02 }}
          whileTap={{ scale: 0.98 }}
          transition={{ duration: 0.2 }}
          initial={false}
        >
          Talk to Morgan
        </motion.button>
        <motion.a
          href="https://5dlabs.ai/founder"
          className={secondaryClass}
          whileHover={{ scale: 1.02 }}
          whileTap={{ scale: 0.98 }}
          transition={{ duration: 0.2 }}
          initial={false}
        >
          Meet the Founder
        </motion.a>
      </div>

      {/* Morgan modal overlay */}
      <AnimatePresence>
        {showMorgan && (
          <motion.div
            className="fixed inset-0 z-[100] flex items-center justify-center p-4"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.25 }}
          >
            {/* Backdrop */}
            <motion.div
              className="absolute inset-0 bg-black/60 backdrop-blur-sm"
              onClick={closeMorgan}
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
            />

            {/* Modal card */}
            <motion.div
              className="relative w-full max-w-md rounded-2xl border border-border/50 bg-background/95 backdrop-blur-xl shadow-2xl overflow-hidden"
              style={{ height: "min(580px, 85vh)" }}
              initial={{ opacity: 0, scale: 0.92, y: 20 }}
              animate={{ opacity: 1, scale: 1, y: 0 }}
              exit={{ opacity: 0, scale: 0.92, y: 20 }}
              transition={{ duration: 0.3, ease: [0.25, 0.4, 0, 1] }}
            >
              {/* Header */}
              <div className="flex items-center justify-between px-5 py-3 border-b border-border/30">
                <div className="flex items-center gap-2">
                  <span className="size-2 rounded-full bg-cyan animate-pulse" />
                  <span className="text-sm font-semibold text-foreground">Morgan</span>
                  <span className="text-xs text-muted-foreground">Investor Relations</span>
                </div>
                <button
                  type="button"
                  onClick={closeMorgan}
                  className="rounded-lg p-1.5 text-muted-foreground hover:text-foreground hover:bg-white/10 transition-colors"
                  aria-label="Close Morgan"
                >
                  <X className="size-4" />
                </button>
              </div>

              {/* Widget body */}
              <div className="w-full" style={{ height: "calc(100% - 48px)" }}>
                <LemonSliceWidget
                  agentId={INVESTOR_MORGAN_AGENT_ID}
                  initialState="active"
                  inline
                  className="w-full h-full"
                  customActiveWidth={400}
                  customActiveHeight={520}
                  autoStartConversation={false}
                  showStartTalkingButton
                />
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </>
  );
}

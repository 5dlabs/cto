"use client";

import { motion } from "framer-motion";

const primaryClass =
  "px-8 py-4 rounded-lg border border-cyan/30 bg-cyan/10 backdrop-blur-sm text-foreground font-semibold text-lg hover:border-cyan/50 hover:bg-cyan/20 transition-colors text-center inline-block";

const secondaryClass =
  "px-8 py-4 rounded-lg border border-border/40 bg-card/30 backdrop-blur-sm text-foreground font-semibold text-lg hover:border-border/60 hover:bg-card/50 transition-colors text-center inline-block";

export function InvestorCtaButtons() {
  return (
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
      <motion.a
        href="https://5dlabs.ai/cto/morgan"
        className={secondaryClass}
        whileHover={{ scale: 1.02 }}
        whileTap={{ scale: 0.98 }}
        transition={{ duration: 0.2 }}
        initial={false}
      >
        Talk to Morgan
      </motion.a>
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
  );
}

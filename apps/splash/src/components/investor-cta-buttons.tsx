"use client";

import { motion } from "framer-motion";

const buttonClass =
  "px-8 py-4 rounded-lg border border-cyan/20 bg-cyan/10 backdrop-blur-sm text-foreground font-semibold text-lg hover:border-cyan/40 hover:bg-cyan/15 transition-colors text-center inline-block";

export function InvestorCtaButtons() {
  return (
    <div className="flex flex-col sm:flex-row justify-center gap-4">
      <motion.a
        href="/5dlabs-investor-one-pager.pdf"
        download
        className={buttonClass}
        whileHover={{ scale: 1.02 }}
        whileTap={{ scale: 0.98 }}
        transition={{ duration: 0.2 }}
        initial={false}
      >
        Download One-Pager (PDF)
      </motion.a>
      <motion.a
        href="https://cal.com/jonathon-fritz-2uhdqe/discovery"
        target="_blank"
        rel="noopener noreferrer"
        className={buttonClass}
        whileHover={{ scale: 1.02 }}
        whileTap={{ scale: 0.98 }}
        transition={{ duration: 0.2 }}
        initial={false}
      >
        Schedule a Call
      </motion.a>
      <motion.a
        href="/founder"
        className={buttonClass}
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

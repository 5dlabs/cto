"use client";

import { motion } from "framer-motion";
import Image from "next/image";

export function Header() {
  return (
    <motion.header
      initial={{ opacity: 0, y: -10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.8, ease: "easeOut" }}
      className="fixed top-0 left-0 right-0 z-50 px-6 py-4 bg-background/80 backdrop-blur-sm"
    >
      <div className="max-w-6xl mx-auto flex items-center justify-between">
        {/* Logo */}
        <a href="/" className="flex items-center gap-3 group">
          <Image
            src="/5dlabs-logo-header-v2.png"
            alt="5D Labs"
            width={140}
            height={140}
            className="opacity-95 group-hover:opacity-100 transition-opacity"
          />
        </a>

        {/* Nav links - can be expanded later */}
        <nav className="hidden sm:flex items-center gap-6">
          <a
            href="#agents"
            className="text-sm text-muted-foreground hover:text-foreground transition-colors"
          >
            Team
          </a>
          <a
            href="#infrastructure"
            className="text-sm text-muted-foreground hover:text-foreground transition-colors"
          >
            Infrastructure
          </a>
          <a
            href="#platform"
            className="text-sm text-muted-foreground hover:text-foreground transition-colors"
          >
            Platform
          </a>
          <a
            href="https://github.com/5dlabs"
            target="_blank"
            rel="noopener noreferrer"
            className="text-sm text-muted-foreground hover:text-foreground transition-colors"
          >
            GitHub
          </a>
          <a
            href="https://cto.5dlabs.ai/"
            className="px-4 py-2 rounded-lg bg-gradient-to-r from-cyan-500 to-blue-500 text-white font-medium text-sm hover:from-cyan-600 hover:to-blue-600 transition-all shadow-lg shadow-cyan-500/20 hover:shadow-cyan-500/40"
          >
            Start Now
          </a>
        </nav>
      </div>
    </motion.header>
  );
}

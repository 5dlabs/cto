"use client";

import Image from "next/image";
import { motion } from "framer-motion";

export function MorganHeroImage() {
  return (
    <motion.div
      className="relative aspect-square max-w-[280px] mx-auto mb-8"
      initial={{ opacity: 0, scale: 0.96 }}
      whileInView={{ opacity: 1, scale: 1 }}
      viewport={{ once: true, margin: "-50px" }}
      transition={{ duration: 0.5, ease: "easeOut" }}
    >
      {/* Glow + pulse container with soft edge vignette */}
      <motion.div
        className="relative w-full h-full rounded-2xl overflow-hidden"
        animate={{
          boxShadow: [
            "0 0 40px -8px rgba(34, 211, 238, 0.35), 0 0 60px -12px rgba(236, 72, 153, 0.2)",
            "0 0 50px -6px rgba(34, 211, 238, 0.45), 0 0 70px -8px rgba(236, 72, 153, 0.3)",
            "0 0 40px -8px rgba(34, 211, 238, 0.35), 0 0 60px -12px rgba(236, 72, 153, 0.2)",
          ],
        }}
        transition={{
          duration: 3,
          repeat: Infinity,
          ease: "easeInOut",
        }}
      >
        {/* Image with soft vignette fade at edges */}
        <div
          className="absolute inset-0 rounded-2xl"
          style={{
            maskImage: "radial-gradient(ellipse 90% 90% at 50% 50%, black 70%, transparent 100%)",
            WebkitMaskImage: "radial-gradient(ellipse 90% 90% at 50% 50%, black 70%, transparent 100%)",
          }}
        >
          <Image
            src="/agents/morgan-hero.png?v=20260318"
            alt="Morgan — your control agent"
            fill
            className="object-contain object-center"
            sizes="280px"
          />
        </div>
        {/* Gradient border ring */}
        <div
          className="absolute inset-0 rounded-2xl pointer-events-none"
          style={{
            boxShadow: "inset 0 0 0 2px rgba(34, 211, 238, 0.4), inset 0 0 0 4px rgba(236, 72, 153, 0.15)",
          }}
        />
      </motion.div>
    </motion.div>
  );
}

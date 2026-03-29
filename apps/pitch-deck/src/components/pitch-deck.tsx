"use client";

import Image from "next/image";
import Link from "next/link";
import { Orbitron } from "next/font/google";
import { useCallback, useEffect, useRef, useState } from "react";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { motion } from "framer-motion";
import { DeckExportControls } from "@/components/deck-export-controls";
import { slides } from "@/lib/deck-content";
import { cn } from "@/lib/utils";

/** Eurostile-adjacent display sans; closest practical match to typical 3D-render wordmarks (raster is not identifiable to exact font name). */
const wordmarkDisplay = Orbitron({
  subsets: ["latin"],
  weight: ["600"],
  display: "swap",
});

function scrollToIndex(container: HTMLDivElement | null, index: number) {
  if (!container) return;
  const slide = container.querySelector(
    `[data-slide-index="${index}"]`,
  ) as HTMLElement | null;
  slide?.scrollIntoView({ behavior: "smooth", block: "start" });
}

export function PitchDeck() {
  const scrollerRef = useRef<HTMLDivElement>(null);
  const [active, setActive] = useState(0);

  const onScroll = useCallback(() => {
    const el = scrollerRef.current;
    if (!el) return;
    const { top, height } = el.getBoundingClientRect();
    const mid = top + height * 0.35;
    let best = 0;
    let bestDist = Infinity;
    slides.forEach((_, i) => {
      const slide = el.querySelector(
        `[data-slide-index="${i}"]`,
      ) as HTMLElement | null;
      if (!slide) return;
      const r = slide.getBoundingClientRect();
      const c = r.top + r.height / 2;
      const d = Math.abs(c - mid);
      if (d < bestDist) {
        bestDist = d;
        best = i;
      }
    });
    setActive(best);
  }, []);

  useEffect(() => {
    const el = scrollerRef.current;
    if (!el) return;
    el.addEventListener("scroll", onScroll, { passive: true });
    return () => el.removeEventListener("scroll", onScroll);
  }, [onScroll]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "ArrowRight" || e.key === "PageDown" || e.key === " ") {
        e.preventDefault();
        const next = Math.min(active + 1, slides.length - 1);
        scrollToIndex(scrollerRef.current, next);
      } else if (e.key === "ArrowLeft" || e.key === "PageUp") {
        e.preventDefault();
        const prev = Math.max(active - 1, 0);
        scrollToIndex(scrollerRef.current, prev);
      } else if (e.key === "Home") {
        scrollToIndex(scrollerRef.current, 0);
      } else if (e.key === "End") {
        scrollToIndex(scrollerRef.current, slides.length - 1);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [active]);

  return (
    <>
      <div
        ref={scrollerRef}
        className="deck-scroll relative z-10"
        tabIndex={0}
        role="region"
        aria-label="Pitch deck slides"
      >
        {slides.map((slide, i) => {
          const isHero = slide.layout === "hero";
          const isImpact = slide.layout === "impact";
          return (
          <section
            key={slide.id}
            data-slide-index={i}
            className={cn(
              "deck-slide relative flex flex-col justify-center px-6 py-16 sm:px-10 md:px-16 lg:px-24",
              isHero && "py-12 sm:py-16",
            )}
          >
            <div className="mx-auto w-full max-w-6xl">
              <motion.div
                initial={{ opacity: 0, y: 12 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true, margin: "-20% 0px" }}
                transition={{ duration: 0.35 }}
                className={cn(
                  "rounded-2xl px-6 py-8 text-lg leading-relaxed sm:px-10 sm:py-10 md:text-xl md:leading-relaxed print:!translate-y-0 print:!opacity-100 print:shadow-none",
                  isHero
                    ? "premium-shell border border-white/10 px-8 py-10 sm:px-12 sm:py-12"
                    : "glass-card",
                  isImpact && "border border-primary/15 bg-gradient-to-br from-primary/5 via-transparent to-[oklch(0.7_0.25_320)]/10",
                )}
              >
                {isHero ? (
                  <div className="mb-8 flex w-full flex-col items-center">
                    {/* Raster is alpha-cropped (no huge transparent pad); HTML lines read as one lockup */}
                    <div className="flex w-full flex-col items-center">
                      <Image
                        src="/5dlabs-logo-3d.png"
                        alt="5D Labs — 5D mark and portal"
                        width={303}
                        height={272}
                        className="block h-auto w-[min(100%,12.75rem)] sm:w-[min(100%,14.45rem)] drop-shadow-[0_0_32px_oklch(0.75_0.18_195_/_0.28)]"
                        priority
                      />
                      <p
                        className={cn(
                          wordmarkDisplay.className,
                          "mt-1.5 text-center text-[1.1rem] uppercase leading-none tracking-[0.14em] text-cyan-200/95 sm:mt-2 sm:text-[1.2375rem] md:text-[1.375rem]",
                        )}
                      >
                        5D LABS
                      </p>
                    </div>
                    <span className="mt-2.5 text-center font-mono text-xs uppercase tracking-[0.3em] text-muted-foreground sm:mt-3 sm:text-sm">
                      AI-native venture studio
                    </span>
                  </div>
                ) : null}
                {!isHero ? (
                  <p
                    className={cn(
                      "font-mono uppercase tracking-[0.18em] text-muted-foreground",
                      isImpact ? "text-xs sm:text-sm" : "text-sm sm:text-base",
                    )}
                  >
                    {String(i + 1).padStart(2, "0")} · {slide.label}
                  </p>
                ) : null}
                {slide.eyebrow ? (
                  <p
                    className={cn(
                      "font-semibold uppercase tracking-[0.16em] text-primary",
                      isHero
                        ? "mb-4 text-center text-sm sm:text-base"
                        : "mb-3 mt-2 text-sm sm:text-base",
                    )}
                  >
                    {slide.eyebrow}
                  </p>
                ) : null}
                <h1
                  className={cn(
                    "text-balance font-semibold tracking-tight",
                    isHero &&
                      "gradient-text text-center text-4xl leading-[1.08] sm:text-5xl md:text-6xl lg:text-7xl",
                    isImpact &&
                      cn(
                        "text-4xl leading-[1.12] text-foreground sm:text-5xl md:text-6xl lg:text-[3.25rem]",
                        slide.eyebrow
                          ? "mt-[calc(0.5rem+2pt)]"
                          : "mt-8 sm:mt-10",
                      ),
                    !isHero &&
                      !isImpact &&
                      cn(
                        "text-3xl text-foreground sm:text-4xl md:text-5xl lg:text-6xl",
                        slide.eyebrow
                          ? "mt-[calc(1rem+2pt)]"
                          : "mt-9 sm:mt-11",
                      ),
                  )}
                >
                  {slide.headline}
                </h1>
                {slide.subhead ? (
                  <p
                    className={cn(
                      "text-pretty text-muted-foreground",
                      isHero
                        ? "mx-auto mt-6 max-w-3xl text-center text-xl leading-relaxed sm:text-2xl"
                        : "mt-5 text-xl leading-relaxed sm:text-2xl",
                      isImpact && "mt-5 text-foreground/90",
                    )}
                  >
                    {slide.subhead}
                  </p>
                ) : null}
                {slide.stats?.length ? (
                  <div className="mt-10 grid grid-cols-2 gap-4 sm:grid-cols-4 sm:gap-5">
                    {slide.stats.map((s) => (
                      <div
                        key={s.label}
                        className="premium-chip rounded-xl px-4 py-4 text-center sm:px-5 sm:py-5"
                      >
                        <div className="text-3xl font-semibold tabular-nums tracking-tight text-foreground sm:text-4xl">
                          {s.value}
                        </div>
                        <div className="mt-2 text-sm leading-snug text-muted-foreground sm:text-base">
                          {s.label}
                        </div>
                      </div>
                    ))}
                  </div>
                ) : null}
                {slide.bullets?.length ? (
                  <ul
                    className={cn(
                      "space-y-4 text-pretty text-foreground/95",
                      !slide.subhead && !slide.stats?.length
                        ? "mt-[4.5rem] sm:mt-[5rem] md:mt-[5.5rem]"
                        : "mt-8",
                      isImpact
                        ? "text-xl leading-relaxed sm:text-2xl md:leading-relaxed"
                        : "text-lg leading-relaxed sm:text-xl md:leading-relaxed",
                    )}
                  >
                    {slide.bullets.map((b, bi) => (
                      <li key={`${slide.id}-b${bi}`} className="flex gap-4">
                        <span className="mt-3 h-2 w-2 shrink-0 rounded-full bg-primary shadow-[0_0_10px_oklch(0.75_0.18_195_/_0.45)]" />
                        <span>{b}</span>
                      </li>
                    ))}
                  </ul>
                ) : null}
                {slide.table ? (
                  <div className="mt-8 overflow-x-auto rounded-xl border border-border/80">
                    <table className="w-full min-w-[36rem] text-left text-base md:text-lg">
                      <thead>
                        <tr className="border-b border-border bg-secondary/40">
                          {slide.table.headers.map((h) => (
                            <th
                              key={h}
                              className="px-4 py-3 font-medium text-muted-foreground md:px-5 md:py-4"
                            >
                              {h}
                            </th>
                          ))}
                        </tr>
                      </thead>
                      <tbody>
                        {slide.table.rows.map((row, ri) => (
                          <tr
                            key={`${slide.id}-r${ri}`}
                            className="border-b border-border/60 last:border-0"
                          >
                            {row.map((cell, ci) => (
                              <td
                                key={`${slide.id}-r${ri}-c${ci}`}
                                className="px-4 py-3 text-foreground/95 md:px-5 md:py-3.5"
                              >
                                {cell}
                              </td>
                            ))}
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                ) : null}
                {slide.callout ? (
                  <p
                    className={cn(
                      "mt-8 rounded-lg border border-primary/25 bg-primary/10 px-5 py-4 leading-relaxed text-foreground/95 md:px-6 md:py-5",
                      "text-lg sm:text-xl md:text-2xl md:leading-snug",
                      isHero && "text-center",
                    )}
                  >
                    {slide.callout}
                  </p>
                ) : null}
                {slide.cta ? (
                  <div
                    className={cn(
                      "mt-8 flex flex-wrap items-center gap-4",
                      isHero && "justify-center",
                    )}
                  >
                    <a
                      href={slide.cta.href}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="inline-flex items-center gap-2 rounded-lg bg-gradient-to-r from-cyan-500 to-blue-600 px-7 py-3.5 text-base font-semibold text-white shadow-lg shadow-cyan-500/20 transition hover:from-cyan-600 hover:to-blue-700 print:hidden"
                    >
                      {slide.cta.label}
                    </a>
                    <span className="hidden font-mono text-sm text-muted-foreground print:inline">
                      {slide.cta.label}: {slide.cta.href}
                    </span>
                  </div>
                ) : null}
                {isHero ? (
                  <p className="mt-8 text-center font-mono text-xs uppercase tracking-[0.22em] text-muted-foreground/80 sm:text-sm">
                    {String(i + 1).padStart(2, "0")} · {slide.label}
                  </p>
                ) : null}
                {slide.footnote ? (
                  <p className="mt-8 whitespace-pre-line font-mono text-sm leading-relaxed text-muted-foreground sm:text-base">
                    {slide.footnote}
                  </p>
                ) : null}
              </motion.div>
            </div>
          </section>
          );
        })}
      </div>

      <div
        className={cn(
          "deck-chrome fixed bottom-0 left-0 right-0 z-20 flex flex-col backdrop-blur-md",
          "border-t border-border/80 bg-background/80",
        )}
      >
        <div className="flex items-center gap-2 px-3 py-2.5 sm:gap-3 sm:px-4 sm:py-3">
          <div className="flex min-w-0 flex-1 items-center justify-start gap-2">
            <button
              type="button"
              aria-label="Previous slide"
              className="glass-badge inline-flex size-12 shrink-0 items-center justify-center rounded-lg text-foreground transition hover:bg-white/10"
              onClick={() =>
                scrollToIndex(scrollerRef.current, Math.max(active - 1, 0))
              }
            >
              <ChevronLeft className="size-6" />
            </button>
            <button
              type="button"
              aria-label="Next slide"
              className="glass-badge inline-flex size-12 shrink-0 items-center justify-center rounded-lg text-foreground transition hover:bg-white/10"
              onClick={() =>
                scrollToIndex(
                  scrollerRef.current,
                  Math.min(active + 1, slides.length - 1),
                )
              }
            >
              <ChevronRight className="size-6" />
            </button>
          </div>
          <div className="shrink-0 px-1 font-mono text-xs text-muted-foreground sm:px-2 sm:text-sm md:text-base">
            {active + 1} / {slides.length}
          </div>
          <div className="flex min-w-0 flex-1 justify-end">
            <DeckExportControls />
          </div>
        </div>
        <nav
          className="flex flex-wrap items-center justify-center gap-x-2 gap-y-1 border-t border-border/50 px-3 py-2 text-center sm:gap-x-3"
          aria-label="Legal and site identity"
        >
          <span className="text-[0.65rem] font-medium uppercase tracking-[0.12em] text-muted-foreground sm:text-xs">
            5D Labs · Investor deck
          </span>
          <span className="text-[0.65rem] text-muted-foreground/50 sm:text-xs" aria-hidden>
            ·
          </span>
          <Link
            href="/privacy/"
            className="text-[0.65rem] text-muted-foreground underline underline-offset-2 transition hover:text-foreground sm:text-xs"
          >
            Privacy Policy
          </Link>
          <span className="text-[0.65rem] text-muted-foreground/50 sm:text-xs" aria-hidden>
            ·
          </span>
          <Link
            href="/terms/"
            className="text-[0.65rem] text-muted-foreground underline underline-offset-2 transition hover:text-foreground sm:text-xs"
          >
            Terms of Service
          </Link>
        </nav>
      </div>
    </>
  );
}

"use client";

import Script from "next/script";
import { useEffect, useRef } from "react";

import { PRODUCT_MORGAN_AGENT_ID } from "@/lib/morgan-agents";
const LEMON_SLICE_SCRIPT =
  "https://unpkg.com/@lemonsliceai/lemon-slice-widget@1.0.27/dist/index.js";

interface LemonSliceWidgetProps {
  agentId?: string;
  initialState?: "active" | "minimized";
  inline?: boolean;
  className?: string;
  customActiveWidth?: number;
  customActiveHeight?: number;
  customMinimizedWidth?: number;
  customMinimizedHeight?: number;
  /**
   * When true (default), opens the mic shortly after load so the session starts immediately.
   * When false, skips that — the host’s looping “first message” can play until the user taps or speaks (fewer tokens on idle views).
   */
  autoStartConversation?: boolean;
  showStartTalkingButton?: boolean;
}

interface LemonSliceWidgetElement extends HTMLElement {
  mute?: () => Promise<void>;
  unmute?: () => Promise<void>;
  canUnmute?: () => boolean;
  isMuted?: () => boolean;
  micOn?: () => Promise<void>;
}

export function LemonSliceWidget({
  agentId = PRODUCT_MORGAN_AGENT_ID,
  initialState = "active",
  inline = true,
  className,
  customActiveWidth,
  customActiveHeight,
  customMinimizedWidth,
  customMinimizedHeight,
  autoStartConversation = true,
  showStartTalkingButton = false,
}: LemonSliceWidgetProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const startConversationRef = useRef<() => Promise<void>>(async () => {});

  useEffect(() => {
    const containerEl = containerRef.current;
    if (!containerEl) return;

    const el = document.createElement("lemon-slice-widget") as LemonSliceWidgetElement;
    el.setAttribute("agent-id", agentId);
    el.setAttribute("initial-state", initialState);
    // Keep the widget in active mode so users land directly in conversation UI.
    if (initialState === "active") {
      el.setAttribute("controlled-widget-state", "active");
    }
    if (inline) el.setAttribute("inline", "true");

    // Ensure the widget itself is centered within the container flex box.
    el.style.display = "block";
    el.style.margin = "0 auto";

    // Morgan avatar is 9:14 portrait (morgan-image.md). LemonSlice center-crops;
    // wrong aspect ratio cuts off the head. Always pass 9:14 to avoid cropping.
    const AVATAR_ASPECT = 14 / 9; // height / width

    if (customMinimizedWidth != null) {
      el.setAttribute("custom-minimized-width", String(customMinimizedWidth));
    }
    if (customMinimizedHeight != null) {
      el.setAttribute("custom-minimized-height", String(customMinimizedHeight));
    }

    containerEl.innerHTML = "";
    containerEl.appendChild(el);

    const widthCap = customActiveWidth ?? 400;
    const heightCap = customActiveHeight ?? Math.floor(widthCap * AVATAR_ASPECT);

    const applyActiveSizing = () => {
      const availableWidth = containerEl.clientWidth;
      const availableHeight = containerEl.clientHeight;
      if (availableWidth <= 0 || availableHeight <= 0) return;

      const minW = 280;
      const minH = Math.floor(minW * AVATAR_ASPECT);

      // Fit by width first, then constrain height to container; keep 9:14.
      let activeW = Math.max(minW, Math.floor(Math.min(widthCap, availableWidth)));
      let activeH = Math.floor(activeW * AVATAR_ASPECT);

      if (activeH > availableHeight) {
        activeH = Math.max(minH, Math.floor(Math.min(heightCap, availableHeight)));
        activeW = Math.floor(activeH / AVATAR_ASPECT);
      }

      el.setAttribute("custom-active-width", String(activeW));
      el.setAttribute("custom-active-height", String(activeH));
    };

    applyActiveSizing();
    const resizeObserver = new ResizeObserver(() => applyActiveSizing());
    resizeObserver.observe(containerEl);

    let conversationStarted = false;
    const startConversation = async () => {
      if (!autoStartConversation || conversationStarted) {
        return;
      }
      try {
        if (typeof el.micOn !== "function") {
          return;
        }
        await el.micOn();
        conversationStarted = true;
      } catch {
        // Keep retries enabled until a successful micOn().
      }
    };
    startConversationRef.current = startConversation;

    // Speaker was on by default before; widget/lib may now default to muted.
    // Explicitly unmute once the widget is ready so Morgan's voice plays.
    const tryUnmute = () => {
      try {
        if (typeof el.unmute === "function" && el.canUnmute?.() && el.isMuted?.()) {
          void el.unmute();
        }
      } catch {
        // ignore
      }
    };
    void customElements.whenDefined("lemon-slice-widget").then(() => {
      // Try immediately for browsers that already have media permission.
      void startConversation();
      // Retry because widget internals can finish initialization after define().
      setTimeout(() => void startConversation(), 500);
      setTimeout(() => void startConversation(), 1500);
      setTimeout(() => void startConversation(), 3000);
      setTimeout(tryUnmute, 300);
      setTimeout(tryUnmute, 1200);
    });

    // Fallback: first user interaction anywhere should immediately start mic/room.
    const handleFirstInteraction = () => {
      void startConversation().finally(() => {
        containerEl.removeEventListener("pointerdown", handleFirstInteraction);
        containerEl.removeEventListener("keydown", handleFirstInteraction);
        document.removeEventListener("pointerdown", handleFirstInteraction, true);
        document.removeEventListener("keydown", handleFirstInteraction, true);
      });
    };
    containerEl.addEventListener("pointerdown", handleFirstInteraction);
    containerEl.addEventListener("keydown", handleFirstInteraction);
    document.addEventListener("pointerdown", handleFirstInteraction, true);
    document.addEventListener("keydown", handleFirstInteraction, true);

    return () => {
      resizeObserver.disconnect();
      containerEl.removeEventListener("pointerdown", handleFirstInteraction);
      containerEl.removeEventListener("keydown", handleFirstInteraction);
      document.removeEventListener("pointerdown", handleFirstInteraction, true);
      document.removeEventListener("keydown", handleFirstInteraction, true);
      containerEl.removeChild(el);
    };
  }, [agentId, initialState, inline, customActiveWidth, customActiveHeight, customMinimizedWidth, customMinimizedHeight, autoStartConversation]);

  return (
    <>
      <Script
        src={LEMON_SLICE_SCRIPT}
        strategy="afterInteractive"
        type="module"
      />
      <div className="w-full h-full flex flex-col items-center gap-3">
        {showStartTalkingButton ? (
          <button
            type="button"
            onClick={() => void startConversationRef.current()}
            className="rounded-full px-4 py-2 text-sm font-semibold text-white bg-gradient-to-r from-violet-500 via-indigo-500 to-cyan-500 shadow-[0_0_18px_rgba(99,102,241,0.35)] hover:brightness-110 transition-all"
          >
            Start talking now
          </button>
        ) : null}
        <div ref={containerRef} className={className} />
      </div>
    </>
  );
}

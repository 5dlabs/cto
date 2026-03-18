"use client";

import Script from "next/script";
import { useEffect, useRef } from "react";

const MORGAN_AGENT_ID = "agent_0b8ca791bd37c632";
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
}

export function LemonSliceWidget({
  agentId = MORGAN_AGENT_ID,
  initialState = "active",
  inline = true,
  className,
  customActiveWidth,
  customActiveHeight,
  customMinimizedWidth,
  customMinimizedHeight,
}: LemonSliceWidgetProps) {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    const el = document.createElement("lemon-slice-widget");
    el.setAttribute("agent-id", agentId);
    el.setAttribute("initial-state", initialState);
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

    containerRef.current.innerHTML = "";
    containerRef.current.appendChild(el);

    const widthCap = customActiveWidth ?? 400;
    const heightCap = customActiveHeight ?? Math.floor(widthCap * AVATAR_ASPECT);

    const applyActiveSizing = () => {
      const containerEl = containerRef.current;
      if (!containerEl) return;
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
    resizeObserver.observe(containerRef.current);

    return () => {
      resizeObserver.disconnect();
      containerRef.current?.removeChild(el);
    };
  }, [agentId, initialState, inline, customActiveWidth, customActiveHeight, customMinimizedWidth, customMinimizedHeight]);

  return (
    <>
      <Script
        src={LEMON_SLICE_SCRIPT}
        strategy="lazyOnload"
        type="module"
      />
      <div ref={containerRef} className={className} />
    </>
  );
}

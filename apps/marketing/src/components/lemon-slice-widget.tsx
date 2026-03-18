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
    if (customActiveWidth != null) el.setAttribute("custom-active-width", String(customActiveWidth));
    if (customActiveHeight != null) el.setAttribute("custom-active-height", String(customActiveHeight));
    if (customMinimizedWidth != null) el.setAttribute("custom-minimized-width", String(customMinimizedWidth));
    if (customMinimizedHeight != null) el.setAttribute("custom-minimized-height", String(customMinimizedHeight));
    containerRef.current.innerHTML = "";
    containerRef.current.appendChild(el);
    return () => {
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

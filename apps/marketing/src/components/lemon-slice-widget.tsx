"use client";

import Script from "next/script";
import { useEffect, useRef } from "react";

const MORGAN_AGENT_ID = "agent_0b8ca791bd37c632";
const LEMON_SLICE_SCRIPT =
  "https://unpkg.com/@lemonsliceai/lemon-slice-widget@1.0.27/dist/index.js";

interface LemonSliceWidgetProps {
  agentId?: string;
  initialState?: "active" | "minimized";
  className?: string;
}

export function LemonSliceWidget({
  agentId = MORGAN_AGENT_ID,
  initialState = "active",
  className,
}: LemonSliceWidgetProps) {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!containerRef.current) return;
    const el = document.createElement("lemon-slice-widget");
    el.setAttribute("agent-id", agentId);
    el.setAttribute("initial-state", initialState);
    containerRef.current.innerHTML = "";
    containerRef.current.appendChild(el);
    return () => {
      containerRef.current?.removeChild(el);
    };
  }, [agentId, initialState]);

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

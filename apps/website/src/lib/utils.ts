import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export const colorMap: Record<string, { bg: string; text: string; border: string }> = {
  cyan: { bg: "bg-cyan/10", text: "text-cyan", border: "border-cyan/20" },
  purple: { bg: "bg-[oklch(0.7_0.25_320)]/10", text: "text-[oklch(0.7_0.25_320)]", border: "border-[oklch(0.7_0.25_320)]/20" },
  blue: { bg: "bg-blue-500/10", text: "text-blue-400", border: "border-blue-500/20" },
  orange: { bg: "bg-orange-500/10", text: "text-orange-400", border: "border-orange-500/20" },
  yellow: { bg: "bg-yellow-500/10", text: "text-yellow-500", border: "border-yellow-500/20" },
  emerald: { bg: "bg-emerald-500/10", text: "text-emerald-400", border: "border-emerald-500/20" },
};

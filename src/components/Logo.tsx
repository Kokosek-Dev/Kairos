import { cn } from "@/lib/utils";

/** The Zenith mark: a ring with a bright dot at its zenith. */
export function Logo({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 32 32" className={cn("shrink-0", className)} aria-hidden="true">
      <defs>
        <linearGradient id="zenith-logo" x1="4" y1="4" x2="28" y2="28" gradientUnits="userSpaceOnUse">
          <stop style={{ stopColor: "var(--accent)" }} />
          <stop offset="1" style={{ stopColor: "var(--accent-strong)" }} />
        </linearGradient>
      </defs>
      <circle cx="16" cy="18" r="10" fill="none" stroke="url(#zenith-logo)" strokeWidth="3.5" />
      <circle cx="16" cy="5.5" r="3" fill="url(#zenith-logo)" />
    </svg>
  );
}

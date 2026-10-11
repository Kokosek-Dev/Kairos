import { cn } from "@/lib/utils";

/** The Kairos mark: a K with a spark, for the right moment. It takes the accent color of the launcher. */
export function Logo({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 64 64" className={cn("shrink-0", className)} aria-hidden="true">
      <defs>
        <linearGradient id="kairos-logo" x1="14" y1="10" x2="50" y2="54" gradientUnits="userSpaceOnUse">
          <stop style={{ stopColor: "var(--accent)" }} />
          <stop offset="1" style={{ stopColor: "var(--accent-2)" }} />
        </linearGradient>
      </defs>
      <g fill="none" stroke="url(#kairos-logo)" strokeWidth="9" strokeLinecap="round" strokeLinejoin="round">
        <path d="M20 14V50" />
        <path d="M25 33L44 50" />
        <path d="M25 31L39 18" />
      </g>
      <path d="M47 3l2.4 6.6L56 12l-6.6 2.4L47 21l-2.4-6.6L38 12l6.6-2.4z" style={{ fill: "var(--foreground)" }} />
    </svg>
  );
}

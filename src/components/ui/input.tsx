import type { ComponentProps } from "react";
import { cn } from "@/lib/utils";

const FIELD =
  "w-full select-text rounded-md border bg-surface px-3 text-sm outline-none transition-colors placeholder:text-muted focus:border-accent disabled:opacity-50";

export function Input({ className, ...props }: ComponentProps<"input">) {
  return <input className={cn(FIELD, "h-10", className)} {...props} />;
}

export function Textarea({ className, ...props }: ComponentProps<"textarea">) {
  return <textarea className={cn(FIELD, "min-h-24 resize-y py-2 font-mono", className)} {...props} />;
}

export function Field({ label, hint, children }: { label: string; hint?: string; children: React.ReactNode }) {
  return (
    <div className="space-y-1.5">
      <span className="text-sm font-medium">{label}</span>
      {children}
      {hint && <p className="text-xs text-muted">{hint}</p>}
    </div>
  );
}

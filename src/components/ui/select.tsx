import type { ComponentProps } from "react";
import { cn } from "@/lib/utils";

export function Select({ className, ...props }: ComponentProps<"select">) {
  return (
    <select
      className={cn(
        "h-10 rounded-md border bg-surface px-3 text-sm outline-none transition-colors focus:border-accent disabled:opacity-50",
        className,
      )}
      {...props}
    />
  );
}

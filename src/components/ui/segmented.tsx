import { cn } from "@/lib/utils";

interface SegmentedProps<T extends string | number> {
  value: T;
  options: { value: T; label: string }[];
  onChange: (value: T) => void;
}

export function Segmented<T extends string | number>({ value, options, onChange }: SegmentedProps<T>) {
  return (
    <div className="flex gap-1 rounded-lg border bg-surface/60 p-1">
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          onClick={() => onChange(option.value)}
          className={cn(
            "h-8 flex-1 rounded-md px-3 text-xs font-medium text-muted transition-colors hover:text-foreground",
            value === option.value && "bg-brand text-accent-foreground hover:text-accent-foreground",
          )}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}

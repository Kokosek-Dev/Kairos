import { cn } from "@/lib/utils";

interface SwitchProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: string;
}

export function Switch({ checked, onChange, label }: SwitchProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      onClick={() => onChange(!checked)}
      className={cn("relative h-6 w-11 shrink-0 rounded-full border transition-colors", checked ? "border-transparent bg-brand" : "bg-surface-raised")}
    >
      <span className={cn("absolute top-0.5 size-4.5 rounded-full bg-white shadow transition-all", checked ? "left-[1.4rem]" : "left-0.5")} />
    </button>
  );
}

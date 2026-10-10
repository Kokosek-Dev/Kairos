import { cn } from "@/lib/utils";

export function AccountAvatar({ name, className }: { name: string; className?: string }) {
  return (
    <div
      className={cn(
        "grid size-9 shrink-0 place-items-center rounded-full bg-accent/20 font-semibold uppercase text-accent",
        className,
      )}
    >
      {name.slice(0, 1)}
    </div>
  );
}

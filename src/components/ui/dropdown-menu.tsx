import * as MenuPrimitive from "@radix-ui/react-dropdown-menu";
import type { ComponentProps } from "react";
import { cn } from "@/lib/utils";

export const DropdownMenu = MenuPrimitive.Root;
export const DropdownMenuTrigger = MenuPrimitive.Trigger;

export function DropdownMenuContent({ className, ...props }: ComponentProps<typeof MenuPrimitive.Content>) {
  return (
    <MenuPrimitive.Portal>
      <MenuPrimitive.Content
        align="end"
        sideOffset={6}
        className={cn(
          "glass z-50 min-w-44 rounded-lg p-1 shadow-glow data-[state=closed]:animate-out data-[state=open]:animate-in data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95",
          className,
        )}
        {...props}
      />
    </MenuPrimitive.Portal>
  );
}

interface ItemProps extends ComponentProps<typeof MenuPrimitive.Item> {
  danger?: boolean;
}

export function DropdownMenuItem({ className, danger, ...props }: ItemProps) {
  return (
    <MenuPrimitive.Item
      className={cn(
        "flex h-9 cursor-pointer items-center gap-2.5 rounded-md px-3 text-sm outline-none transition-colors data-[disabled]:pointer-events-none data-[disabled]:opacity-50",
        danger
          ? "text-danger data-[highlighted]:bg-danger/15"
          : "text-foreground data-[highlighted]:bg-surface-raised",
        className,
      )}
      {...props}
    />
  );
}

export function DropdownMenuSeparator() {
  return <MenuPrimitive.Separator className="my-1 h-px bg-border" />;
}

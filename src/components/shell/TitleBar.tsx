import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Logo } from "@/components/Logo";
import { AccountMenu } from "@/features/accounts/AccountMenu";
import { cn } from "@/lib/utils";

const appWindow = getCurrentWindow();

interface ControlProps {
  label: string;
  onClick: () => void;
  danger?: boolean;
  children: React.ReactNode;
}

function Control({ label, onClick, danger, children }: ControlProps) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      onClick={onClick}
      className={cn(
        "grid h-full w-12 place-items-center text-muted transition-colors hover:text-foreground",
        danger ? "hover:bg-danger" : "hover:bg-surface-raised",
      )}
    >
      {children}
    </button>
  );
}

export function TitleBar() {
  const { t } = useTranslation();

  return (
    <header className="relative flex h-11 shrink-0 items-center border-b bg-background/50 backdrop-blur-xl after:absolute after:inset-x-0 after:-bottom-px after:h-px after:bg-gradient-to-r after:from-transparent after:via-accent/60 after:to-transparent">
      <div data-tauri-drag-region className="flex h-full flex-1 items-center gap-2 px-4">
        <Logo className="pointer-events-none size-5 transition-transform duration-700 [div:hover>&]:rotate-[360deg]" />
        <span className="pointer-events-none text-xs font-semibold tracking-[0.25em] text-foreground/80">ZENITH</span>
      </div>
      <AccountMenu />
      <Control label={t("window.minimize")} onClick={() => void appWindow.minimize()}>
        <Minus size={16} />
      </Control>
      <Control label={t("window.maximize")} onClick={() => void appWindow.toggleMaximize()}>
        <Square size={12} />
      </Control>
      <Control label={t("window.close")} onClick={() => void appWindow.close()} danger>
        <X size={16} />
      </Control>
    </header>
  );
}

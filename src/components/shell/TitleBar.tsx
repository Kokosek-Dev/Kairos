import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X } from "lucide-react";
import { useTranslation } from "react-i18next";
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
    <header className="flex h-10 shrink-0 items-center border-b bg-background">
      <div data-tauri-drag-region className="flex h-full flex-1 items-center gap-2 px-4">
        <span className="pointer-events-none size-2.5 rounded-full bg-accent shadow-glow" />
        <span className="pointer-events-none text-xs font-semibold tracking-[0.2em] text-muted">
          ZENITH
        </span>
      </div>
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

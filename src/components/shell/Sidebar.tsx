import { motion } from "framer-motion";
import { Compass, Layers, Library, Settings, Terminal, type LucideIcon } from "lucide-react";
import { useTranslation } from "react-i18next";
import { NavLink } from "react-router-dom";
import { isBusy, useGameStore } from "@/features/launch/gameStore";
import { cn } from "@/lib/utils";

interface NavItem {
  to: string;
  labelKey: string;
  icon: LucideIcon;
}

const ITEMS: NavItem[] = [
  { to: "/", labelKey: "nav.library", icon: Library },
  { to: "/versions", labelKey: "nav.versions", icon: Layers },
  { to: "/discover", labelKey: "nav.discover", icon: Compass },
  { to: "/settings", labelKey: "nav.settings", icon: Settings },
];

function ConsoleButton() {
  const { t } = useTranslation();
  const { phase, consoleOpen, setConsoleOpen } = useGameStore();
  if (phase.kind === "idle") return null;

  return (
    <button
      type="button"
      onClick={() => setConsoleOpen(!consoleOpen)}
      className="mt-auto flex h-10 items-center gap-3 rounded-md px-3 font-medium text-muted transition-colors hover:bg-surface-raised hover:text-foreground"
    >
      <Terminal size={18} />
      {t("console.title")}
      {isBusy(phase) && <span className="ml-auto size-2 animate-pulse rounded-full bg-success" />}
    </button>
  );
}

export function Sidebar() {
  const { t } = useTranslation();

  return (
    <nav className="flex w-56 shrink-0 flex-col gap-1 border-r bg-surface/50 p-3">
      {ITEMS.map(({ to, labelKey, icon: Icon }) => (
        <NavLink
          key={to}
          to={to}
          end={to === "/"}
          className={({ isActive }) =>
            cn(
              "relative flex h-10 items-center gap-3 rounded-md px-3 font-medium text-muted transition-colors hover:text-foreground",
              isActive && "text-foreground",
            )
          }
        >
          {({ isActive }) => (
            <>
              {isActive && (
                <motion.span
                  layoutId="nav-active"
                  className="absolute inset-0 rounded-md bg-accent/15 ring-1 ring-accent/40"
                  transition={{ type: "spring", stiffness: 500, damping: 40 }}
                />
              )}
              <Icon size={18} className={cn("relative", isActive && "text-accent")} />
              <span className="relative">{t(labelKey)}</span>
            </>
          )}
        </NavLink>
      ))}
      <ConsoleButton />
    </nav>
  );
}

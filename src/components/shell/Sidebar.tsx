import { useQuery } from "@tanstack/react-query";
import { getVersion } from "@tauri-apps/api/app";
import { motion } from "framer-motion";
import { Compass, Home, Layers, Library, Settings, Terminal, type LucideIcon } from "lucide-react";
import { useTranslation } from "react-i18next";
import { NavLink } from "react-router-dom";
import { Logo } from "@/components/Logo";
import { isBusy, useGameStore } from "@/features/launch/gameStore";
import { cn } from "@/lib/utils";

interface NavItem {
  to: string;
  labelKey: string;
  icon: LucideIcon;
  /** Hue offset from the accent, so every section has its own color. */
  shift: number;
}

const ITEMS: NavItem[] = [
  { to: "/", labelKey: "nav.home", icon: Home, shift: 0 },
  { to: "/library", labelKey: "nav.library", icon: Library, shift: 50 },
  { to: "/discover", labelKey: "nav.discover", icon: Compass, shift: 110 },
  { to: "/versions", labelKey: "nav.versions", icon: Layers, shift: 170 },
  { to: "/settings", labelKey: "nav.settings", icon: Settings, shift: 230 },
];

function ConsoleButton() {
  const { t } = useTranslation();
  const { phase, consoleOpen, setConsoleOpen } = useGameStore();
  if (phase.kind === "idle") return null;

  return (
    <button
      type="button"
      onClick={() => setConsoleOpen(!consoleOpen)}
      className="flex h-10 items-center gap-3 rounded-md px-3 font-medium text-muted transition-colors hover:bg-surface-raised hover:text-foreground"
    >
      <Terminal size={18} />
      {t("console.title")}
      {isBusy(phase) && <span className="ml-auto size-2 animate-pulse rounded-full bg-success" />}
    </button>
  );
}

function Footer() {
  const version = useQuery({ queryKey: ["app-version"], queryFn: getVersion, staleTime: Infinity });
  return (
    <div className="flex items-center gap-2 px-3 pt-2 text-xs text-muted/70">
      <Logo className="size-4 opacity-70" />
      <span>Zenith {version.data ? `v${version.data}` : ""}</span>
    </div>
  );
}

export function Sidebar() {
  const { t } = useTranslation();

  return (
    <nav className="flex w-56 shrink-0 flex-col gap-1 border-r bg-surface/40 p-3 backdrop-blur-xl">
      {ITEMS.map(({ to, labelKey, icon: Icon, shift }) => (
        <NavLink
          key={to}
          to={to}
          end={to === "/"}
          style={{ ["--item" as string]: `oklch(0.78 0.15 calc(var(--accent-hue) + ${shift}))` }}
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
                  className="absolute inset-0 rounded-md shadow-[inset_0_0_0_1px_color-mix(in_oklch,var(--item)_45%,transparent)]" style={{ background: "color-mix(in oklch, var(--item) 14%, transparent)" }}
                  transition={{ type: "spring", stiffness: 500, damping: 40 }}
                />
              )}
              <Icon size={18} className="relative" style={{ color: "var(--item)", opacity: isActive ? 1 : 0.75 }} />
              <span className="relative">{t(labelKey)}</span>
            </>
          )}
        </NavLink>
      ))}
      <div className="mt-auto flex flex-col gap-1">
        <ConsoleButton />
        <Footer />
      </div>
    </nav>
  );
}

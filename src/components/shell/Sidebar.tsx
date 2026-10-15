import { useQuery } from "@tanstack/react-query";
import { getVersion } from "@tauri-apps/api/app";
import { motion } from "framer-motion";
import { BookOpen, Blocks, Compass, Home, Library, MessageSquareText, Server, Settings, Shirt, Terminal, type LucideIcon } from "lucide-react";
import { useTranslation } from "react-i18next";
import { NavLink } from "react-router-dom";
import { Logo } from "@/components/Logo";
import { InstanceIcon } from "@/features/instances/InstanceIcon";
import { useInstances } from "@/features/instances/useInstances";
import { useFeedbackStore } from "@/features/feedback/feedbackStore";
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
  { to: "/textures", labelKey: "nav.textures", icon: Shirt, shift: 170 },
  { to: "/journal", labelKey: "nav.journal", icon: BookOpen, shift: 200 },
  { to: "/servers", labelKey: "nav.servers", icon: Server, shift: 230 },
  { to: "/bedrock", labelKey: "nav.bedrock", icon: Blocks, shift: 260 },
  { to: "/settings", labelKey: "nav.settings", icon: Settings, shift: 290 },
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

function RecentInstances() {
  const { t } = useTranslation();
  const instances = useInstances();
  const recent = [...(instances.data ?? [])]
    .filter((instance) => instance.lastPlayedAt)
    .sort((a, b) => (b.lastPlayedAt ?? "").localeCompare(a.lastPlayedAt ?? ""))
    .slice(0, 5);
  if (recent.length === 0) return null;

  return (
    <div className="mt-4 flex flex-col gap-1 border-t pt-3">
      <p className="px-3 pb-1 text-[11px] font-semibold uppercase tracking-wider text-muted/70">{t("nav.recent")}</p>
      {recent.map((instance) => (
        <NavLink
          key={instance.id}
          to={`/instances/${instance.id}`}
          className="flex h-10 items-center gap-3 rounded-md px-3 text-muted transition-all hover:translate-x-1 hover:bg-surface-raised hover:text-foreground"
        >
          <InstanceIcon icon={instance.icon} instanceId={instance.id} className="size-6 rounded-md" iconSize={13} />
          <span className="truncate text-sm">{instance.name}</span>
        </NavLink>
      ))}
    </div>
  );
}

function Footer() {
  const version = useQuery({ queryKey: ["app-version"], queryFn: getVersion, staleTime: Infinity });
  const { t } = useTranslation();
  return (
    <div className="flex items-center gap-2 px-3 pt-2 text-xs text-muted/70">
      <Logo className="size-4 opacity-70" />
      <span>Kairos {version.data ? `v${version.data}` : ""}</span>
      <button
        type="button"
        aria-label={t("feedback.title")}
        title={t("feedback.title")}
        onClick={() => useFeedbackStore.getState().show()}
        className="ml-auto rounded p-1 transition-colors hover:bg-surface-raised hover:text-foreground"
      >
        <MessageSquareText size={14} />
      </button>
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
              <motion.span className="relative grid" whileHover={{ scale: 1.2, rotate: -8 }} transition={{ type: "spring", stiffness: 400, damping: 14 }}>
                <Icon size={18} style={{ color: "var(--item)", opacity: isActive ? 1 : 0.75 }} />
              </motion.span>
              <span className="relative">{t(labelKey)}</span>
            </>
          )}
        </NavLink>
      ))}
      <RecentInstances />
      <div className="mt-auto flex flex-col gap-1">
        <ConsoleButton />
        <Footer />
      </div>
    </nav>
  );
}

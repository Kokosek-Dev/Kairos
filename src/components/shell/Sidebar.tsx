import { motion } from "framer-motion";
import { Compass, Layers, Library, LogIn, Settings, Terminal, type LucideIcon } from "lucide-react";
import { useTranslation } from "react-i18next";
import { NavLink, useNavigate } from "react-router-dom";
import { AccountAvatar } from "@/features/accounts/AccountAvatar";
import { useAccounts } from "@/features/accounts/AccountsSection";
import { useSignInStore } from "@/features/accounts/signInStore";
import { Button } from "@/components/ui/button";
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

function AccountChip() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const accounts = useAccounts();
  const startSignIn = useSignInStore((state) => state.start);
  const active = accounts.data?.accounts.find((account) => account.id === accounts.data?.activeId);

  if (accounts.isPending) return <div className="skeleton mt-2 h-12 rounded-md" />;
  if (!active) {
    return (
      <Button variant="secondary" className="mt-2 w-full" onClick={() => void startSignIn()}>
        <LogIn size={16} />
        {t("accounts.signIn")}
      </Button>
    );
  }
  return (
    <button
      type="button"
      onClick={() => navigate("/settings")}
      className="mt-2 flex h-12 items-center gap-3 rounded-md border bg-surface px-3 text-left transition-colors hover:bg-surface-raised"
    >
      <AccountAvatar name={active.username} className="size-7 text-xs" />
      <span className="min-w-0 flex-1 truncate font-medium">{active.username}</span>
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
      <AccountChip />
    </nav>
  );
}

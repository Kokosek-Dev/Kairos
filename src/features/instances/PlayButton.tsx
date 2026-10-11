import { ChevronDown, Play, Square, WifiOff } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { isBusy, useGameStore } from "@/features/launch/gameStore";
import { useOfflineStore } from "@/features/launch/offlineStore";
import { useIncognito } from "@/features/incognito/useIncognito";
import { useActiveAccount } from "@/features/skins/useProfile";
import type { Instance } from "@/lib/tauri/instances";
import { cn } from "@/lib/utils";

/** Play button with a small menu for offline play, which needs an account that owns the game. */
export function PlayButton({ instance, className }: { instance: Instance; className?: string }) {
  const { t } = useTranslation();
  const { instanceId, phase, play, stop } = useGameStore();
  const askOffline = useOfflineStore((state) => state.ask);
  const { active } = useActiveAccount();
  const incognito = useIncognito();

  const thisRunning = instanceId === instance.id && isBusy(phase);
  const otherRunning = !thisRunning && isBusy(phase);

  if (thisRunning) {
    return (
      <Button size="lg" variant="secondary" className={className} onClick={() => void stop()}>
        <Square size={16} />
        {t("console.stop")}
      </Button>
    );
  }

  return (
    <div className={cn("flex", className)}>
      <Button
        size="lg"
        className={cn("flex-1", active && !incognito && "rounded-r-none")}
        disabled={otherRunning}
        onClick={() => void play(instance.id, instance.name, incognito)}
      >
        <Play size={18} />
        {t("instances.play")}
      </Button>
      {active && !incognito && (
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              size="lg"
              className="rounded-l-none border-l border-white/20 px-3"
              disabled={otherRunning}
              aria-label={t("offline.menu")}
            >
              <ChevronDown size={16} />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuItem onSelect={() => askOffline({ instanceId: instance.id, title: instance.name })}>
              <WifiOff size={15} />
              {t("offline.menuItem")}
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      )}
    </div>
  );
}

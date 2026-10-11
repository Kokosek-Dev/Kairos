import { useMutation, useQueryClient } from "@tanstack/react-query";
import { motion } from "framer-motion";
import { Copy, FolderOpen, MoreHorizontal, Settings2, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { isBusy, useGameStore } from "@/features/launch/gameStore";
import { formatPlayTime, formatRelative } from "@/lib/format";
import { duplicateInstance, openInstanceFolder, type Instance } from "@/lib/tauri/instances";
import { loaderChipStyle } from "@/lib/loaderColors";
import { cn } from "@/lib/utils";
import { InstanceIcon } from "./InstanceIcon";
import { PlayButton } from "./PlayButton";
import { findPreset } from "./presets";
import { INSTANCES_KEY } from "./useInstances";

interface InstanceCardProps {
  instance: Instance;
  index?: number;
  onDelete: (instance: Instance) => void;
}

function bannerFor(instance: Instance): string {
  const hue = instance.icon.kind === "preset" ? String(findPreset(instance.icon.key).hue) : "var(--accent-hue)";
  return `linear-gradient(180deg, oklch(0.5 0.17 ${hue} / 0.5), transparent)`;
}

function Chip({ children, style }: { children: React.ReactNode; style?: React.CSSProperties }) {
  return (
    <span
      className={cn("rounded-full px-2 py-0.5 text-[11px] font-medium", !style && "bg-surface-raised text-muted")}
      style={style}
    >
      {children}
    </span>
  );
}

export function InstanceCard({ instance, index = 0, onDelete }: InstanceCardProps) {
  const { t, i18n } = useTranslation();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { instanceId, phase } = useGameStore();

  const thisRunning = instanceId === instance.id && isBusy(phase);
  const settingsPath = `/instances/${instance.id}`;

  const duplicate = useMutation({
    mutationFn: () => duplicateInstance(instance.id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: INSTANCES_KEY }),
  });

  return (
    <motion.article
      layout
      initial={{ opacity: 0, y: 12 }}
      animate={{ opacity: 1, y: 0 }}
      whileHover={{ y: -3 }}
      transition={{ duration: 0.25, delay: Math.min(index, 8) * 0.05 }}
      className={cn(
        "glow-follow group relative flex flex-col gap-4 overflow-hidden rounded-2xl border bg-surface p-4 transition-[border-color,box-shadow] hover:border-accent/50 hover:shadow-glow",
        thisRunning && "border-success/50",
      )}
    >
      <div
        className="pointer-events-none absolute inset-x-0 top-0 h-40 opacity-70 transition-opacity group-hover:opacity-100"
        style={{ background: bannerFor(instance) }}
      />

      <div className="relative flex items-start gap-3">
        <button type="button" onClick={() => navigate(settingsPath)} className="flex min-w-0 flex-1 items-center gap-3 text-left">
          <InstanceIcon icon={instance.icon} instanceId={instance.id} className="size-16 shadow-lg ring-1 ring-white/10" iconSize={30} />
          <div className="min-w-0">
            <h3 className="truncate text-base font-semibold">{instance.name}</h3>
            <div className="mt-1.5 flex flex-wrap gap-1.5">
              <Chip>{instance.gameVersion}</Chip>
              <Chip style={loaderChipStyle(instance.loader)}>
                {t(`instances.loader.${instance.loader}`)}
                {instance.loaderVersion ? ` ${instance.loaderVersion}` : ""}
              </Chip>
            </div>
            <p className="mt-1.5 text-xs text-muted">
              {thisRunning
                ? t("instances.running")
                : instance.lastPlayedAt
                  ? t("instances.lastPlayed", { when: formatRelative(instance.lastPlayedAt, i18n.language) })
                  : t("instances.neverPlayed")}
              {instance.playTimeSecs >= 60 && ` · ${formatPlayTime(instance.playTimeSecs, i18n.language)}`}
            </p>
          </div>
        </button>

        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="sm" className="size-8 px-0" aria-label={t("instances.menu")}>
              <MoreHorizontal size={16} />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuItem onSelect={() => navigate(settingsPath)}>
              <Settings2 size={15} />
              {t("instances.settings")}
            </DropdownMenuItem>
            <DropdownMenuItem onSelect={() => void openInstanceFolder(instance.id)}>
              <FolderOpen size={15} />
              {t("instances.openFolder")}
            </DropdownMenuItem>
            <DropdownMenuItem onSelect={() => duplicate.mutate()}>
              <Copy size={15} />
              {t("instances.duplicate")}
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem danger disabled={thisRunning} onSelect={() => onDelete(instance)}>
              <Trash2 size={15} />
              {t("instances.delete")}
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      <PlayButton instance={instance} className="relative" />
    </motion.article>
  );
}

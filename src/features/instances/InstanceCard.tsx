import { useMutation, useQueryClient } from "@tanstack/react-query";
import { motion } from "framer-motion";
import { Copy, FolderOpen, MoreHorizontal, Play, Settings2, Square, Trash2 } from "lucide-react";
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
import { formatRelative } from "@/lib/format";
import { duplicateInstance, openInstanceFolder, type Instance } from "@/lib/tauri/instances";
import { InstanceIcon } from "./InstanceIcon";
import { INSTANCES_KEY } from "./useInstances";

interface InstanceCardProps {
  instance: Instance;
  onDelete: (instance: Instance) => void;
}

export function InstanceCard({ instance, onDelete }: InstanceCardProps) {
  const { t, i18n } = useTranslation();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { instanceId, phase, play, stop } = useGameStore();

  const thisRunning = instanceId === instance.id && isBusy(phase);
  const otherRunning = !thisRunning && isBusy(phase);
  const settingsPath = `/instances/${instance.id}`;

  const duplicate = useMutation({
    mutationFn: () => duplicateInstance(instance.id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: INSTANCES_KEY }),
  });

  return (
    <motion.article
      layout
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      whileHover={{ y: -2 }}
      transition={{ duration: 0.15 }}
      className="group flex flex-col gap-4 rounded-xl border bg-surface p-4 transition-colors hover:border-accent/40 hover:bg-surface-raised/60"
    >
      <div className="flex items-start gap-3">
        <button
          type="button"
          onClick={() => navigate(settingsPath)}
          className="flex min-w-0 flex-1 items-center gap-3 text-left"
        >
          <InstanceIcon icon={instance.icon} instanceId={instance.id} className="size-14" />
          <div className="min-w-0">
            <h3 className="truncate font-semibold">{instance.name}</h3>
            <p className="text-xs text-muted">
              {instance.gameVersion} · {t(`instances.loader.${instance.loader}`)}
            </p>
            <p className="mt-0.5 text-xs text-muted">
              {instance.lastPlayedAt
                ? t("instances.lastPlayed", { when: formatRelative(instance.lastPlayedAt, i18n.language) })
                : t("instances.neverPlayed")}
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

      {thisRunning ? (
        <Button size="lg" variant="secondary" onClick={() => void stop()}>
          <Square size={16} />
          {t("console.stop")}
        </Button>
      ) : (
        <Button size="lg" disabled={otherRunning} onClick={() => void play(instance.id, instance.name)}>
          <Play size={18} />
          {t("instances.play")}
        </Button>
      )}
    </motion.article>
  );
}

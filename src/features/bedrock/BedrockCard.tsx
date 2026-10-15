import { useMutation, useQuery } from "@tanstack/react-query";
import { motion } from "framer-motion";
import { Blocks, MoreHorizontal, PackageOpen, Play, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { playBedrockInstance, type BedrockInstance } from "@/lib/tauri/bedrock";
import { errorMessage } from "@/lib/tauri/errors";
import { bedrockInstalled } from "@/lib/tauri/servers";

const HUE = 160;

interface BedrockCardProps {
  instance: BedrockInstance;
  index?: number;
  onManage: (instance: BedrockInstance) => void;
  onDelete: (instance: BedrockInstance) => void;
}

/** A Bedrock instance next to the Java ones: its packs go into the game when it is played. */
export function BedrockCard({ instance, index = 0, onManage, onDelete }: BedrockCardProps) {
  const { t } = useTranslation();
  const installed = useQuery({ queryKey: ["bedrock-installed"], queryFn: bedrockInstalled, staleTime: Infinity });
  const play = useMutation({
    mutationFn: () => playBedrockInstance(instance.id),
    onSuccess: () => toast.success(t("bedrock.ready")),
    onError: (error) => toast.error(errorMessage(t, error)),
  });

  return (
    <motion.article
      layout
      initial={{ opacity: 0, y: 12 }}
      animate={{ opacity: 1, y: 0 }}
      whileHover={{ y: -3 }}
      transition={{ duration: 0.25, delay: Math.min(index, 8) * 0.05 }}
      className="glow-follow group relative flex flex-col gap-4 overflow-hidden rounded-2xl border bg-surface p-4 transition-[border-color,box-shadow] hover:border-accent/50 hover:shadow-glow"
    >
      <div
        className="pointer-events-none absolute inset-x-0 top-0 h-40 opacity-70 transition-opacity group-hover:opacity-100"
        style={{ background: `linear-gradient(180deg, oklch(0.5 0.17 ${HUE} / 0.5), transparent)` }}
      />

      <div className="relative flex items-start gap-3">
        <button type="button" onClick={() => onManage(instance)} className="flex min-w-0 flex-1 items-center gap-3 text-left">
          <div
            className="grid size-16 shrink-0 place-items-center rounded-lg text-white shadow-lg ring-1 ring-white/10"
            style={{ background: `linear-gradient(135deg, oklch(0.66 0.17 ${HUE}), oklch(0.42 0.16 ${HUE + 30}))` }}
          >
            <Blocks size={30} />
          </div>
          <div className="min-w-0">
            <h3 className="truncate text-base font-semibold">{instance.name}</h3>
            <div className="mt-1.5 flex flex-wrap gap-1.5">
              <span className="rounded-full px-2 py-0.5 text-[11px] font-medium" style={{ background: `oklch(0.5 0.15 ${HUE} / 0.3)`, color: `oklch(0.85 0.12 ${HUE})` }}>
                {t("servers.bedrock")}
              </span>
              <span className="rounded-full bg-surface-raised px-2 py-0.5 text-[11px] font-medium text-muted">{t("bedrock.packsCount", { count: instance.packs.length })}</span>
            </div>
          </div>
        </button>

        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="sm" className="size-8 px-0" aria-label={t("instances.menu")}>
              <MoreHorizontal size={16} />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuItem onSelect={() => onManage(instance)}>
              <PackageOpen size={15} />
              {t("bedrock.manage")}
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem danger onSelect={() => onDelete(instance)}>
              <Trash2 size={15} />
              {t("instances.delete")}
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      <Button className="relative" disabled={installed.data !== true || play.isPending} title={installed.data === false ? t("bedrock.notInstalled") : undefined} onClick={() => play.mutate()}>
        <Play size={16} />
        {t("bedrock.play")}
      </Button>
    </motion.article>
  );
}

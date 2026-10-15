import { useMutation, useQueryClient } from "@tanstack/react-query";
import { ExternalLink, FilePlus2, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { addBedrockPack, removeBedrockPack, type BedrockInstance } from "@/lib/tauri/bedrock";
import { errorMessage } from "@/lib/tauri/errors";
import { openExternalLink } from "@/lib/tauri/modrinth";
import { BEDROCK_KEY } from "./useBedrockInstances";

const ADDONS_URL = "https://www.curseforge.com/minecraft-bedrock/addons";

interface BedrockPacksDialogProps {
  instance: BedrockInstance | null;
  onClose: () => void;
}

/** The add-on packs of one Bedrock instance: what is in it, adding a file, taking a pack out. */
export function BedrockPacksDialog({ instance, onClose }: BedrockPacksDialogProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const refresh = () => queryClient.invalidateQueries({ queryKey: BEDROCK_KEY });
  const failed = (error: unknown) => toast.error(errorMessage(t, error));

  const add = useMutation({ mutationFn: (id: string) => addBedrockPack(id), onSuccess: refresh, onError: failed });
  const remove = useMutation({ mutationFn: ({ id, folder }: { id: string; folder: string }) => removeBedrockPack(id, folder), onSuccess: refresh, onError: failed });

  return (
    <Dialog open={instance !== null} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="w-[min(34rem,calc(100vw-2rem))]">
        <DialogTitle>{instance?.name}</DialogTitle>
        <DialogDescription>{t("bedrock.note")}</DialogDescription>

        <div className="mt-4 space-y-4">
          {instance?.packs.length === 0 ? (
            <p className="rounded-lg border border-dashed p-3 text-sm text-muted">{t("bedrock.noPacks")}</p>
          ) : (
            <ul className="max-h-72 space-y-1.5 overflow-y-auto pr-1">
              {instance?.packs.map((pack) => (
                <li key={pack.folder} className="flex items-center gap-3 rounded-lg border bg-surface/50 px-3 py-2 text-sm">
                  <span className="min-w-0 flex-1 truncate font-medium">{pack.name}</span>
                  <span className="rounded-full bg-accent/15 px-2 py-0.5 text-[11px] font-medium text-accent">{t(`bedrock.${pack.kind}`)}</span>
                  <span className="font-mono text-xs text-muted">{pack.version}</span>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="size-7 px-0 hover:text-danger"
                    aria-label={t("bedrock.removePack")}
                    title={t("bedrock.removePack")}
                    disabled={remove.isPending}
                    onClick={() => remove.mutate({ id: instance.id, folder: pack.folder })}
                  >
                    <X size={14} />
                  </Button>
                </li>
              ))}
            </ul>
          )}

          <div className="flex flex-wrap items-center justify-between gap-2">
            <div className="flex gap-2">
              <Button variant="secondary" size="sm" disabled={add.isPending || !instance} onClick={() => instance && add.mutate(instance.id)}>
                <FilePlus2 size={15} />
                {t("bedrock.addPack")}
              </Button>
              <Button variant="ghost" size="sm" onClick={() => void openExternalLink(ADDONS_URL)}>
                <ExternalLink size={14} />
                {t("bedrock.browse")}
              </Button>
            </div>
            <Button onClick={onClose}>{t("common.close")}</Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}

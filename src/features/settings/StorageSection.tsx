import { useMutation, useQuery } from "@tanstack/react-query";
import { FolderInput, HardDrive } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { formatBytes } from "@/lib/format";
import { errorMessage } from "@/lib/tauri/errors";
import { getDataLocation, moveDataLocation, pickDataFolder, restartApp } from "@/lib/tauri/location";

/** Shows where the files live and lets the player move them to another folder or drive. */
export function StorageChooser() {
  const { t } = useTranslation();
  const location = useQuery({ queryKey: ["data-location"], queryFn: getDataLocation });
  const [target, setTarget] = useState<string | null>(null);

  const choose = useMutation({ mutationFn: pickDataFolder, onSuccess: (folder) => folder && setTarget(folder) });
  const move = useMutation({
    mutationFn: async (folder: string) => {
      await moveDataLocation(folder);
      await restartApp();
    },
  });

  return (
    <>
      <div className="flex flex-wrap items-center justify-between gap-4 rounded-lg border bg-background/40 px-4 py-3">
        <div className="min-w-0">
          <p className="truncate font-mono text-sm" title={location.data?.path}>
            {location.data?.path ?? "…"}
          </p>
          {location.data && <p className="text-xs text-muted">{t("storage.used", { size: formatBytes(location.data.sizeBytes) })}</p>}
        </div>
        <Button variant="secondary" size="sm" disabled={choose.isPending} onClick={() => choose.mutate()}>
          <FolderInput size={14} />
          {t("storage.change")}
        </Button>
      </div>
      {choose.isError && <p className="mt-2 text-sm text-danger">{errorMessage(t, choose.error)}</p>}

      <Dialog open={target !== null} onOpenChange={(open) => !open && !move.isPending && setTarget(null)}>
        <DialogContent className="w-[min(30rem,calc(100vw-2rem))]">
          <DialogTitle>{t("storage.confirmTitle")}</DialogTitle>
          <DialogDescription>{t("storage.confirmBody")}</DialogDescription>
          <p className="mt-4 break-all rounded-md border bg-background/40 px-3 py-2 font-mono text-sm">{target}</p>
          {move.isError && <p className="mt-3 text-sm text-danger">{errorMessage(t, move.error)}</p>}
          <div className="mt-6 flex justify-end gap-2">
            <Button variant="ghost" disabled={move.isPending} onClick={() => setTarget(null)}>
              {t("common.cancel")}
            </Button>
            <Button disabled={move.isPending || target === null} onClick={() => target && move.mutate(target)}>
              {move.isPending ? t("storage.moving") : t("storage.move")}
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </>
  );
}

export function StorageSection() {
  const { t } = useTranslation();

  return (
    <section className="panel p-5">
      <h2 className="flex items-center gap-2 font-semibold">
        <HardDrive size={16} className="text-accent" />
        {t("storage.title")}
      </h2>
      <p className="mb-4 mt-1 text-muted">{t("storage.hint")}</p>
      <StorageChooser />
    </section>
  );
}

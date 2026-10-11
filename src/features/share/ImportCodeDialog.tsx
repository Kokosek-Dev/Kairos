import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { INSTANCES_KEY } from "@/features/instances/useInstances";
import { errorMessage } from "@/lib/tauri/errors";
import { importInstanceCode, previewInstanceCode } from "@/lib/tauri/share";
import type { InstallProgress } from "@/lib/tauri/versions";

const CODE_PREFIX = "KAIROS1-";

export function ImportCodeDialog({ open, onOpenChange }: { open: boolean; onOpenChange: (open: boolean) => void }) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [code, setCode] = useState("");
  const [progress, setProgress] = useState<InstallProgress | null>(null);
  const trimmed = code.replace(/\s/g, "");
  const preview = useQuery({
    queryKey: ["code-preview", trimmed],
    queryFn: () => previewInstanceCode(trimmed),
    enabled: trimmed.startsWith(CODE_PREFIX) && trimmed.length > CODE_PREFIX.length + 8,
    retry: false,
  });

  const create = useMutation({
    mutationFn: () => importInstanceCode(trimmed, setProgress),
    onSuccess: async ({ instance, failed }) => {
      await queryClient.invalidateQueries({ queryKey: INSTANCES_KEY });
      toast.success(t("share.created", { name: instance.name }));
      if (failed > 0) toast.warning(t("share.failed", { count: failed }));
      onOpenChange(false);
      setCode("");
      navigate(`/instances/${instance.id}`);
    },
  });

  const shown = preview.data;
  return (
    <Dialog open={open} onOpenChange={(next) => !create.isPending && onOpenChange(next)}>
      <DialogContent className="w-[min(34rem,calc(100vw-2rem))]">
        <DialogTitle>{t("share.importTitle")}</DialogTitle>
        <DialogDescription>{t("share.importBody")}</DialogDescription>
        <textarea
          autoFocus
          value={code}
          disabled={create.isPending}
          placeholder={`${CODE_PREFIX}…`}
          onChange={(event) => setCode(event.target.value)}
          className="mt-4 h-28 w-full resize-none rounded-md border bg-background/60 p-3 font-mono text-xs outline-none select-text placeholder:text-muted focus:border-accent"
        />
        {preview.isError && <p className="mt-2 text-sm text-danger">{t("share.invalid")}</p>}
        {shown && (
          <div className="mt-3 rounded-lg border bg-surface/60 p-3 text-sm">
            <p className="font-medium">{shown.name}</p>
            <p className="text-muted">
              Minecraft {shown.gameVersion} · {t(`instances.loader.${shown.loader}`)}
              {shown.loaderVersion ? ` ${shown.loaderVersion}` : ""}
            </p>
            <p className="text-muted">{t("share.counts", { mods: shown.mods, packs: shown.resourcePacks, shaders: shown.shaders })}</p>
          </div>
        )}
        {create.isPending && (
          <div className="mt-4">
            <div className="h-2 overflow-hidden rounded-full bg-surface-raised">
              <div
                className="h-full bg-brand transition-all"
                style={{ width: `${progress && progress.totalFiles > 0 ? (progress.doneFiles / progress.totalFiles) * 100 : 5}%` }}
              />
            </div>
            <p className="mt-2 text-sm text-muted">{t("share.creating", { done: progress?.doneFiles ?? 0, total: progress?.totalFiles ?? 0 })}</p>
          </div>
        )}
        {create.isError && <p className="mt-3 text-sm text-danger">{errorMessage(t, create.error)}</p>}
        <div className="mt-5 flex justify-end gap-2">
          <Button variant="ghost" disabled={create.isPending} onClick={() => onOpenChange(false)}>
            {t("common.cancel")}
          </Button>
          <Button disabled={!shown || create.isPending} onClick={() => create.mutate()}>
            {t("share.create")}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}

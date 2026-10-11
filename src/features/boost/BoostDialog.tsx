import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Check, X, Zap } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { contentKey } from "@/features/discover/jobsStore";
import { INSTANCES_KEY } from "@/features/instances/useInstances";
import { formatMemory } from "@/lib/format";
import { boostInstance, boostMods, type BoostReport } from "@/lib/tauri/boost";
import { errorMessage } from "@/lib/tauri/errors";
import type { Instance } from "@/lib/tauri/instances";
import type { InstallProgress } from "@/lib/tauri/versions";

const TITLES: Record<string, string> = {
  sodium: "Sodium",
  lithium: "Lithium",
  "ferrite-core": "FerriteCore",
  modernfix: "ModernFix",
  entityculling: "Entity Culling",
  embeddium: "Embeddium",
};

const titleOf = (project: string) => TITLES[project] ?? project;

interface BoostDialogProps {
  instance: Instance;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function BoostDialog({ instance, open, onOpenChange }: BoostDialogProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [progress, setProgress] = useState<InstallProgress | null>(null);
  const [report, setReport] = useState<BoostReport | null>(null);
  const mods = useQuery({ queryKey: ["boost-mods", instance.loader], queryFn: () => boostMods(instance.loader), enabled: open });

  const boost = useMutation({
    mutationFn: () => boostInstance(instance.id, setProgress),
    onSuccess: async (result) => {
      setReport(result);
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: INSTANCES_KEY }),
        queryClient.invalidateQueries({ queryKey: contentKey(instance.id) }),
        queryClient.invalidateQueries({ queryKey: ["moments", instance.id] }),
      ]);
    },
  });

  const close = (next: boolean) => {
    if (boost.isPending) return;
    onOpenChange(next);
    if (!next) {
      setReport(null);
      setProgress(null);
      boost.reset();
    }
  };

  return (
    <Dialog open={open} onOpenChange={close}>
      <DialogContent className="w-[min(32rem,calc(100vw-2rem))]">
        <DialogTitle className="flex items-center gap-2">
          <Zap size={18} className="text-accent" />
          {t("boost.title")}
        </DialogTitle>
        <DialogDescription>{t("boost.body")}</DialogDescription>

        {!report && (
          <div className="mt-4 space-y-3">
            {mods.data && mods.data.length > 0 ? (
              <div>
                <p className="mb-2 text-sm font-medium">{t("boost.mods")}</p>
                <ul className="flex flex-wrap gap-2">
                  {mods.data.map((project) => (
                    <li key={project} className="rounded-full border bg-surface/60 px-3 py-1 text-sm">
                      {titleOf(project)}
                    </li>
                  ))}
                </ul>
              </div>
            ) : (
              mods.data && <p className="text-sm text-muted">{t("boost.vanillaNote")}</p>
            )}
            <p className="text-sm text-muted">{t("boost.undoNote")}</p>
            {boost.isPending && (
              <div>
                <div className="h-2 overflow-hidden rounded-full bg-surface-raised">
                  <div
                    className="h-full bg-brand transition-all"
                    style={{ width: `${progress && progress.totalFiles > 0 ? (progress.doneFiles / progress.totalFiles) * 100 : 5}%` }}
                  />
                </div>
                <p className="mt-2 text-sm text-muted">
                  {t("boost.running", { done: progress?.doneFiles ?? 0, total: progress?.totalFiles ?? 0 })}
                </p>
              </div>
            )}
            {boost.isError && <p className="text-sm text-danger">{errorMessage(t, boost.error)}</p>}
          </div>
        )}

        {report && (
          <div className="mt-4 space-y-3">
            <p className="font-medium text-success">{t("boost.done")}</p>
            <ul className="space-y-1.5">
              {report.items.map((item) => (
                <li key={item.project} className="flex items-start gap-2 text-sm">
                  {item.installed ? <Check size={16} className="mt-0.5 shrink-0 text-success" /> : <X size={16} className="mt-0.5 shrink-0 text-danger" />}
                  <span>
                    <span className="font-medium">{item.title ?? titleOf(item.project)}</span>
                    {!item.installed && <span className="text-muted"> · {t("boost.skipped")}</span>}
                  </span>
                </li>
              ))}
            </ul>
            <p className="text-sm text-muted">{t("boost.memorySet", { memory: formatMemory(report.memoryMb) })}</p>
            {report.javaFlagsAdded && <p className="text-sm text-muted">{t("boost.javaSet")}</p>}
          </div>
        )}

        <div className="mt-6 flex justify-end gap-2">
          <Button variant="ghost" disabled={boost.isPending} onClick={() => close(false)}>
            {report ? t("common.close") : t("common.cancel")}
          </Button>
          {!report && (
            <Button disabled={boost.isPending || !mods.data} onClick={() => boost.mutate()}>
              <Zap size={16} />
              {t("boost.start")}
            </Button>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}

import { useQuery } from "@tanstack/react-query";
import { FileSearch, LifeBuoy, Settings2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { analyzeCrash, revealFile, type CrashKind } from "@/lib/tauri/crash";
import { useProblemReport } from "@/features/settings/useProblemReport";
import { useGameStore } from "./gameStore";

/** Kinds of crash that are fixed in the instance settings, so a shortcut is offered. */
const SETTINGS_FIXES: CrashKind[] = ["heap_too_large", "out_of_memory", "wrong_java"];

export function CrashDialog() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { crash, dismissCrash } = useGameStore();
  const report = useProblemReport();
  const analysis = useQuery({
    queryKey: ["crash", crash?.instanceId, crash?.startedAt],
    queryFn: () => analyzeCrash(crash?.instanceId ?? "", crash?.startedAt ?? 0),
    enabled: crash !== null,
    retry: false,
  });
  const result = analysis.data;
  const kind: CrashKind = result?.kind ?? "unknown";

  return (
    <Dialog open={crash !== null} onOpenChange={(open) => !open && dismissCrash()}>
      <DialogContent className="w-[min(34rem,calc(100vw-2rem))]">
        <DialogTitle>{t(`crash.kind.${kind}.title`)}</DialogTitle>
        <DialogDescription>{t("crash.subtitle", { name: crash?.title })}</DialogDescription>
        <p className="mt-4">{t(`crash.kind.${kind}.advice`)}</p>
        {result && result.details.length > 0 && (
          <pre className="mt-4 max-h-44 overflow-auto whitespace-pre-wrap break-words rounded-md border bg-background/60 p-3 font-mono text-xs text-muted">
            {result.details.join("\n")}
          </pre>
        )}
        <div className="mt-6 flex flex-wrap justify-end gap-2">
          {result?.reportPath && (
            <Button variant="secondary" onClick={() => void revealFile(result.reportPath ?? "")}>
              <FileSearch size={16} />
              {t("crash.showReport")}
            </Button>
          )}
          {crash && SETTINGS_FIXES.includes(kind) && (
            <Button
              variant="secondary"
              onClick={() => {
                navigate(`/instances/${crash.instanceId}`);
                dismissCrash();
              }}
            >
              <Settings2 size={16} />
              {t("crash.openSettings")}
            </Button>
          )}
          <Button variant="secondary" disabled={report.isPending} onClick={() => report.mutate(crash?.instanceId ?? null)}>
            <LifeBuoy size={16} />
            {t("help.saveReport")}
          </Button>
          <Button onClick={dismissCrash}>{t("common.close")}</Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}

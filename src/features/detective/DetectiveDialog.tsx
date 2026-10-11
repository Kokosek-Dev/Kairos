import { Loader2, Search, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { useDetectiveStore } from "./detectiveStore";

export function DetectiveDialog() {
  const { t } = useTranslation();
  const { stage, candidates, titles, suspects, testing, culprit, checking, round, start, answer, finish, cancel } = useDetectiveStore();
  const name = culprit ? (titles[culprit] ?? culprit) : "";
  const settled = stage === "found" || stage === "multiple";

  return (
    <Dialog open={stage !== "closed"} onOpenChange={(open) => !open && void (settled ? finish(false) : cancel())}>
      <DialogContent className="w-[min(32rem,calc(100vw-2rem))]">
        <DialogTitle className="flex items-center gap-2">
          <Search size={18} className="text-accent" />
          {stage === "found" ? t("detective.foundTitle", { name }) : stage === "multiple" ? t("detective.multipleTitle") : t("detective.title")}
        </DialogTitle>
        <DialogDescription>
          {stage === "found" ? t("detective.foundBody") : stage === "multiple" ? t("detective.multipleBody") : t("detective.body")}
        </DialogDescription>

        {stage === "intro" && (
          <p className="mt-4 text-sm text-muted">
            {candidates.length < 2 ? t("detective.needsMods") : t("detective.safety", { count: candidates.length })}
          </p>
        )}

        {(stage === "launching" || stage === "asking") && (
          <div className="mt-4 space-y-3">
            <p className="text-sm text-muted">
              {checking
                ? t("detective.checking", { name })
                : t("detective.round", { round, suspects: suspects.length, enabled: testing.length })}
            </p>
            {stage === "launching" && (
              <p className="flex items-center gap-2 text-sm">
                <Loader2 size={16} className="animate-spin text-accent" />
                {t("detective.running")}
              </p>
            )}
            {stage === "asking" && <p className="font-medium">{t("detective.ask")}</p>}
          </div>
        )}

        {stage === "error" && <p className="mt-4 text-sm text-danger">{t("detective.error")}</p>}

        <div className="mt-6 flex flex-wrap justify-end gap-2">
          {stage === "intro" && (
            <>
              <Button variant="ghost" onClick={() => void cancel()}>
                {t("common.cancel")}
              </Button>
              <Button disabled={candidates.length < 2} onClick={() => void start()}>
                <Search size={16} />
                {t("detective.start")}
              </Button>
            </>
          )}
          {(stage === "launching" || stage === "error") && (
            <Button variant="secondary" onClick={() => void cancel()}>
              {t("detective.stopRestore")}
            </Button>
          )}
          {stage === "asking" && (
            <>
              <Button variant="ghost" onClick={() => void cancel()}>
                {t("detective.stopRestore")}
              </Button>
              <Button variant="secondary" onClick={() => void answer(false)}>
                {t("detective.no")}
              </Button>
              <Button onClick={() => void answer(true)}>{t("detective.yes")}</Button>
            </>
          )}
          {stage === "found" && (
            <>
              <Button variant="secondary" onClick={() => void finish(true)}>
                <Trash2 size={16} />
                {t("detective.remove")}
              </Button>
              <Button onClick={() => void finish(false)}>{t("detective.keepOff")}</Button>
            </>
          )}
          {stage === "multiple" && <Button onClick={() => void finish(false)}>{t("common.close")}</Button>}
        </div>
      </DialogContent>
    </Dialog>
  );
}

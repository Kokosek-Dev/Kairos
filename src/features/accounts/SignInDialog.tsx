import { Check, Copy, ExternalLink, Loader2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { cn } from "@/lib/utils";
import { errorMessage, isLauncherError } from "@/lib/tauri/errors";
import { openLoginPage, SIGN_IN_STEPS, type SignInStep } from "@/lib/tauri/auth";
import { useSignInStore, type SignInPhase } from "./signInStore";

const APPROVAL_FORM_URL = "https://aka.ms/mce-reviewappid";

function Steps({ current }: { current: SignInStep }) {
  const { t } = useTranslation();
  const currentIndex = SIGN_IN_STEPS.indexOf(current);
  return (
    <ul className="mt-5 space-y-2">
      {SIGN_IN_STEPS.map((step, index) => (
        <li
          key={step}
          className={cn("flex items-center gap-3 text-sm", index > currentIndex ? "text-muted" : "text-foreground")}
        >
          {index < currentIndex ? (
            <Check size={16} className="text-success" />
          ) : index === currentIndex ? (
            <Loader2 size={16} className="animate-spin text-accent" />
          ) : (
            <span className="size-4 rounded-full border" />
          )}
          {t(`signIn.step.${step}`)}
        </li>
      ))}
    </ul>
  );
}

function Body({ phase, onClose }: { phase: SignInPhase; onClose: () => void }) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const retry = useSignInStore((state) => state.start);

  switch (phase.kind) {
    case "idle":
    case "starting":
      return (
        <div className="mt-6 flex items-center gap-3 text-muted">
          <Loader2 size={18} className="animate-spin" />
          {t("signIn.starting")}
        </div>
      );
    case "code":
      return (
        <div className="mt-5">
          <p className="text-muted">{t("signIn.codeHint")}</p>
          <div className="mt-4 flex items-center justify-center gap-3 rounded-xl border bg-background/60 py-5">
            <span className="select-text font-mono text-3xl font-bold tracking-[0.3em]">{phase.userCode}</span>
            <Button
              variant="ghost"
              size="sm"
              aria-label={t("signIn.copy")}
              onClick={() => void navigator.clipboard.writeText(phase.userCode)}
            >
              <Copy size={16} />
            </Button>
          </div>
          <div className="mt-4 flex items-center justify-between gap-3">
            <span className="flex items-center gap-2 text-sm text-muted">
              <Loader2 size={14} className="animate-spin" />
              {t("signIn.waiting")}
            </span>
            <Button variant="secondary" size="sm" onClick={() => void openLoginPage(phase.verificationUri)}>
              <ExternalLink size={14} />
              {t("signIn.openPage")}
            </Button>
          </div>
        </div>
      );
    case "working":
      return <Steps current={phase.step} />;
    case "done":
      return (
        <div className="mt-6">
          <p className="flex items-center gap-2 font-medium text-success">
            <Check size={18} />
            {t("signIn.done", { name: phase.username })}
          </p>
          <div className="mt-5 flex justify-end">
            <Button onClick={onClose}>{t("signIn.close")}</Button>
          </div>
        </div>
      );
    case "failed": {
      const code = isLauncherError(phase.error) ? phase.error.code : "";
      return (
        <div className="mt-5">
          <p className="text-danger">{errorMessage(t, phase.error)}</p>
          <div className="mt-5 flex flex-wrap justify-end gap-2">
            {code === "auth_not_configured" && (
              <Button
                variant="secondary"
                onClick={() => {
                  onClose();
                  navigate("/settings");
                }}
              >
                {t("signIn.openSettings")}
              </Button>
            )}
            {code === "auth_app_not_approved" && (
              <Button variant="secondary" onClick={() => void openLoginPage(APPROVAL_FORM_URL)}>
                <ExternalLink size={14} />
                {t("signIn.approvalForm")}
              </Button>
            )}
            <Button variant="ghost" onClick={onClose}>
              {t("signIn.close")}
            </Button>
            {code !== "auth_not_configured" && <Button onClick={() => void retry()}>{t("signIn.retry")}</Button>}
          </div>
        </div>
      );
    }
  }
}

export function SignInDialog() {
  const { t } = useTranslation();
  const { open, phase, close } = useSignInStore();

  return (
    <Dialog open={open} onOpenChange={(next) => !next && close()}>
      <DialogContent className="w-[min(30rem,calc(100vw-2rem))]">
        <DialogTitle>{t("signIn.title")}</DialogTitle>
        <DialogDescription>{t("signIn.subtitle")}</DialogDescription>
        <Body phase={phase} onClose={close} />
      </DialogContent>
    </Dialog>
  );
}

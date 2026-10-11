import { useQuery } from "@tanstack/react-query";
import { Check, Copy } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { errorMessage } from "@/lib/tauri/errors";
import { exportInstanceCode } from "@/lib/tauri/share";

interface ShareCodeDialogProps {
  instanceId: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function ShareCodeDialog({ instanceId, open, onOpenChange }: ShareCodeDialogProps) {
  const { t } = useTranslation();
  const [copied, setCopied] = useState(false);
  const code = useQuery({ queryKey: ["instance-code", instanceId], queryFn: () => exportInstanceCode(instanceId), enabled: open, gcTime: 0 });

  const copy = async () => {
    if (!code.data) return;
    try {
      await navigator.clipboard.writeText(code.data);
      setCopied(true);
      toast.success(t("share.copied"));
      setTimeout(() => setCopied(false), 2000);
    } catch {
      toast.error(t("share.copyFailed"));
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(34rem,calc(100vw-2rem))]">
        <DialogTitle>{t("share.title")}</DialogTitle>
        <DialogDescription>{t("share.body")}</DialogDescription>
        <textarea
          readOnly
          value={code.data ?? ""}
          onFocus={(event) => event.currentTarget.select()}
          className="mt-4 h-32 w-full resize-none rounded-md border bg-background/60 p-3 font-mono text-xs outline-none select-text focus:border-accent"
        />
        {code.isError && <p className="mt-2 text-sm text-danger">{errorMessage(t, code.error)}</p>}
        <p className="mt-3 text-xs text-muted">{t("share.note")}</p>
        <div className="mt-5 flex justify-end gap-2">
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t("common.close")}
          </Button>
          <Button disabled={!code.data} onClick={() => void copy()}>
            {copied ? <Check size={16} /> : <Copy size={16} />}
            {t("share.copy")}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}

import { useMutation, useQuery } from "@tanstack/react-query";
import { Check, Copy, ExternalLink, Send } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { Input, Textarea } from "@/components/ui/input";
import { Segmented } from "@/components/ui/segmented";
import { Switch } from "@/components/ui/switch";
import { useIncognito } from "@/features/incognito/useIncognito";
import { errorMessage } from "@/lib/tauri/errors";
import { feedbackAvailable, feedbackDetails, sendFeedback, type FeedbackKind } from "@/lib/tauri/feedback";
import { openExternalLink } from "@/lib/tauri/modrinth";
import { useFeedbackStore } from "./feedbackStore";

const KINDS: FeedbackKind[] = ["bug", "idea", "other"];
const MIN_LENGTH = 10;
const MAX_LENGTH = 4000;
const ISSUES_URL = "https://github.com/Kokosek-Dev/Kairos/issues/new";
/** What fits into a link without it being refused. */
const ISSUE_TEXT_LIMIT = 1500;

export function FeedbackDialog() {
  const { t } = useTranslation();
  const { open, kind: initialKind, instanceId, close } = useFeedbackStore();
  const incognito = useIncognito();
  const [kind, setKind] = useState<FeedbackKind>("bug");
  const [message, setMessage] = useState("");
  const [contact, setContact] = useState("");
  const [detailsChoice, setDetailsChoice] = useState<boolean | null>(null);
  const [showDetails, setShowDetails] = useState(false);
  const [copied, setCopied] = useState(false);

  /** Technical details help with a problem, so they start on there and off for ideas and other messages. */
  const withDetails = detailsChoice ?? kind === "bug";

  const available = useQuery({ queryKey: ["feedback-available"], queryFn: feedbackAvailable, staleTime: Infinity });
  const details = useQuery({
    queryKey: ["feedback-details", instanceId],
    queryFn: () => feedbackDetails(instanceId),
    enabled: open && withDetails,
    gcTime: 0,
  });

  useEffect(() => {
    if (open) setKind(initialKind);
  }, [open, initialKind]);

  const reset = () => {
    setMessage("");
    setContact("");
    setShowDetails(false);
    send.reset();
  };

  const send = useMutation({
    mutationFn: () =>
      sendFeedback({
        kind,
        message: message.trim(),
        contact: contact.trim() || null,
        details: withDetails ? (details.data ?? null) : null,
      }),
  });

  const trimmed = message.trim().length;
  const valid = trimmed >= MIN_LENGTH && trimmed <= MAX_LENGTH;
  const canSend = available.data === true && !incognito && valid && !send.isPending;

  const report = () =>
    [`[${t(`feedback.kind.${kind}`)}]`, message.trim(), contact.trim() && `${t("feedback.contact")}: ${contact.trim()}`, withDetails && details.data].filter(Boolean).join("\n\n");

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(report());
      setCopied(true);
      toast.success(t("feedback.copied"));
      setTimeout(() => setCopied(false), 2000);
    } catch {
      toast.error(t("share.copyFailed"));
    }
  };

  const openIssue = () => {
    const title = message.trim().split("\n")[0].slice(0, 80);
    const text = `${message.trim().slice(0, ISSUE_TEXT_LIMIT)}\n\n(${t(`feedback.kind.${kind}`)})`;
    void openExternalLink(`${ISSUES_URL}?title=${encodeURIComponent(title)}&body=${encodeURIComponent(text)}`);
  };

  const onOpenChange = (next: boolean) => {
    if (send.isPending) return;
    if (!next) {
      close();
      if (send.isSuccess) reset();
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(34rem,calc(100vw-2rem))]">
        <DialogTitle>{t("feedback.title")}</DialogTitle>
        <DialogDescription>{available.data === false ? t("feedback.offline") : t("feedback.subtitle")}</DialogDescription>

        {send.isSuccess ? (
          <div className="mt-6 space-y-4 text-center">
            <p className="flex items-center justify-center gap-2 text-lg font-semibold text-success">
              <Check size={20} />
              {t("feedback.sent")}
            </p>
            <p className="text-muted">{t("feedback.sentBody")}</p>
            <Button onClick={() => onOpenChange(false)}>{t("common.close")}</Button>
          </div>
        ) : (
          <>
            <div className="mt-4 space-y-4">
              <Segmented value={kind} onChange={setKind} options={KINDS.map((value) => ({ value, label: t(`feedback.kind.${value}`) }))} />
              <div>
                <Textarea
                  value={message}
                  maxLength={MAX_LENGTH}
                  placeholder={t(`feedback.placeholder.${kind}`)}
                  className="h-28 font-sans"
                  onChange={(event) => setMessage(event.target.value)}
                />
                {trimmed > 0 && trimmed < MIN_LENGTH && <p className="mt-1 text-xs text-muted">{t("feedback.tooShort")}</p>}
              </div>
              <Input value={contact} maxLength={120} placeholder={t("feedback.contactPlaceholder")} onChange={(event) => setContact(event.target.value)} />
              <div className="rounded-lg border bg-surface/50 p-3">
                <div className="flex items-center justify-between gap-3">
                  <div className="min-w-0">
                    <p className="font-medium">{t("feedback.includeDetails")}</p>
                    <button type="button" className="text-xs text-accent hover:underline" onClick={() => setShowDetails((value) => !value)} disabled={!withDetails}>
                      {showDetails ? t("feedback.hideDetails") : t("feedback.showDetails")}
                    </button>
                  </div>
                  <Switch checked={withDetails} onChange={setDetailsChoice} label={t("feedback.includeDetails")} />
                </div>
                {withDetails && showDetails && (
                  <pre className="mt-3 max-h-28 overflow-auto whitespace-pre-wrap break-words rounded-md border bg-background/60 p-2 font-mono text-[11px] text-muted select-text">
                    {details.data ?? "…"}
                  </pre>
                )}
              </div>
            </div>

            {incognito && <p className="mt-3 text-sm text-muted">{t("feedback.incognito")}</p>}
            {send.isError && <p className="mt-3 text-sm text-danger">{errorMessage(t, send.error)}</p>}

            <div className="mt-5 flex flex-wrap items-center justify-between gap-2">
              <div className="flex gap-2">
                <Button variant="ghost" size="sm" disabled={!valid} onClick={() => void copy()}>
                  {copied ? <Check size={14} /> : <Copy size={14} />}
                  {t("feedback.copy")}
                </Button>
                <Button variant="ghost" size="sm" disabled={!valid} onClick={openIssue}>
                  <ExternalLink size={14} />
                  {t("feedback.github")}
                </Button>
              </div>
              <div className="flex gap-2">
                <Button variant="ghost" onClick={() => onOpenChange(false)}>
                  {t("common.cancel")}
                </Button>
                <Button disabled={!canSend} onClick={() => send.mutate()}>
                  <Send size={16} />
                  {send.isPending ? t("feedback.sending") : t("feedback.send")}
                </Button>
              </div>
            </div>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}

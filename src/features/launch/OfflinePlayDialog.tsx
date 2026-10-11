import { WifiOff } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { useActiveAccount } from "@/features/skins/useProfile";
import { useGameStore } from "./gameStore";
import { useOfflineStore } from "./offlineStore";

export function OfflinePlayDialog() {
  const { t } = useTranslation();
  const { request, dismiss } = useOfflineStore();
  const { active } = useActiveAccount();
  const play = useGameStore((state) => state.play);

  const confirm = () => {
    if (request) void play(request.instanceId, request.title, true);
    dismiss();
  };

  return (
    <Dialog open={request !== null} onOpenChange={(open) => !open && dismiss()}>
      <DialogContent className="w-[min(30rem,calc(100vw-2rem))]">
        <div className="mb-3 grid size-11 place-items-center rounded-xl bg-accent/15 text-accent">
          <WifiOff size={22} />
        </div>
        <DialogTitle>{t("offline.title")}</DialogTitle>
        <DialogDescription>{t("offline.body", { name: active?.username ?? "" })}</DialogDescription>
        <ul className="mt-4 space-y-1.5 text-sm text-muted">
          <li>• {t("offline.noServers")}</li>
          <li>• {t("offline.noSkin")}</li>
          <li>• {t("offline.ownersOnly")}</li>
        </ul>
        <div className="mt-6 flex justify-end gap-2">
          <Button variant="ghost" onClick={dismiss}>
            {t("common.cancel")}
          </Button>
          <Button onClick={confirm}>{t("offline.confirm")}</Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}

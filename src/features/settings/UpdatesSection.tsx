import { useMutation, useQuery } from "@tanstack/react-query";
import { getVersion } from "@tauri-apps/api/app";
import { RefreshCw } from "lucide-react";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { findUpdate, offerUpdate } from "@/features/updates/updater";

export function UpdatesSection() {
  const { t } = useTranslation();
  const version = useQuery({ queryKey: ["app-version"], queryFn: getVersion, staleTime: Infinity });
  const check = useMutation({
    mutationFn: findUpdate,
    onSuccess: (update) => (update ? offerUpdate(update) : toast.success(t("updater.upToDate"))),
  });

  return (
    <section className="panel p-5">
      <h2 className="flex items-center gap-2 font-semibold">
        <RefreshCw size={16} className="text-accent" />
        {t("updater.title")}
      </h2>
      <p className="mb-4 mt-1 text-muted">{t("updater.current", { version: version.data ?? "" })}</p>
      <Button variant="secondary" disabled={check.isPending} onClick={() => check.mutate()}>
        {t("updater.check")}
      </Button>
    </section>
  );
}

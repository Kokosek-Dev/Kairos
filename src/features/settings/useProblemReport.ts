import { useMutation } from "@tanstack/react-query";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";
import { errorMessage } from "@/lib/tauri/errors";
import { saveProblemReport } from "@/lib/tauri/crash";

/** Saves a problem report and tells the player where it went. */
export function useProblemReport() {
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (instanceId: string | null) => saveProblemReport(instanceId),
    onSuccess: (path) => path && toast.success(t("help.saved", { path })),
    onError: (error) => toast.error(errorMessage(t, error)),
  });
}

import { check, type Update } from "@tauri-apps/plugin-updater";
import { toast } from "sonner";
import i18n from "@/i18n";
import { restartApp } from "@/lib/tauri/location";

/** The newer version of Kairos, or null when this one is current or the check could not be made. */
export async function findUpdate(): Promise<Update | null> {
  try {
    return await check();
  } catch {
    return null;
  }
}

export async function installUpdate(update: Update): Promise<void> {
  const t = i18n.t.bind(i18n);
  const toastId = toast.loading(t("updater.downloading", { version: update.version }));
  try {
    await update.downloadAndInstall();
    toast.success(t("updater.restarting"), { id: toastId });
    await restartApp();
  } catch {
    toast.error(t("updater.failed"), { id: toastId });
  }
}

export function offerUpdate(update: Update): void {
  const t = i18n.t.bind(i18n);
  toast(t("updater.available", { version: update.version }), {
    duration: Infinity,
    action: { label: t("updater.install"), onClick: () => void installUpdate(update) },
  });
}

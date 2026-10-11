import { toast } from "sonner";
import i18n from "@/i18n";
import { usePreferences } from "@/features/settings/preferences";
import { errorMessage } from "@/lib/tauri/errors";
import { createSafetyMoment } from "@/lib/tauri/moments";
import { checkContentUpdates, updateContent } from "@/lib/tauri/modrinth";

/** Brings the instance's content up to date before it starts. A failure never stops the game from launching. */
export async function updateBeforeLaunch(instanceId: string): Promise<void> {
  const { autoUpdateMods, incognito } = usePreferences.getState();
  if (!autoUpdateMods || incognito) return;
  const t = i18n.t.bind(i18n);
  try {
    const updates = await checkContentUpdates(instanceId);
    if (updates.length === 0) return;
    await createSafetyMoment(instanceId, "before_update").catch(() => undefined);
    const toastId = toast.loading(t("updates.running", { count: updates.length }));
    try {
      await updateContent(
        instanceId,
        updates.map(({ projectId, newVersionId }) => ({ projectId, versionId: newVersionId })),
        () => undefined,
      );
      toast.success(t("updates.done", { count: updates.length }), { id: toastId });
    } catch (error) {
      toast.error(t("updates.failed", { reason: errorMessage(t, error) }), { id: toastId });
    }
  } catch {
    // Offline or the catalogue is unreachable: play with what is installed.
  }
}

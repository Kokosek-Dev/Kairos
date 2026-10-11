import { toast } from "sonner";
import { create } from "zustand";
import { INSTANCES_KEY } from "@/features/instances/useInstances";
import { queryClient } from "@/lib/queryClient";
import { errorMessage, isLauncherError } from "@/lib/tauri/errors";
import type { Instance } from "@/lib/tauri/instances";
import {
  importMrpack,
  installContent,
  installModpack,
  updateContent,
  type InstallReport,
} from "@/lib/tauri/modrinth";
import { cancelInstall, type InstallProgress } from "@/lib/tauri/versions";
import type { TFunction } from "i18next";

export const contentKey = (instanceId: string) => ["content", instanceId];

interface JobsState {
  /** Progress of the jobs that are running right now, by job key. */
  jobs: Record<string, InstallProgress>;
  install: (t: TFunction, instanceId: string, projectId: string, versionId?: string) => Promise<void>;
  installModpack: (t: TFunction, projectId: string) => Promise<Instance | null>;
  importPack: (t: TFunction, path: string) => Promise<Instance | null>;
  update: (t: TFunction, instanceId: string, updates: { projectId: string; versionId: string }[]) => Promise<void>;
  cancel: (key: string) => Promise<void>;
}

const STARTING: InstallProgress = { stage: "resolving", doneFiles: 0, totalFiles: 0, doneBytes: 0, totalBytes: 0 };

function describe(t: TFunction, report: InstallReport): string {
  const dependencies = report.installed.filter((entry) => entry.dependency).length;
  const name = report.installed.find((entry) => !entry.dependency)?.title ?? report.installed[0]?.title ?? "";
  const base =
    dependencies > 0 ? t("content.installedWithDependencies", { name, count: dependencies }) : t("content.installedSummary", { name });
  return report.missingDependencies.length > 0
    ? `${base} ${t("content.missingDependencies", { count: report.missingDependencies.length })}`
    : base;
}

/** The backend registers cancellable jobs under these keys. */
function cancelKey(key: string): string {
  if (key.startsWith("update:")) return `modrinth-update:${key.slice("update:".length)}`;
  return key === "mrpack-import" ? key : `modrinth:${key}`;
}

export const useJobsStore = create<JobsState>((set) => {
  const setProgress = (key: string, progress: InstallProgress | null) =>
    set((current) => {
      const { [key]: _removed, ...rest } = current.jobs;
      return { jobs: progress ? { ...rest, [key]: progress } : rest };
    });

  const run = async <T>(
    key: string,
    t: TFunction,
    work: (onProgress: (progress: InstallProgress) => void) => Promise<T>,
    success: (result: T) => string,
  ): Promise<T | null> => {
    setProgress(key, STARTING);
    try {
      const result = await work((progress) => setProgress(key, progress));
      toast.success(success(result));
      return result;
    } catch (error) {
      if (!(isLauncherError(error) && error.code === "cancelled")) toast.error(errorMessage(t, error));
      return null;
    } finally {
      setProgress(key, null);
    }
  };

  return {
    jobs: {},

    install: async (t, instanceId, projectId, versionId) => {
      await run(
        projectId,
        t,
        (onProgress) => installContent(instanceId, projectId, versionId ?? null, onProgress),
        (report) => describe(t, report),
      );
      await queryClient.invalidateQueries({ queryKey: contentKey(instanceId) });
    },

    installModpack: async (t, projectId) => {
      const instance = await run(
        projectId,
        t,
        (onProgress) => installModpack(projectId, null, onProgress),
        (created) => t("content.modpackInstalled", { name: created.name }),
      );
      await queryClient.invalidateQueries({ queryKey: INSTANCES_KEY });
      return instance;
    },

    importPack: async (t, path) => {
      const instance = await run(
        "mrpack-import",
        t,
        (onProgress) => importMrpack(path, onProgress),
        (created) => t("content.modpackInstalled", { name: created.name }),
      );
      await queryClient.invalidateQueries({ queryKey: INSTANCES_KEY });
      return instance;
    },

    update: async (t, instanceId, updates) => {
      await run(
        `update:${instanceId}`,
        t,
        (onProgress) => updateContent(instanceId, updates, onProgress),
        () => t("content.updated", { count: updates.length }),
      );
      await queryClient.invalidateQueries({ queryKey: contentKey(instanceId) });
    },

    cancel: (key) => cancelInstall(cancelKey(key)),
  };
});

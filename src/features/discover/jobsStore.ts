import { create } from "zustand";
import { queryClient } from "@/lib/queryClient";
import { INSTANCES_KEY } from "@/features/instances/useInstances";
import { cancelInstall } from "@/lib/tauri/versions";
import { errorMessage, isLauncherError } from "@/lib/tauri/errors";
import {
  importMrpack,
  installContent,
  installModpack,
  updateContent,
  type InstallReport,
} from "@/lib/tauri/modrinth";
import type { InstallProgress } from "@/lib/tauri/versions";
import type { Instance } from "@/lib/tauri/instances";
import type { TFunction } from "i18next";

export const contentKey = (instanceId: string) => ["content", instanceId];

export interface JobState {
  progress?: InstallProgress;
  message?: { kind: "success" | "error"; text: string };
}

interface JobsState {
  jobs: Record<string, JobState>;
  install: (t: TFunction, instanceId: string, projectId: string, versionId?: string) => Promise<void>;
  installModpack: (t: TFunction, projectId: string) => Promise<Instance | null>;
  importPack: (t: TFunction, path: string) => Promise<Instance | null>;
  update: (t: TFunction, instanceId: string, updates: { projectId: string; versionId: string }[]) => Promise<void>;
  cancel: (key: string) => Promise<void>;
  dismiss: (key: string) => void;
}

function describe(t: TFunction, report: InstallReport): string {
  const dependencies = report.installed.filter((entry) => entry.dependency).length;
  const name = report.installed.find((entry) => !entry.dependency)?.title ?? report.installed[0]?.title ?? "";
  const base =
    dependencies > 0 ? t("content.installedWithDependencies", { name, count: dependencies }) : t("content.installedSummary", { name });
  return report.missingDependencies.length > 0
    ? `${base} ${t("content.missingDependencies", { count: report.missingDependencies.length })}`
    : base;
}

export const useJobsStore = create<JobsState>((set, get) => {
  const patch = (key: string, state: JobState | null) =>
    set((current) => {
      const { [key]: _removed, ...rest } = current.jobs;
      return { jobs: state ? { ...rest, [key]: state } : rest };
    });

  const run = async <T>(
    key: string,
    t: TFunction,
    work: (onProgress: (progress: InstallProgress) => void) => Promise<T>,
    onSuccess: (result: T) => JobState["message"],
  ): Promise<T | null> => {
    patch(key, { progress: { stage: "resolving", doneFiles: 0, totalFiles: 0, doneBytes: 0, totalBytes: 0 } });
    try {
      const result = await work((progress) => patch(key, { progress }));
      patch(key, { message: onSuccess(result) });
      return result;
    } catch (error) {
      if (isLauncherError(error) && error.code === "cancelled") {
        patch(key, null);
      } else {
        patch(key, { message: { kind: "error", text: errorMessage(t, error) } });
      }
      return null;
    }
  };

  return {
    jobs: {},

    install: async (t, instanceId, projectId, versionId) => {
      await run(
        projectId,
        t,
        (onProgress) => installContent(instanceId, projectId, versionId ?? null, onProgress),
        (report) => ({ kind: "success", text: describe(t, report) }),
      );
      await queryClient.invalidateQueries({ queryKey: contentKey(instanceId) });
    },

    installModpack: async (t, projectId) => {
      const instance = await run(
        projectId,
        t,
        (onProgress) => installModpack(projectId, null, onProgress),
        (created) => ({ kind: "success", text: t("content.modpackInstalled", { name: created.name }) }),
      );
      await queryClient.invalidateQueries({ queryKey: INSTANCES_KEY });
      return instance;
    },

    importPack: async (t, path) => {
      const instance = await run(
        "mrpack-import",
        t,
        (onProgress) => importMrpack(path, onProgress),
        (created) => ({ kind: "success", text: t("content.modpackInstalled", { name: created.name }) }),
      );
      await queryClient.invalidateQueries({ queryKey: INSTANCES_KEY });
      return instance;
    },

    update: async (t, instanceId, updates) => {
      await run(
        `update:${instanceId}`,
        t,
        (onProgress) => updateContent(instanceId, updates, onProgress),
        () => ({ kind: "success", text: t("content.updated", { count: updates.length }) }),
      );
      await queryClient.invalidateQueries({ queryKey: contentKey(instanceId) });
    },

    cancel: async (key) => {
      const target = key.startsWith("update:") ? `modrinth-update:${key.slice(7)}` : key === "mrpack-import" ? key : `modrinth:${key}`;
      await cancelInstall(target);
    },

    dismiss: (key) => get().jobs[key] && patch(key, null),
  };
});

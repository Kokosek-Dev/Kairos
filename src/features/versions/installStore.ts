import { create } from "zustand";
import { isLauncherError } from "@/lib/tauri/errors";
import { cancelInstall, installVersion, type InstallProgress } from "@/lib/tauri/versions";

interface InstallState {
  progress: Record<string, InstallProgress>;
  errors: Record<string, string>;
  start: (versionId: string, onFinished: () => void) => Promise<void>;
  cancel: (versionId: string) => Promise<void>;
}

const RESOLVING: InstallProgress = {
  stage: "resolving",
  doneFiles: 0,
  totalFiles: 0,
  doneBytes: 0,
  totalBytes: 0,
};

function without<T>(record: Record<string, T>, key: string): Record<string, T> {
  const { [key]: _removed, ...rest } = record;
  return rest;
}

export const useInstallStore = create<InstallState>((set) => ({
  progress: {},
  errors: {},

  start: async (versionId, onFinished) => {
    set((state) => ({
      progress: { ...state.progress, [versionId]: RESOLVING },
      errors: without(state.errors, versionId),
    }));
    try {
      await installVersion(versionId, (update) =>
        set((state) => ({ progress: { ...state.progress, [versionId]: update } })),
      );
    } catch (error) {
      if (isLauncherError(error) && error.code !== "cancelled") {
        set((state) => ({ errors: { ...state.errors, [versionId]: error.message } }));
      }
    } finally {
      set((state) => ({ progress: without(state.progress, versionId) }));
      onFinished();
    }
  },

  cancel: (versionId) => cancelInstall(versionId),
}));

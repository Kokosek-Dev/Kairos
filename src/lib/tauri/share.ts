import { invoke } from "@tauri-apps/api/core";
import type { Instance, LoaderKind } from "./instances";
import { withProgress } from "./modrinth";
import type { InstallProgress } from "./versions";

export interface CodePreview {
  name: string;
  gameVersion: string;
  loader: LoaderKind;
  loaderVersion: string | null;
  mods: number;
  resourcePacks: number;
  shaders: number;
}

export interface ImportOutcome {
  instance: Instance;
  failed: number;
}

export const exportInstanceCode = (instanceId: string) => invoke<string>("export_instance_code", { instanceId });
export const previewInstanceCode = (code: string) => invoke<CodePreview>("preview_instance_code", { code });
export const importInstanceCode = (code: string, onProgress: (progress: InstallProgress) => void) =>
  withProgress<ImportOutcome>("import_instance_code", { code }, onProgress);

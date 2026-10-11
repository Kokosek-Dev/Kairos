import { invoke } from "@tauri-apps/api/core";

export type InstanceIcon = { kind: "preset"; key: string } | { kind: "custom"; file: string };
export type IconInput = { kind: "preset"; key: string } | { kind: "custom"; path: string };

export type LoaderKind = "vanilla" | "fabric" | "quilt" | "forge" | "neoforge";

export interface LoaderVersion {
  version: string;
  recommended: boolean;
}

export interface Instance {
  schemaVersion: number;
  id: string;
  name: string;
  icon: InstanceIcon;
  gameVersion: string;
  loader: LoaderKind;
  loaderVersion: string | null;
  memoryMb: number;
  jvmArgs: string[];
  javaPath: string | null;
  createdAt: string;
  lastPlayedAt: string | null;
  playTimeSecs: number;
}

export interface NewInstance {
  name: string;
  gameVersion: string;
  icon: IconInput;
  loader: LoaderKind;
  loaderVersion: string | null;
}

export interface InstanceSettings {
  name: string;
  gameVersion: string;
  loader: LoaderKind;
  loaderVersion: string | null;
  memoryMb: number;
  jvmArgs: string[];
  javaPath: string | null;
}

export const listLoaderVersions = (loader: LoaderKind, gameVersion: string) =>
  invoke<LoaderVersion[]>("list_loader_versions", { loader, gameVersion });
export const listInstances = () => invoke<Instance[]>("list_instances");
export const getInstance = (instanceId: string) => invoke<Instance>("get_instance", { instanceId });
export const createInstance = (newInstance: NewInstance) => invoke<Instance>("create_instance", { new: newInstance });
export const updateInstance = (instanceId: string, settings: InstanceSettings) =>
  invoke<Instance>("update_instance", { instanceId, settings });
export const setInstanceIcon = (instanceId: string, icon: IconInput) =>
  invoke<Instance>("set_instance_icon", { instanceId, icon });
export const getInstanceIcon = (instanceId: string) => invoke<string | null>("get_instance_icon", { instanceId });
export const duplicateInstance = (instanceId: string) => invoke<Instance>("duplicate_instance", { instanceId });
export const deleteInstance = (instanceId: string) => invoke<void>("delete_instance", { instanceId });
export const openInstanceFolder = (instanceId: string) => invoke<void>("open_instance_folder", { instanceId });
export const pickIconFile = () => invoke<string | null>("pick_icon_file");
export const pickJavaFile = () => invoke<string | null>("pick_java_file");

export interface MemoryAdvice {
  totalMb: number;
  recommendedMb: number;
  maxMb: number;
}

export const getMemoryAdvice = (modded: boolean) => invoke<MemoryAdvice>("get_memory_advice", { modded });

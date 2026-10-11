import { invoke } from "@tauri-apps/api/core";
import type { LoaderKind } from "./instances";
import { withProgress } from "./modrinth";
import type { InstallProgress } from "./versions";

export interface BoostItem {
  project: string;
  title: string | null;
  installed: boolean;
  reason: string | null;
}

export interface BoostReport {
  items: BoostItem[];
  memoryMb: number;
  javaFlagsAdded: boolean;
}

export const discordAvailable = () => invoke<boolean>("discord_available");
export const boostMods = (loader: LoaderKind) => invoke<string[]>("boost_mods", { loader });
export const boostInstance = (instanceId: string, onProgress: (progress: InstallProgress) => void) =>
  withProgress<BoostReport>("boost_instance", { instanceId }, onProgress);

import { Channel, invoke } from "@tauri-apps/api/core";

export type VersionKind = "release" | "snapshot" | "old_beta" | "old_alpha";

export interface ManifestEntry {
  id: string;
  type: VersionKind;
  url: string;
  sha1: string;
  releaseTime: string;
}

export interface VersionManifest {
  latest: { release: string; snapshot: string };
  versions: ManifestEntry[];
}

export interface InstallProgress {
  stage: "resolving" | "downloading" | "done";
  doneFiles: number;
  totalFiles: number;
  doneBytes: number;
  totalBytes: number;
}


export const getVersionManifest = () => invoke<VersionManifest>("get_version_manifest");

export const listInstalledVersions = () => invoke<string[]>("list_installed_versions");

export function installVersion(
  versionId: string,
  onProgress: (progress: InstallProgress) => void,
): Promise<void> {
  const channel = new Channel<InstallProgress>();
  channel.onmessage = onProgress;
  return invoke<void>("install_version", { versionId, onProgress: channel });
}

export const cancelInstall = (versionId: string) => invoke<void>("cancel_install", { versionId });

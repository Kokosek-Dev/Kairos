import { invoke } from "@tauri-apps/api/core";
import type { LoaderKind } from "./instances";

export type MomentReason = "manual" | "before_update" | "before_change" | "before_restore";

export interface Moment {
  id: string;
  note: string;
  reason: MomentReason;
  createdAt: string;
  includesWorlds: boolean;
  sizeBytes: number;
  mods: string[];
  settings: {
    gameVersion: string;
    loader: LoaderKind;
    loaderVersion: string | null;
    memoryMb: number;
  };
}

export interface MomentDifference {
  added: string[];
  removed: string[];
  versionChanged: boolean;
}

export interface MomentSizes {
  settingsBytes: number;
  worldsBytes: number;
}

export const listMoments = (instanceId: string) => invoke<Moment[]>("list_moments", { instanceId });
export const createMoment = (instanceId: string, note: string, includeWorlds: boolean) =>
  invoke<Moment>("create_moment", { instanceId, note, includeWorlds });
export const createSafetyMoment = (instanceId: string, reason: "before_update" | "before_change") =>
  invoke<Moment>("create_safety_moment", { instanceId, reason });
export const restoreMoment = (instanceId: string, momentId: string) => invoke<void>("restore_moment", { instanceId, momentId });
export const deleteMoment = (instanceId: string, momentId: string) => invoke<void>("delete_moment", { instanceId, momentId });
export const compareMoment = (instanceId: string, momentId: string) =>
  invoke<MomentDifference>("compare_moment", { instanceId, momentId });
export const momentSizes = (instanceId: string) => invoke<MomentSizes>("moment_sizes", { instanceId });

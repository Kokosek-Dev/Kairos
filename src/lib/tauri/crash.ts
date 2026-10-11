import { invoke } from "@tauri-apps/api/core";

export type CrashKind =
  | "heap_too_large"
  | "out_of_memory"
  | "wrong_java"
  | "missing_dependency"
  | "mod_conflict"
  | "graphics_driver"
  | "mod_problem"
  | "unknown";

export interface CrashAnalysis {
  kind: CrashKind;
  details: string[];
  reportPath: string | null;
}

export const analyzeCrash = (instanceId: string, startedMs: number) =>
  invoke<CrashAnalysis | null>("analyze_crash", { instanceId, startedMs });
export const revealFile = (path: string) => invoke<void>("reveal_file", { path });
export const saveProblemReport = (instanceId: string | null) =>
  invoke<string | null>("save_problem_report", { instanceId });

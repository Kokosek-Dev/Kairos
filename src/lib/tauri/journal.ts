import { invoke } from "@tauri-apps/api/core";

export interface Session {
  instanceId: string;
  /** Unix time in seconds when the game window opened. */
  startedAt: number;
  durationSecs: number;
}

export const listSessions = () => invoke<Session[]>("list_sessions");

import { Channel, invoke } from "@tauri-apps/api/core";

export type LogStream = "stdout" | "stderr";
export type PrepareStep = "version" | "java" | "files";

export type LaunchEvent =
  | { type: "queued"; runId: number }
  | { type: "preparing"; step: PrepareStep; doneBytes: number; totalBytes: number }
  | { type: "running"; pid: number | null }
  | { type: "log"; stream: LogStream; level: string | null; line: string }
  | { type: "exited"; code: number | null };

export function launchGame(versionId: string, onEvent: (event: LaunchEvent) => void): Promise<void> {
  const channel = new Channel<LaunchEvent>();
  channel.onmessage = onEvent;
  return invoke<void>("launch_game", { versionId, onEvent: channel });
}

export const stopGame = (runId: number) => invoke<void>("stop_game", { runId });

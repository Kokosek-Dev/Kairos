import { create } from "zustand";
import { isLauncherError } from "@/lib/tauri/versions";
import {
  launchGame,
  stopGame,
  type LaunchEvent,
  type LogStream,
  type PrepareStep,
} from "@/lib/tauri/launch";

const MAX_LOG_LINES = 5000;

export type GamePhase =
  | { kind: "idle" }
  | { kind: "preparing"; step: PrepareStep; doneBytes: number; totalBytes: number }
  | { kind: "running"; pid: number | null }
  | { kind: "exited"; code: number | null }
  | { kind: "failed"; message: string };

export interface LogLine {
  id: number;
  stream: LogStream;
  level: string | null;
  text: string;
}

interface GameState {
  versionId: string | null;
  runId: number | null;
  phase: GamePhase;
  lines: LogLine[];
  consoleOpen: boolean;
  play: (versionId: string) => Promise<void>;
  stop: () => Promise<void>;
  clear: () => void;
  setConsoleOpen: (open: boolean) => void;
}

let nextLineId = 0;

export const isBusy = (phase: GamePhase): boolean => phase.kind === "preparing" || phase.kind === "running";

export const useGameStore = create<GameState>((set, get) => {
  const handle = (event: LaunchEvent) => {
    switch (event.type) {
      case "queued":
        set({ runId: event.runId });
        break;
      case "preparing":
        set({
          phase: { kind: "preparing", step: event.step, doneBytes: event.doneBytes, totalBytes: event.totalBytes },
        });
        break;
      case "running":
        set({ phase: { kind: "running", pid: event.pid } });
        break;
      case "log":
        set((state) => ({
          lines: [...state.lines, { id: nextLineId++, stream: event.stream, level: event.level, text: event.line }].slice(-MAX_LOG_LINES),
        }));
        break;
      case "exited":
        set({ phase: { kind: "exited", code: event.code }, runId: null });
        break;
    }
  };

  return {
    versionId: null,
    runId: null,
    phase: { kind: "idle" },
    lines: [],
    consoleOpen: false,

    play: async (versionId) => {
      if (isBusy(get().phase)) return;
      set({
        versionId,
        runId: null,
        lines: [],
        consoleOpen: true,
        phase: { kind: "preparing", step: "version", doneBytes: 0, totalBytes: 0 },
      });
      try {
        await launchGame(versionId, handle);
      } catch (error) {
        if (isLauncherError(error) && error.code === "cancelled") {
          set({ phase: { kind: "idle" }, runId: null });
        } else {
          const message = isLauncherError(error) ? error.message : String(error);
          set({ phase: { kind: "failed", message }, runId: null });
        }
      }
    },

    stop: async () => {
      const { runId } = get();
      if (runId !== null) await stopGame(runId);
    },

    clear: () => set({ lines: [] }),
    setConsoleOpen: (open) => set({ consoleOpen: open }),
  };
});

import { create } from "zustand";
import { contentKey } from "@/features/discover/jobsStore";
import { useGameStore } from "@/features/launch/gameStore";
import { usePreferences } from "@/features/settings/preferences";
import { queryClient } from "@/lib/queryClient";
import { listContent, removeContent, setContentEnabled } from "@/lib/tauri/modrinth";
import { createSafetyMoment } from "@/lib/tauri/moments";

export type DetectiveStage = "closed" | "intro" | "launching" | "asking" | "found" | "multiple" | "error";

interface DetectiveState {
  stage: DetectiveStage;
  instanceId: string;
  title: string;
  /** Mods the search may switch off: enabled mods that are not dependencies of other mods. */
  candidates: string[];
  titles: Record<string, string>;
  /** Mods that still might be the culprit. */
  suspects: string[];
  /** Mods switched on in the test that is running now. */
  testing: string[];
  culprit: string | null;
  checking: boolean;
  round: number;
  open: (instanceId: string, title: string) => Promise<void>;
  start: () => Promise<void>;
  answer: (problemShown: boolean) => Promise<void>;
  finish: (removeCulprit: boolean) => Promise<void>;
  cancel: () => Promise<void>;
}

const CLOSED = {
  stage: "closed" as DetectiveStage,
  instanceId: "",
  title: "",
  candidates: [] as string[],
  titles: {} as Record<string, string>,
  suspects: [] as string[],
  testing: [] as string[],
  culprit: null as string | null,
  checking: false,
  round: 0,
};

const firstHalf = (items: string[]): string[] => items.slice(0, Math.ceil(items.length / 2));

const refresh = (instanceId: string) => queryClient.invalidateQueries({ queryKey: contentKey(instanceId) });

async function switchTo(instanceId: string, candidates: string[], enabled: string[]) {
  await Promise.all(candidates.map((file) => setContentEnabled(instanceId, "mod", file, enabled.includes(file))));
  await refresh(instanceId);
}

export const useDetectiveStore = create<DetectiveState>((set, get) => {
  const launch = async (candidates: string[], testing: string[]) => {
    const { instanceId, title } = get();
    try {
      await switchTo(instanceId, candidates, testing);
      set((state) => ({ stage: "launching", testing, round: state.round + 1 }));
      await useGameStore.getState().play(instanceId, title, usePreferences.getState().incognito, true);
      if (useGameStore.getState().phase.kind === "idle") await get().cancel();
    } catch {
      set({ stage: "error" });
    }
  };

  const nextTest = async () => {
    const { candidates, suspects } = get();
    if (suspects.length === 1) {
      const culprit = suspects[0];
      set({ culprit, checking: true });
      await launch(candidates, candidates.filter((file) => file !== culprit));
    } else {
      await launch(candidates, firstHalf(suspects));
    }
  };

  const conclude = async (problemShown: boolean) => {
    const { candidates, suspects, testing, checking, instanceId } = get();
    if (checking) {
      set({ stage: problemShown ? "multiple" : "found" });
      if (problemShown) await switchTo(instanceId, candidates, candidates);
      return;
    }
    set({ suspects: problemShown ? testing : suspects.filter((file) => !testing.includes(file)) });
    await nextTest();
  };

  useGameStore.subscribe((state, previous) => {
    if (get().stage !== "launching" || state.phase === previous.phase) return;
    if (state.phase.kind === "exited") {
      useGameStore.getState().dismissCrash();
      if (state.crash) void conclude(true);
      else set({ stage: "asking" });
    } else if (state.phase.kind === "failed") {
      set({ stage: "error" });
    }
  });

  return {
    ...CLOSED,

    open: async (instanceId, title) => {
      const items = await listContent(instanceId);
      const mods = items.filter((item) => item.kind === "mod" && item.enabled && !item.entry?.dependency);
      const titles = Object.fromEntries(mods.map((item) => [item.fileName, item.entry?.title ?? item.fileName]));
      set({ ...CLOSED, stage: "intro", instanceId, title, candidates: mods.map((item) => item.fileName), titles });
    },

    start: async () => {
      const { instanceId, candidates } = get();
      await createSafetyMoment(instanceId, "before_change").catch(() => undefined);
      set({ suspects: candidates, culprit: null, checking: false, round: 0 });
      await nextTest();
    },

    answer: (problemShown) => conclude(problemShown),

    finish: async (removeCulprit) => {
      const { instanceId, culprit } = get();
      if (removeCulprit && culprit) await removeContent(instanceId, "mod", culprit);
      await refresh(instanceId);
      set(CLOSED);
    },

    cancel: async () => {
      const { instanceId, candidates, stage } = get();
      if (stage === "closed") return;
      set(CLOSED);
      if (stage === "intro") return;
      await useGameStore.getState().stop();
      await switchTo(instanceId, candidates, candidates).catch(() => undefined);
    },
  };
});
